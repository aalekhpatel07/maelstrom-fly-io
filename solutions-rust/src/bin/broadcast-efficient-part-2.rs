use std::{sync::mpsc::{channel, RecvTimeoutError}, thread::spawn, collections::{HashMap, HashSet}, ops::Rem, time::{Instant, Duration}};

use maelstrom::*;
use serde::{Serialize, Deserialize};


const SYNC_INTERVAL: Duration = Duration::from_millis(250);
const STRIDE: usize = 4;


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Init {
        node_id: String,
        node_ids: Vec<String>
    },
    InitOk,
    Topology {
        topology: HashMap<String, Vec<String>>
    },
    TopologyOk,
    Broadcast {
        message: usize
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>
    },
    Sync {
        messages: Vec<usize>
    },
    SyncOk {
        messages: Vec<usize>
    }
}


#[derive(Debug, Default)]
pub struct RemoteNodeHandler {
    unacknowledged_messages: Vec<usize>
}

impl RemoteNodeHandler {
    fn new() -> Self {
        Default::default()
    }

    fn send_message(&mut self, message: usize) {
        self.unacknowledged_messages.push(message);
    }

    pub fn acknowledge_synced(&mut self, messages: &[usize]) {
        self.unacknowledged_messages.retain(|message| !messages.contains(message));
    }
}


pub fn main() {

    let (tx, rx) = channel::<Envelope<Message>>();

    spawn(move || {
        read_stdin(tx);
    });

    let mut remote_node_handlers: HashMap<String, RemoteNodeHandler> = Default::default();
    let mut our_neighbors: Vec<String> = Default::default();

    let mut messages: HashSet<usize> = HashSet::new();
    let mut all_node_ids: Vec<String> = Default::default();
    let mut our_id = Default::default();

    let mut deadline = Instant::now() + SYNC_INTERVAL;

    loop {
        let should_wait_for_at_most = deadline - Instant::now(); 
        match rx.recv_timeout(should_wait_for_at_most) {
            Ok(envelope) => {
                match envelope.message() {
                    Message::Init { node_id, node_ids } => {
                        our_id = node_id.to_owned();
                        all_node_ids = node_ids.clone();
                        for (_, node_id) in node_ids.iter().enumerate() {
                            remote_node_handlers.insert(node_id.clone(), RemoteNodeHandler::new());
                        }
                        envelope.reply(Message::InitOk).send();
                    },

                    Message::Topology { .. } => {
                        // Let's create a topology where
                        // 1 of every STRIDE nodes of our cluster
                        // (except us) is our neighbor.
                        // Don't respect the topology. Just create a custom one.
                        let our_position = all_node_ids.iter().position(|node_id| node_id == &our_id).unwrap();

                        our_neighbors = 
                            all_node_ids
                            .iter()
                            .skip((our_position + 1) % STRIDE)
                            .step_by(STRIDE)
                            .cloned()
                            .collect();

                        envelope.reply(Message::TopologyOk).send();
                    },

                    Message::Broadcast { message } => {

                        if messages.insert(*message) {
                            for neighbor in &our_neighbors {
                                remote_node_handlers.get_mut(neighbor).unwrap().send_message(*message);
                            }
                        }
                        envelope.reply(Message::BroadcastOk).send();
                    },

                    Message::BroadcastOk => {

                    },

                    Message::Read => {
                        envelope.reply(Message::ReadOk { messages: messages.iter().copied().collect() }).send();
                    },

                    Message::Sync { messages: inbound } => {
                        for &message in inbound {
                            if messages.insert(message) {
                                for neighbor in &our_neighbors {
                                    remote_node_handlers.get_mut(neighbor).unwrap().send_message(message);
                                }
                            }
                        }
                        envelope.reply(Message::SyncOk { messages: inbound.iter().copied().collect() }).send();
                    }

                    Message::SyncOk { messages: acknowledged_messages } => {
                        remote_node_handlers.get_mut(&envelope.src).unwrap().acknowledge_synced(acknowledged_messages);
                    }

                    _ => unimplemented!()
                }
            },
            Err(RecvTimeoutError::Disconnected) => {},
            Err(RecvTimeoutError::Timeout) => {}
        }

        if Instant::now() >= deadline {
            remote_node_handlers
            .iter()
            .for_each(|(remote_node_id, remote_node_handler)| {
                if !remote_node_handler.unacknowledged_messages.is_empty() {
                    Envelope::new(
                        &our_id, 
                        &remote_node_id, 
                        None, 
                        Message::Sync { 
                            messages: remote_node_handler.unacknowledged_messages.iter().copied().collect()
                        }
                    ).send();
                }
            });
            deadline += SYNC_INTERVAL;
        }
    }

}

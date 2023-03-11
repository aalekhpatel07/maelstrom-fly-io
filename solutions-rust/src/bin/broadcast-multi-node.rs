
use std::{sync::mpsc::{channel, RecvTimeoutError}, thread::spawn, collections::{HashMap, HashSet}, ops::Rem, time::{Instant, Duration}};

use maelstrom::*;
use serde::{Serialize, Deserialize};


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

    const SYNC_INTERVAL: Duration = Duration::from_millis(250);

    let mut remote_node_handlers: HashMap<String, RemoteNodeHandler> = Default::default();
    let mut our_neighbors: Vec<String> = Default::default();

    let mut messages: HashSet<usize> = HashSet::new();
    let mut our_id = Default::default();

    let mut deadline = Instant::now() + SYNC_INTERVAL;

    loop {
        let should_wait_for_at_most = deadline - Instant::now();
        match rx.recv_timeout(should_wait_for_at_most) {
            Ok(envelope) => {
                match envelope.message() {
                    // Create new handlers for every node in the cluster.
                    // This is for state-keeping for individual nodes using an actor pattern.
                    Message::Init { node_id, node_ids } => {
                        our_id = node_id.to_owned();
                        for (_, node_id) in node_ids.iter().enumerate() {
                            remote_node_handlers.insert(node_id.clone(), RemoteNodeHandler::new());
                        }
                        envelope.reply(Message::InitOk).send();
                    },

                    // Set up our topology.
                    Message::Topology { topology } => {
                        our_neighbors = topology.get(&our_id).unwrap().clone();
                        envelope.reply(Message::TopologyOk).send();
                    },

                    // Standard broadcast from a client. Just write to our messages
                    // and if we hadn't seen before, dump it to the buffer for every
                    // remote server, so that it can be sent later.
                    Message::Broadcast { message } => {
                        if messages.insert(*message) {
                            for neighbor in &our_neighbors {
                                remote_node_handlers.get_mut(neighbor).unwrap().send_message(*message);
                            }
                        }
                        envelope.reply(Message::BroadcastOk).send();
                    },
                    // We're using a different channel of comms amongst
                    // internal nodes so we won't reuse Broadcast.
                    Message::BroadcastOk => {},
                    Message::Read => {
                        envelope.reply(Message::ReadOk { messages: messages.iter().copied().collect() }).send();
                    },
                    // Sync's are internal comms that servers use to populate local buffers
                    // that get flushed periodically as a single message.
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
                    // SyncOk's are internal messages from other servers
                    // that we can use to mark some messages as acknowledged, in bulk,
                    // for that given server.
                    Message::SyncOk { messages: acknowledged_messages } => {
                        remote_node_handlers.get_mut(&envelope.src).unwrap().acknowledge_synced(acknowledged_messages);
                    }

                    _ => unimplemented!()
                }
            },
            Err(_) => {},
        }

        // The buffer flush may be due, so take care of it.
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

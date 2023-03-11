use std::{collections::{HashSet, HashMap}, sync::{mpsc::{Receiver, channel}, Arc, Mutex, RwLock}, fs::Metadata, thread::{spawn, JoinHandle}};

use maelstrom_common::*;
use broadcast::*;


#[derive(Debug, Default)]
pub struct NodeMetadata {
    id: Option<String>,
    node_ids: HashSet<String>,
    topology: HashMap<String, HashSet<String>>
}

#[derive(Debug, Default)]
pub struct Node {
    pub metadata: NodeMetadata,
    pub messages: HashSet<usize>
}

#[derive(Debug)]
pub struct MessageBroadcaster {
    rx: Receiver<Envelope<Message>>,
    state: Arc<Mutex<Node>>,
}

impl MessageBroadcaster {

    pub fn new(rx: Receiver<Envelope<Message>>, state: Arc<Mutex<Node>>) -> Self {
        Self {
            rx,
            state
        }
    }
    pub fn start(self) -> JoinHandle<()> {
        spawn(move || {
            self.run()
        })
    }

    pub fn run(self) {
        while let Ok(envelope) = self.rx.recv() {
            let mut guard = self.state.lock().unwrap();
            if guard.metadata.id.is_some() && Some(&envelope.dest) != guard.metadata.id.as_ref() {
                panic!("Received an envelope meant for someone else. wtf...");
            }

            match &envelope.body {
                Message::Init { msg_id, node_id, node_ids } => {                    
                    guard.metadata.id = Some(node_id.clone());
                    guard.metadata.node_ids = node_ids.clone();
                    envelope.reply(
                        Message::InitOk { in_reply_to: *msg_id }
                    ).send();
                },
                Message::Topology { msg_id, topology } => {
                    guard.metadata.topology = topology.clone();
                    envelope.reply(
                        Message::TopologyOk { in_reply_to: *msg_id }
                    ).send();
                },
                Message::Broadcast { msg_id, message } => {
                    // We're guaranteed that a single client doesn't receive any duplicated
                    // messages but a server node could still send us a broadcast, so we'll have
                    // to ignore it.
                    let our_id = guard.metadata.id.clone().unwrap();
                    
                    if !guard.messages.contains(message) {
                        guard.messages.insert(*message);

                        guard
                        .metadata
                        .topology
                        .keys()
                        .filter(
                            |&neighbor| neighbor != &envelope.src && neighbor != &our_id
                        )
                        .for_each(|neighbor| {
                            Envelope::new(
                                &guard.metadata.id.as_ref().unwrap(), 
                                &neighbor, 
                                Message::Broadcast { msg_id: None, message: *message }
                            )
                            .send();
                        });

                        // Only reply the clients.
                        if msg_id.is_some() {
                            envelope.reply(
                                Message::BroadcastOk { in_reply_to: *msg_id }
                            ).send();
                        }
                    }
                },
                Message::Read { msg_id } => {                    
                    let messages = guard.messages.clone();
                    envelope.reply(
                        Message::ReadOk { in_reply_to: *msg_id, messages }
                    ).send();
                },
                Message::BroadcastOk { .. } => {
                    // No-op. We don't care about ack-ing our neighbors.
                    // because for this challenge they'll always be available.
                },
                _ => {
                    panic!("WTF")
                }
            }
        }
    }
}


pub fn main() {
    let state = Arc::new(Mutex::new(Default::default()));

    let (tx, rx) = channel();
    let mut post_office = PostOffice::new(rx);

    let acknowledger = MessageBroadcaster::new(post_office.subscribe(), state);
    post_office.start();
    acknowledger.start();

    listen(tx);
}
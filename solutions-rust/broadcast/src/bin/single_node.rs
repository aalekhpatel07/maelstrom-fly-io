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
pub struct MessageAcknowledger {
    rx: Receiver<Envelope<Message>>,
    state: Arc<Mutex<Node>>,
}

impl MessageAcknowledger {

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
                    guard.messages.insert(*message);
                    envelope.reply(
                        Message::BroadcastOk { in_reply_to: *msg_id }
                    ).send();
                },
                Message::Read { msg_id } => {                    
                    let messages = guard.messages.clone();
                    envelope.reply(
                        Message::ReadOk { in_reply_to: *msg_id, messages }
                    ).send();
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

    let acknowledger = MessageAcknowledger::new(post_office.subscribe(), state);
    post_office.start();
    acknowledger.start();

    listen(tx);
}
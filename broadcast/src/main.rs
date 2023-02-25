use std::{collections::HashMap, sync::{Arc, Mutex}};
use maelstrom_common::{Actor, Envelope, run};
use serde::{Serialize, Deserialize};


pub type SharedState<T> = Arc<Mutex<T>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "init")]
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    #[serde(rename = "broadcast")]
    Broadcast {
        msg_id: usize,
        message: usize,
    },
    #[serde(rename = "read")]
    Read {
        msg_id: usize,
    },
    #[serde(rename = "topology")]
    Topology {
        msg_id: usize,
        topology: HashMap<String, Vec<String>>
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "init_ok")]
    InitOk {
        in_reply_to: usize,
    },
    #[serde(rename = "broadcast_ok")]
    BroadcastOk {
        in_reply_to: usize,
    },
    #[serde(rename = "read_ok")]
    Read {
        in_reply_to: usize,
        messages: Vec<usize>,
    },
    #[serde(rename = "topology_ok")]
    Topology {
        in_reply_to: usize,
    },
}

#[derive(Debug, Default)]
pub struct Broadcast {
    pub node_id: Option<String>,
    pub neighbors: Vec<String>,
    pub messages: Vec<usize>,
}


impl Actor for Broadcast {
    type InboundMessage = Request;
    type OutboundMessage = Response;

    fn handle_message(
        &mut self,
        msg: maelstrom_common::Envelope<Self::InboundMessage>,
    ) -> Option<maelstrom_common::Envelope<Self::OutboundMessage>> {
        match msg.body {
            Request::Init { msg_id, node_id, node_ids } => {
                self.node_id = Some(node_id);
                self.neighbors = node_ids;
                let body = Response::InitOk { in_reply_to: msg_id };
                Some(
                    Envelope {
                        src: msg.dest,
                        dest: msg.src,
                        body
                    }
                )
            },
            Request::Topology { msg_id, topology } => {
                let node_id = self.node_id.clone().unwrap();
                self.neighbors = topology.get(&node_id).expect("to find a set of neighbors for us.").clone();

                let body = Response::Topology { in_reply_to: msg_id };

                Some(
                    Envelope {
                        src: msg.dest,
                        dest: msg.src,
                        body
                    }
                )
            },
            Request::Broadcast { msg_id, message } => {
                self.messages.push(message);
                let body = Response::BroadcastOk { in_reply_to: msg_id };
                Some(
                    Envelope {
                        src: msg.dest,
                        dest: msg.src,
                        body
                    }
                )
            },
            Request::Read { msg_id } => {
                let body = Response::Read {
                    in_reply_to: msg_id,
                    messages: self.messages.clone(),
                };
                Some(
                    Envelope {
                        src: msg.dest,
                        dest: msg.src,
                        body
                    }
                )
            },
        }
    }
}


pub fn main() {
    run(Broadcast::default()).unwrap();
}

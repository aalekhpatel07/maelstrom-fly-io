use broadcast::{Request, Response};
use maelstrom_common::{run, Actor, Envelope};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Broadcast {
    pub node_id: Option<String>,
    pub neighbors: Vec<String>,

    // The order is not important.
    pub messages: HashSet<usize>,
}

impl Actor for Broadcast {
    type InboundMessage = Request;
    type OutboundMessage = Response;

    fn handle_message(
        &mut self,
        msg: maelstrom_common::Envelope<Self::InboundMessage>,
    ) -> Option<maelstrom_common::Envelope<Self::OutboundMessage>> {
        match msg.body {
            Request::Init {
                msg_id,
                node_id,
                node_ids,
            } => {
                self.node_id = Some(node_id);
                self.neighbors = node_ids;
                let body = Response::InitOk {
                    in_reply_to: msg_id,
                };
                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            }
            Request::Topology { msg_id, topology } => {
                let node_id = self.node_id.clone().unwrap();
                self.neighbors = topology
                    .get(&node_id)
                    .expect("to find a set of neighbors for us.")
                    .clone();

                let body = Response::Topology {
                    in_reply_to: msg_id,
                };

                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            }
            Request::Broadcast { msg_id, message } => {
                self.messages.insert(message);
                let body = Response::BroadcastOk {
                    in_reply_to: msg_id,
                };
                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            }
            Request::Read { msg_id } => {
                let body = Response::Read {
                    in_reply_to: msg_id,
                    messages: self.messages.clone().into_iter().collect::<Vec<_>>(),
                };
                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            }
            _ => {
                unreachable!(
                    "Single node set up doesn't expect to receive 
                    any internal communication from other nodes."
                )
            }
        }
    }
}

pub fn main() {
    run(Broadcast::default()).unwrap();
}

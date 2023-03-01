use broadcast::{Request, Response};
use maelstrom_common::{run, Actor};
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
        msg: maelstrom_common::Envelope<maelstrom_common::Message<Self::InboundMessage, Self::OutboundMessage>>,
    ) -> Option<maelstrom_common::Envelope<maelstrom_common::Message<Self::InboundMessage, Self::OutboundMessage>>> {
        Some(match msg.body {
            maelstrom_common::Message::Inbound(Request::Init {
                msg_id,
                ref node_id,
                ref node_ids,
            }) => {
                self.node_id = Some(node_id.clone());
                self.neighbors = node_ids.clone();
                msg.reply(maelstrom_common::Message::Outbound(Response::InitOk {
                    in_reply_to: msg_id,
                }))
            }
            maelstrom_common::Message::Inbound(Request::Topology {
                msg_id,
                ref topology,
            }) => {
                let node_id = self.node_id.clone().unwrap();
                self.neighbors = topology
                    .get(&node_id)
                    .expect("to find a set of neighbors for us.")
                    .clone();
                msg.reply(maelstrom_common::Message::Outbound(Response::Topology {
                    in_reply_to: msg_id,
                }))
            }
            maelstrom_common::Message::Inbound(Request::Broadcast { msg_id, message }) => {
                self.messages.insert(message);
                msg.reply(maelstrom_common::Message::Outbound(Response::BroadcastOk {
                    in_reply_to: msg_id,
                }))
            },
            maelstrom_common::Message::Inbound(Request::Read { msg_id }) => msg.reply(maelstrom_common::Message::Outbound(Response::Read {
                in_reply_to: msg_id,
                messages: self.messages.clone().into_iter().collect::<Vec<_>>(),
            })),
            _ => {
                panic!("Unexpected message: {:?}", msg);
            }
        })
    }
}

pub fn main() {
    run(Broadcast::default()).unwrap();
}

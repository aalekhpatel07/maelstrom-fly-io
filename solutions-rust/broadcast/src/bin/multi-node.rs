use broadcast::{Request, Response};
use maelstrom_common::{run, Actor, Envelope, Message};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Broadcast {
    pub node_id: Option<String>,
    pub neighbors: HashSet<String>,
    pub messages: HashSet<usize>,
    pub all_nodes: Option<HashSet<String>>,
}

impl Actor for Broadcast {
    type InboundMessage = Request;
    type OutboundMessage = Response;

    fn handle_message(
        &mut self,
        msg: maelstrom_common::Envelope<Message<Self::InboundMessage, Self::OutboundMessage>>,
    ) -> Option<maelstrom_common::Envelope<Message<Self::InboundMessage, Self::OutboundMessage>>> {
        match msg.body {
            Message::Inbound(Request::Init {
                msg_id,
                node_id,
                node_ids,
            }) => {
                self.node_id = Some(node_id);
                self.all_nodes = Some(node_ids.into_iter().collect());

                let body = Message::Outbound(Response::InitOk {
                    in_reply_to: msg_id,
                });

                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            }
            Message::Inbound(Request::Topology { msg_id, topology }) => {
                let node_id = self.node_id.clone().unwrap();
                self.neighbors = topology
                    .get(&node_id)
                    .expect("to find a set of neighbors for us.")
                    .clone()
                    .into_iter()
                    .collect();

                let body = Message::Outbound(Response::Topology {
                    in_reply_to: msg_id,
                });

                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            }

            Message::Inbound(Request::Broadcast { msg_id, message }) => {
                if self.messages.contains(&message) {
                    return Some(Envelope {
                        src: self.node_id.clone().unwrap(),
                        dest: msg.src,
                        body: Message::Outbound(Response::BroadcastOk {
                            in_reply_to: msg_id,
                        }),
                    });
                }

                self.messages.insert(message);

                self.neighbors.iter().for_each(|neighbour| {
                    if neighbour == &msg.src {
                        return;
                    }
                    let body: Message<Self::InboundMessage, Self::OutboundMessage> = Message::Inbound(Request::Broadcast {
                        msg_id: None,
                        message,
                    });
                    let envelope: Envelope<Message<Self::InboundMessage, Self::OutboundMessage>> = Envelope {
                        src: self.node_id.clone().unwrap(),
                        dest: neighbour.clone(),
                        body,
                    };
                    self.send_message(envelope).unwrap();
                });

                Some(Envelope {
                    src: self.node_id.clone().unwrap(),
                    dest: msg.src,
                    body: Message::Outbound(Response::BroadcastOk {
                        in_reply_to: msg_id,
                    }),
                })
            }

            Message::Inbound(Request::Read { msg_id }) => {
                let body = Message::Outbound(Response::Read {
                    in_reply_to: msg_id,
                    messages: self.messages.clone().into_iter().collect::<Vec<_>>(),
                });
                Some(Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body,
                })
            },
            Message::Outbound(Response::BroadcastOk { .. }) => {
                None
            }
            _ => {
                panic!("Unexpected message: {:?}", msg);
            }
        }
    }
}

pub fn main() {
    run(Broadcast::default()).unwrap();
}

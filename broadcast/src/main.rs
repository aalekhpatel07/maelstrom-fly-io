use broadcast::{InternalRequest, InternalResponse, Request, Response};
use maelstrom_common::{formatted_log, run, Actor, Envelope};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Broadcast {
    pub node_id: Option<String>,
    pub neighbors: HashSet<String>,
    pub messages: HashSet<usize>,
    pub all_nodes: Option<HashSet<String>>,
}

impl Broadcast {
    /// TLDR; Relay a message to each of the specified
    /// nodes, and ask them to relay it to their
    /// neighbors in turn, if they can.
    ///
    /// Send a message to each neighbor.
    /// For nodes outside our neighborhood, we will
    /// ask our neighbors if they can help forward the message,
    /// and if they can't they will ask their neighbors, and so on.

    /// **Note**: Let's hope the network is connected so eventually
    /// everyone gets the message. Challenge 3b doesn't have network
    /// partitions, so we shouldn't have to worry about that.
    pub fn propagate(
        &mut self,
        message: usize,
        forward_to: Vec<String>,
        remaining_nodes: Vec<String>,
    ) {
        let our_id = self.node_id.clone().unwrap();

        forward_to.iter().for_each(|neighbor| {
            let body = Request::Internal {
                request: InternalRequest::Add {
                    value: message,
                    remaining_nodes: remaining_nodes.clone(),
                },
            };

            let envelope = Envelope {
                src: our_id.clone(),
                dest: neighbor.clone(),
                body,
            };

            self.send_message(envelope).unwrap();
        });
    }

    /// Some neighbor asked us for help forwarding a message.
    /// We'll try to forward it to our neighbors, if we don't know
    /// whether they've already received the message. Then we'll ask
    /// them to forward it to their neighbors, and so on, until
    /// everyone has received the message.
    pub fn process_internal_message_request(&mut self, msg: InternalRequest) -> Option<Response> {
        formatted_log!("INTERNAL", "Processing internal message: {:#?}", msg);
        match msg {
            InternalRequest::Add {
                value,
                remaining_nodes,
            } => {
                self.messages.insert(value);

                // Now, check if we can forward the message to any of the remaining nodes.

                // Who can and should we forward?
                let forward_to = self
                    .neighbors
                    .intersection(&remaining_nodes.clone().into_iter().collect())
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect::<HashSet<_>>();

                // Who should we tell them to forward to, in case
                // we couldn't saturate the network with this message?
                let remaining_nodes = remaining_nodes
                    .into_iter()
                    .collect::<HashSet<_>>()
                    .difference(&forward_to)
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect::<HashSet<_>>();

                self.propagate(
                    value,
                    forward_to.clone().into_iter().collect(),
                    remaining_nodes.into_iter().collect(),
                );

                Some(Response::Internal {
                    response: InternalResponse::AddOk {
                        forwarded_to: forward_to.into_iter().collect(),
                    },
                })
            }
        }
    }
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
                self.all_nodes = Some(node_ids.into_iter().collect());

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
                    .clone()
                    .into_iter()
                    .collect();

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

                // Initially, for any node outside of our topology,
                // we're gonna ask our neighbors if they can help us.

                let remaining_nodes = self
                    .all_nodes
                    .clone()
                    .unwrap()
                    .difference(&self.neighbors)
                    .into_iter()
                    .map(|s| s.to_owned())
                    .collect::<Vec<_>>();

                let forward_to = self.neighbors.clone().into_iter().collect();

                self.propagate(message, forward_to, remaining_nodes);

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

            Request::Internal { request } => {
                self.process_internal_message_request(request)
                    .map(|response| Envelope {
                        src: msg.dest,
                        dest: msg.src,
                        body: response,
                    })
            }
        }
    }
}

pub fn main() {
    run(Broadcast::default()).unwrap();
}

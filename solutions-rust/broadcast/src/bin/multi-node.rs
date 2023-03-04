use broadcast::Message;
use maelstrom_common::{run, Envelope, HandleMessage};
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct Broadcast {
    pub node_id: String,
    pub neighbors: HashSet<String>,
    pub messages: HashSet<usize>,
    pub all_nodes: HashSet<String>,
}

impl HandleMessage for Broadcast {
    type Message = Message;
    type Error = broadcast::Error;

    fn handle_message(
        &mut self,
        msg: Envelope<Self::Message>,
        outbound_msg_tx: std::sync::mpsc::Sender<Envelope<Self::Message>>,
    ) -> Result<(), Self::Error> {
        match msg.body {
            Message::Init {
                msg_id,
                node_id,
                node_ids,
            } => {
                self.node_id = node_id;
                self.all_nodes = node_ids;

                let payload = Message::InitOk {
                    in_reply_to: msg_id,
                };
                let reply = Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body: payload,
                };
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            Message::Topology { msg_id, topology } => {
                self.neighbors = topology
                    .get(&self.node_id)
                    .expect("to find a set of neighbors for us.")
                    .clone();

                let payload = Message::TopologyOk {
                    in_reply_to: msg_id,
                };
                let reply = Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body: payload,
                };
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            Message::Broadcast { msg_id, message } => {
                if self.messages.contains(&message) {
                    let reply = Envelope {
                        src: self.node_id.clone(),
                        dest: msg.src.clone(),
                        body: Message::BroadcastOk {
                            in_reply_to: msg_id,
                        },
                    };
                    outbound_msg_tx.send(reply).unwrap();
                    return Ok(());
                }

                self.messages.insert(message);

                self.neighbors.iter().for_each(|neighbour| {
                    if neighbour == &msg.src {
                        return;
                    }
                    let body: Message = Message::Broadcast {
                        msg_id: None,
                        message,
                    };
                    let envelope: Envelope<Message> = Envelope {
                        src: self.node_id.clone(),
                        dest: neighbour.clone(),
                        body,
                    };
                    outbound_msg_tx.send(envelope).unwrap();
                });

                let reply = Envelope {
                    src: self.node_id.clone(),
                    dest: msg.src.clone(),
                    body: Message::BroadcastOk {
                        in_reply_to: msg_id,
                    },
                };
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            Message::BroadcastOk { in_reply_to: _ } => Ok(()),
            Message::Read { msg_id } => {
                let reply = Envelope {
                    src: self.node_id.clone(),
                    dest: msg.src.clone(),
                    body: Message::ReadOk {
                        in_reply_to: msg_id,
                        messages: self.messages.clone(),
                    },
                };
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            _ => Err(broadcast::Error::UnexpectedMessageType(format!(
                "{:#?}",
                msg
            ))),
        }
    }
}

pub fn main() {
    run(Broadcast::default()).unwrap();
}

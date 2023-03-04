use broadcast::Message;
use maelstrom_common::{run, HandleMessage, Envelope};
use std::{collections::HashSet, sync::mpsc::Sender};


#[derive(Debug, Default)]
pub struct Broadcast {
    pub node_id: Option<String>,
    pub neighbors: HashSet<String>,

    // The order is not important.
    pub messages: HashSet<usize>,
}

impl HandleMessage for Broadcast {
    type Message = Message;
    type Error = broadcast::Error;

    fn handle_message(
        &mut self,
        msg: Envelope<Self::Message>,
        outbound_msg_tx: Sender<Envelope<Self::Message>>
    ) -> Result<(), Self::Error> {
        match msg.body {
            Message::Init {
                msg_id,
                ref node_id,
                node_ids,
            } => {
                self.node_id = Some(node_id.clone());
                self.neighbors = node_ids;

                let payload = Message::InitOk { in_reply_to: msg_id };
                let reply = Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body: payload,
                };
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            Message::Topology {
                msg_id,
                topology,
            } => {
                let node_id = self.node_id.clone().unwrap();
                self.neighbors = topology
                    .get(&node_id)
                    .expect("to find a set of neighbors for us.")
                    .clone();

                let payload = Message::TopologyOk { in_reply_to: msg_id };
                let reply = Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body: payload,
                };
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            Message::Broadcast { msg_id, message } => {
                self.messages.insert(message);
                let reply = msg.reply(Message::BroadcastOk { in_reply_to: msg_id });
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            },
            Message::Read { msg_id } => {
                let reply = msg.reply(Message::ReadOk {
                    in_reply_to: msg_id,
                    messages: self.messages.clone(),
                });
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            },
            _ => {
                Err(broadcast::Error::UnexpectedMessageType(format!("{:#?}", msg.body)))
            }
        }
    }
}

pub fn main() {
    run(Broadcast::default()).unwrap();
}

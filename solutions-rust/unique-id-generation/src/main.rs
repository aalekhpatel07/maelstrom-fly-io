use maelstrom_common::{formatted_log, run, Actor, Envelope, Message};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "generate")]
    Generate { msg_id: usize },
    #[serde(rename = "init")]
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "generate_ok")]
    GenerateOk { id: String, in_reply_to: usize },
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: usize },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UniqueUuid {
    pub counter: AtomicUsize,
    pub node_id: Option<String>,
}

impl Actor for UniqueUuid {
    type InboundMessage = Request;
    type OutboundMessage = Response;
    fn handle_message(
        &mut self,
        msg: Envelope<Message<Self::InboundMessage, Self::OutboundMessage>>,
    ) -> Option<Envelope<Message<Self::InboundMessage, Self::OutboundMessage>>> {
        Some(match msg.body {
            Message::Inbound(Request::Init {
                msg_id, ref node_id, ..
            }) => {
                self.node_id = Some(node_id.clone());
                formatted_log!("INIT", "Initialized node with id: {}", node_id);

                msg.reply(Message::Outbound(Response::InitOk { in_reply_to: msg_id }))
            }
            Message::Inbound(Request::Generate { msg_id }) => {
                let counter = self
                    .counter
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                // Just namespace the ids with the node id and Bob's your uncle.
                let id = format!("{}-{}", self.node_id.as_ref().unwrap(), counter);

                formatted_log!("GENERATE", "Generated id: {}", id);
                
                msg.reply(
                    Message::Outbound(Response::GenerateOk {
                        id,
                        in_reply_to: msg_id,
                    })
                )
            },
            _ => {
                panic!("Unexpected message: {:?}", msg)
            }
        })
    }
}

pub fn main() {
    run(UniqueUuid::default()).unwrap();
}

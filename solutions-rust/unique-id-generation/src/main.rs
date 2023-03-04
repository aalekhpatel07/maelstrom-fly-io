use maelstrom_common::{run, Actor, Envelope};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicUsize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("The generate challenge does not expect to receive any response kind of message but received: {0}")]
    ReceivedUnexpectedResponseMessage(String),
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "generate")]
    Generate { msg_id: Option<usize> },
    #[serde(rename = "init")]
    Init {
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<usize>,
        node_id: String,
    },
    #[serde(rename = "generate_ok")]
    GenerateOk { 
        id: String, 
        #[serde(skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<usize>
    },
    #[serde(rename = "init_ok")]
    InitOk {
        #[serde(skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<usize>,
    },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UniqueUuid {
    pub counter: AtomicUsize,
    pub node_id: Option<String>,
}

impl Actor for UniqueUuid {
    type Message = Message;
    type Error = Error;

    fn handle_message(
        &mut self,
        msg: Envelope<Self::Message>,
        outbound_msg_tx: std::sync::mpsc::Sender<Envelope<Self::Message>>,
    ) -> Result<(), Self::Error> {
        match msg.body {
            Message::Init {
                msg_id, ref node_id, ..
            } => {
                self.node_id = Some(node_id.clone());
                let reply = msg.reply(Message::InitOk { in_reply_to: msg_id });
                outbound_msg_tx.send(reply).unwrap();
                Ok(())
            }
            Message::Generate { msg_id } => {
                let counter = self
                    .counter
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                // Just namespace the ids with the node id and Bob's your uncle.
                let id = format!("{}-{}", self.node_id.as_ref().unwrap(), counter);
                let reply = msg.reply(Message::GenerateOk {
                    id,
                    in_reply_to: msg_id,
                });
                outbound_msg_tx.send(reply).unwrap();

                Ok(())
            },
            _ => Err(Error::ReceivedUnexpectedResponseMessage(serde_json::to_string_pretty(&msg).unwrap()))
        }
    }
}

pub fn main() {
    run(UniqueUuid::default()).unwrap();
}

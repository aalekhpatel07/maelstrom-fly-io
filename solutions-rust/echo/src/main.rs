use maelstrom_common::{run, Actor, Envelope};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "init")]
    Init { msg_id: usize, node_id: String },
    #[serde(rename = "echo")]
    Echo { echo: String, msg_id: usize },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: usize },
    #[serde(rename = "echo_ok")]
    EchoOk { echo: String, in_reply_to: usize },
}

#[derive(Debug, Default)]
pub struct Echo {
    // Store our ID when a client initializes us.
    node_id: Option<String>,
}

impl Actor for Echo {
    type InboundMessage = Request;
    type OutboundMessage = Response;

    fn handle_message(
        &mut self,
        msg: Envelope<Self::InboundMessage>,
    ) -> Option<Envelope<Self::OutboundMessage>> {
        Some(match msg.body {
            Request::Init { msg_id, ref node_id } => {
                self.node_id = Some(node_id.clone());
                eprintln!("[INIT] Initialized node: {node_id}");
                
                msg.reply(Response::InitOk { in_reply_to: msg_id })
            }
            Request::Echo { ref echo, msg_id } => {
                eprintln!("[ECHO] Echoing back: {echo}, in reply to: {msg_id}");

                msg.reply(Response::EchoOk { echo: echo.to_string(), in_reply_to: msg_id })
            }
        })
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Echo::default())?;
    Ok(())
}

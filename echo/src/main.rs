use serde::{Deserialize, Serialize};
use maelstrom_common::{Actor, Maelstrom, Message};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Request {
    pub r#type: String,
    pub msg_id: usize,
    pub echo: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub r#type: String,
    pub msg_id: usize,
    pub echo: String,
    pub in_reply_to: usize
}


#[derive(Debug, Clone)]
pub struct Echo;

impl Echo {
    pub fn new() -> Self {
        Echo
    }
}


impl Actor for Echo {
    type InboundMessage = Request;
    type OutboundMessage = Response;
    fn handle_message(&mut self, msg: Message<Self::InboundMessage>) -> Option<Message<Self::OutboundMessage>> {
        let Request { msg_id, echo, .. } = msg.body;
        let response = Response {
            r#type: "echo_ok".into(),
            msg_id,
            echo,
            in_reply_to: msg_id
        };
        Some(Message::new(msg.dest, msg.src, response))
    }
}


fn main() {
    let echo_actor = Maelstrom::new(Echo::new());
    echo_actor.start().unwrap();
}

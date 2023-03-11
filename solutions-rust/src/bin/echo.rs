use std::{sync::mpsc::{channel, Receiver}, thread::spawn};

use maelstrom::*;
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Init {
        node_id: String,
        node_ids: Vec<String>
    },
    InitOk,
    Echo {
        echo: String
    },
    EchoOk {
        echo: String
    }
}


pub fn handle_message(rx: Receiver<Envelope<Message>>) {
    for msg in rx {
        match msg.message() {
            Message::Echo { echo } => {
                msg.reply(Message::EchoOk { echo: echo.clone() }).send();
            },
            Message::Init { .. } => {
                msg.reply(Message::InitOk).send();
            },
            _ => {}
        }
    }
}

pub fn main() {

    let (tx, rx) = channel::<Envelope<Message>>();

    spawn(move || {
        handle_message(rx)
    });

    read_stdin(tx);
}

use std::{sync::mpsc::channel, thread::spawn, collections::HashMap};

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
    Topology {
        topology: HashMap<String, Vec<String>>
    },
    TopologyOk,
    Broadcast {
        message: usize
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>
    }
}


pub fn main() {

    let (tx, rx) = channel::<Envelope<Message>>();

    spawn(move || {
        read_stdin(tx);
    });

    let mut messages: Vec<usize> = vec![];

    while let Ok(envelope) = rx.recv() {
        match envelope.message() {
            Message::Init { .. } => {
                envelope.reply(Message::InitOk).send();
            },
            Message::Topology { .. } => {
                envelope.reply(Message::TopologyOk).send();
            },
            Message::Broadcast { message } => {
                messages.push(*message);
                envelope.reply(Message::BroadcastOk).send();
            },
            Message::Read => {
                envelope.reply(Message::ReadOk { messages: messages.to_vec() }).send();
            },
            _ => {}
        }
    }

}

use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::{channel, Receiver},
    },
    thread::spawn,
};

use maelstrom::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Generate,
    GenerateOk {
        id: String,
    },
}

pub fn handle_message(rx: Receiver<Envelope<Message>>) {
    let mut our_id = None;
    let id = AtomicUsize::new(0);

    for msg in rx {
        match msg.message() {
            Message::Init { node_id, .. } => {
                our_id = Some(node_id.clone());
                msg.reply(Message::InitOk).send();
            }

            Message::Generate => {
                let id = format!(
                    "{}-{}",
                    our_id.as_ref().unwrap(),
                    id.fetch_add(1, Ordering::SeqCst)
                );
                msg.reply(Message::GenerateOk { id }).send();
            }
            _ => {}
        }
    }
}

pub fn main() {
    let (tx, rx) = channel::<Envelope<Message>>();

    spawn(move || handle_message(rx));

    read_stdin(tx);
}

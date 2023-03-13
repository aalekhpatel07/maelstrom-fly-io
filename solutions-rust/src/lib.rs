use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::io;
use std::sync::mpsc::Sender;

mod envelope;
pub use envelope::*;

pub fn read_stdin<B: Debug + DeserializeOwned>(incoming_messages_tx: Sender<Envelope<B>>) {
    for line in io::stdin().lines().map(Result::unwrap) {
        let decoded = serde_json::from_str(&line).unwrap();
        incoming_messages_tx.send(decoded).unwrap();
    }
}

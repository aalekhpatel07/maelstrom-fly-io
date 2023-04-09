use std::{
    sync::mpsc::{channel, Receiver},
    thread::spawn, time::Duration, collections::HashMap,
};

use maelstrom::*;
use serde::{Deserialize, Serialize};



#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Topology {
        topology: HashMap<String, Vec<String>>
    },
    TopologyOk,
    Add {
        delta: usize
    },
    AddOk,
    Read {
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>
    },
    ReadOk {
        value: usize
    },
    Write {
        key: String,
        value: usize
    },
    WriteOk,
    Cas {
        key: String,
        from: usize,
        to: usize,
        #[serde(skip_serializing_if = "Option::is_none")]
        create_if_not_exists: Option<bool>
    },
    CasOk,
    Error {
        code: usize,
        text: String
    }
}

pub fn is_a_kv_store_envelope(envelope: &Envelope<Message>) -> bool {
    if envelope.src == "seq-kv" || envelope.dest == "seq-kv" {
        return true;
    }
    match envelope.message() {
        Message::Read { key } => {
            key.is_some()
        },
        _ => false
    }
}

fn message_for_kv_store(src: &str, message: Message) -> Envelope<Message> {
    Envelope::new(src, "seq-kv", None, message)
}


pub fn handle_message(rx: Receiver<Envelope<Message>>) {

    const WAIT_DURATION: Duration = Duration::from_millis(500);

    let mut our_node_id = Default::default();
    let mut all_nodes = Default::default();

    // Guaranteed up-to-date value in the kv store that was seen at
    // one point in the past.
    let mut our_value: usize = 0;

    let mut last_cas_flushed: usize = 0;

    // Buffer all adds, and as soon as we get a chance
    // to talk to the seq-kv store, flush it all down.
    let mut pending_add: usize = 0;

    // The idea is to store the cas-ok'ed total to the kv-store,
    // and any pending updates can be cached.
    let mut cas_pending: bool = false;

    // Cas id's should be monotonically increasing.
    let mut last_cas: usize = 0;

    loop {
        match rx.recv_timeout(WAIT_DURATION) {
            Ok(envelope) => {
                match envelope.message() {
                    Message::Init { node_id, node_ids } => {
                        our_node_id = node_id.clone();
                        all_nodes = node_ids.clone();
                        envelope.reply(Message::InitOk).send();

                        // Initialize the kv store to 0.
                        let cas_envelope = message_for_kv_store(
                            &our_node_id, 
                            Message::Cas { 
                                key: "total".to_string(), 
                                from: 0, 
                                to: 0, 
                                create_if_not_exists: Some(true)
                            }
                        );

                        last_cas = cas_envelope.msg_id().unwrap();

                        cas_envelope.send();
                        // We issued a cas, so we don't sync our guaranteed
                        // fresh state in the seq-kv store repeatedly, until we hear back
                        // from it first.
                        cas_pending = true;

                    },
                    Message::Topology {  .. } => {
                        envelope.reply(Message::TopologyOk).send();
                    },
                    // Our reads can be stale, np.
                    Message::Read { .. } => {
                        envelope.reply(Message::ReadOk { value: our_value }).send();
                    },
                    Message::Add { delta } => {
                        pending_add += delta;
                        envelope.reply(Message::AddOk).send();
                    },
                    Message::CasOk => {
                        // If our cas was acknowledged,
                        // it means no other neighbor bumped
                        // the state while we were trying to flush our
                        // pending updates. This is a happy path.
                        if envelope.in_reply_to() == Some(last_cas) {
                            pending_add = 0; // we flushed our pending adds with the last cas.
                            our_value = last_cas_flushed; // our value is effectively the same as stored in the seq-kv store.
                            last_cas = 0; // Just a defunct state.
                            cas_pending = false;
                        }
                    },
                    Message::ReadOk { value } => {
                        if envelope.is_internal() {
                            // Our neighbors acknowledged our read.
                            // In case any of our neighbors have a higher value,
                            // that value must be the most recent one.
                            our_value = our_value.max(*value);
                        } else {
                            // the kv store got back to us with its up-to-date value.
                            our_value = *value;
                        }
                    },
                    Message::Error { .. } => {
                        // We only get errors from the seq-kv store,
                        // and those errors could only signal a failed cas.
                        cas_pending = false;
                        // In case of a failure, ask the store for its latest total, and update ourselves.
                        message_for_kv_store(&our_node_id, Message::Read { key: Some("total".to_owned()) }).send();
                    },
                    _ => {}
                }
            },

            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // We were on one side of a partition (or the messages were just too slow),
                // so, if we don't have any pending state updates, ask our neighbors
                // for any fresh values they might have, and try to get ourselves
                // up-to-date with the most recent value from a neighbor.
                if pending_add == 0 {
                    all_nodes
                    .iter()
                    .filter(|&node_id| node_id != &our_node_id)
                    .for_each(|node| {
                        Envelope::new(
                            &our_node_id, 
                            &node, 
                            None, 
                            Message::Read { key: None } // Send to neighbors, not seq-kv.
                        ).send();
                    });
                }
            },

            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {}
        }

        // If we have pending updates, and a cas is not currently in-flight,
        // try to flush all the updates to the store.
        if pending_add != 0 && !cas_pending {
            last_cas_flushed = our_value + pending_add;
            let env = message_for_kv_store(
                &our_node_id, 
                Message::Cas { 
                    key: "total".into(), 
                    from: our_value, 
                    to: last_cas_flushed, 
                    create_if_not_exists: None
                }
            );
            last_cas = env.msg_id().unwrap();
            cas_pending = true;
            env.send();
        }

    }
}

pub fn main() {

    let (tx_stdin, rx_stdin) = channel();
    spawn(move || handle_message(rx_stdin));

    for line in std::io::stdin().lines().map(Result::unwrap) {
        tx_stdin.send(serde_json::from_str(&line).unwrap()).unwrap();
    }

}

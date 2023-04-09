use std::env;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{collections::HashMap, thread::spawn};

use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use serde::{Serialize, Deserialize};
use maelstrom::{Body, Envelope, read_stdin};
use std::sync::atomic::{AtomicUsize, Ordering};


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Message {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Send {
        key: String,
        msg: usize
    },
    SendOk {
        offset: usize
    },
    Poll {
        offsets: HashMap<String, usize>
    },
    PollOk {
        msgs: HashMap<String, Vec<[usize; 2]>>
    },
    CommitOffsets {
        offsets: HashMap<String, usize>
    },
    CommitOffsetsOk,
    ListCommittedOffsets {
        keys: Vec<String>
    },
    ListCommittedOffsetsOk {
        offsets: HashMap<String, usize>
    }
}


#[derive(Debug, Default)]
pub struct Log {
    storage: Vec<usize>,
    committed_offset: usize
}

impl Log {
    pub fn new(msgs: &[usize]) -> Self {
        Self {
            storage: msgs.to_vec(),
            committed_offset: msgs.len()
        }
    }

    pub fn append(&mut self, msg: usize) -> usize {
        self.storage.push(msg);
        self.storage.len()
    }

    pub fn commit_offset(&mut self, offset: usize) {
        self.committed_offset = self.committed_offset.max(offset);
    }

}


#[derive(Debug, Default)]
pub struct NodeState {
    pub id: String,
    pub neighbors: Vec<String>,
    pub all_nodes: Vec<String>
}

impl NodeState {

    pub fn save_topology(&mut self, topology: &HashMap<String, Vec<String>>) {
        self.neighbors = topology.get(&self.id).unwrap().clone();
    }
    pub fn save_nodes(&mut self, nodes: &[String]) {
        self.all_nodes = nodes.to_vec();
    }

    pub fn set_id(&mut self, id: &String) {
        self.id = id.to_owned();
    }
}


#[derive(Debug, Default)]
pub struct LogState {
    pub logs: HashMap<String, Log>
}

impl LogState {
    pub fn get_messages_from_offset(&self, key: &str, offset: usize) -> Option<Vec<[usize; 2]>> {
        self
        .logs
        .get(key)
        .map(|log| {
            log
            .storage[offset..]
            .iter()
            .enumerate()
            .map(|(idx, msg)| {
                [offset + idx, *msg]
            })
            .collect()
        })
    }

    pub fn commit_offset(&mut self, key: &str, offset: usize) -> Option<()> {
        self.logs.get_mut(key).map(|log| log.commit_offset(offset))
    }

    pub fn get_committed_offset(&self, key: &str) -> Option<usize> {
        self.logs.get(key).map(|log| log.committed_offset)
    }
}


pub fn handle_message(rx: Receiver<Envelope<Message>>) {

    let mut node_state = NodeState::default();
    let mut state: LogState = Default::default();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(envelope) => {
                match envelope.message() {
                    Message::Init { node_id, node_ids } => {
                        node_state.set_id(node_id);
                        node_state.save_nodes(node_ids);
                        envelope.reply(Message::InitOk).send();
                    },
                    Message::Topology { topology } => {
                        node_state.save_topology(topology);
                        envelope.reply(Message::TopologyOk).send();
                    },
                    Message::Send { key, msg } => {

                        let mut offset = 1;

                        state
                        .logs
                        .entry(key.clone())
                        .and_modify(|log| {
                            offset = log.append(*msg);
                        })
                        .or_insert(Log::new(&[*msg]));

                        envelope.reply(Message::SendOk { offset }).send();
                    },
                    Message::Poll { offsets } => {
                        let poll_ok_body: HashMap<String, Vec<[usize; 2]>> = 
                            offsets
                            .into_iter()
                            .filter_map(|(key, offset)| {
                                state
                                .get_messages_from_offset(key, *offset)
                                .map(|ofsts| {
                                    (key.clone(), ofsts)
                                })
                            })
                            .collect();
                        envelope.reply(Message::PollOk { msgs: poll_ok_body }).send();
                    },
                    Message::CommitOffsets { offsets } => {
                        offsets
                        .into_iter()
                        .for_each(|(key, offset)| {
                            state.commit_offset(key, *offset);
                        });
                        envelope.reply(Message::CommitOffsetsOk).send();
                    },
                    Message::ListCommittedOffsets { keys } => {
                        
                        let list_committed_offsets_ok_body: HashMap<String, usize> =
                        keys
                        .into_iter()
                        .filter_map(|k| {
                            state
                            .get_committed_offset(k)
                            .map(|offset| (k.clone(), offset))
                        })
                        .collect();

                        envelope.reply(Message::ListCommittedOffsetsOk { offsets: list_committed_offsets_ok_body }).send();
                    },
                    _ => {}
                }
            },

            Err(err) => {
            },


        }
    }
}


pub fn main() {

    let (tx, rx) = channel();

    spawn(move || {
        handle_message(rx);       
    });

    read_stdin(tx);
}
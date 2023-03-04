use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "init")]
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: HashSet<String>,
    },
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: usize },
    #[serde(rename = "topology")]
    Topology {
        msg_id: usize,
        topology: HashMap<String, HashSet<String>>,
    },

    #[serde(rename = "topology_ok")]
    TopologyOk { in_reply_to: usize },

    #[serde(rename = "broadcast")]
    Broadcast {
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<usize>,
        message: usize,
    },

    #[serde(rename = "broadcast_ok")]
    BroadcastOk {
        #[serde(skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<usize>,
    },

    #[serde(rename = "read")]
    Read { msg_id: usize },

    #[serde(rename = "read_ok")]
    ReadOk {
        in_reply_to: usize,
        messages: HashSet<usize>,
    },
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unexpected message type: {0:#?}")]
    UnexpectedMessageType(String),
}

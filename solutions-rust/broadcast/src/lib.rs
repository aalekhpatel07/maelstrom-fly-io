use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "init")]
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    },
    #[serde(rename = "broadcast")]
    Broadcast {
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<usize>,
        message: usize,
    },
    #[serde(rename = "read")]
    Read { msg_id: usize },
    #[serde(rename = "topology")]
    Topology {
        msg_id: usize,
        topology: HashMap<String, Vec<String>>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: usize },
    #[serde(rename = "broadcast_ok")]
    BroadcastOk {
        #[serde(skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<usize>,
    },
    #[serde(rename = "read_ok")]
    Read {
        in_reply_to: usize,
        messages: Vec<usize>,
    },
    #[serde(rename = "topology_ok")]
    Topology { in_reply_to: usize },
}

use serde::{Deserialize, Serialize};
use std::collections::{
    HashMap, 
    HashSet
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all="snake_case")]
pub enum Message {
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: HashSet<String>,
    },
    InitOk { in_reply_to: usize },
    Topology {
        msg_id: usize,
        topology: HashMap<String, HashSet<String>>,
    },
    TopologyOk { in_reply_to: usize },
    Broadcast {
        msg_id: Option<usize>,
        message: usize,
    },
    BroadcastOk {
        in_reply_to: Option<usize>,
    },
    Read { msg_id: usize },
    ReadOk {
        in_reply_to: usize,
        messages: HashSet<usize>,
    },
}

impl Message {
    pub fn is_from_server_node(&self) -> bool {
        match self {
            Message::Broadcast { msg_id, .. } => msg_id.is_none(),
            Message::BroadcastOk { in_reply_to, .. } => in_reply_to.is_none(),
            _ => false
        }
    }
}
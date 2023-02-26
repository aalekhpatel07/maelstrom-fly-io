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
    Broadcast { msg_id: usize, message: usize },
    #[serde(rename = "read")]
    Read { msg_id: usize },
    #[serde(rename = "topology")]
    Topology {
        msg_id: usize,
        topology: HashMap<String, Vec<String>>,
    },
    #[serde(rename = "internal")]
    Internal { request: InternalRequest },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: usize },
    #[serde(rename = "broadcast_ok")]
    BroadcastOk { in_reply_to: usize },
    #[serde(rename = "read_ok")]
    Read {
        in_reply_to: usize,
        messages: Vec<usize>,
    },
    #[serde(rename = "topology_ok")]
    Topology { in_reply_to: usize },

    #[serde(rename = "internal")]
    Internal { response: InternalResponse },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InternalRequest {
    /// This is what we wish to convey to our peers if they're at all reachable.
    /// "Hey node! Can you please add this value to your set of messages?"
    /// "Oh and also, if possible, could you please forward this message to the given nodes that I couldn't reach myself?"
    #[serde(rename = "add")]
    Add {
        value: usize,
        remaining_nodes: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InternalResponse {
    /// This is what our peers will respond with if they're good friends with us.
    /// "Oh hey fellow node. I've added the value to my set of messages and forwarded the message to the given nodes."
    #[serde(rename = "add_ok")]
    AddOk { forwarded_to: Vec<String> },
}

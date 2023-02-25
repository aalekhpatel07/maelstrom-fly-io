use serde::{Serialize, Deserialize};
use std::sync::atomic::AtomicUsize;
use maelstrom_common::{
    Actor,
    Envelope,
    formatted_log,
    run
};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "generate")]
    Generate {
        msg_id: usize,
    },
    #[serde(rename = "init")]
    Init {
        msg_id: usize,
        node_id: String,
        node_ids: Vec<String>,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "generate_ok")]
    GenerateOk {
        id: String,
        in_reply_to: usize,
    },
    #[serde(rename = "init_ok")]
    InitOk {
        in_reply_to: usize,
    },
}


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UniqueUuid {
    pub counter: AtomicUsize,
    pub node_id: Option<String>
}


impl Actor for UniqueUuid {
    type InboundMessage = Request;
    type OutboundMessage = Response;
    fn handle_message(
        &mut self,
        msg: Envelope<Self::InboundMessage>
    ) -> Option<Envelope<Self::OutboundMessage>> {
        Some(match msg.body {
            Request::Init { msg_id, node_id, .. } => {
                self.node_id = Some(node_id.clone());
                formatted_log!("INIT", "Initialized node with id: {}", node_id);

                Envelope {
                    body: Response::InitOk {
                        in_reply_to: msg_id
                    },
                    src: msg.dest,
                    dest: msg.src
                }
            },
            Request::Generate { msg_id } => {
                let counter = self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                
                // Just namespace the ids with the node id and Bob's your uncle.
                let id = format!("{}-{}", self.node_id.as_ref().unwrap(), counter);

                formatted_log!("GENERATE", "Generated id: {}", id);

                Envelope {
                    body: Response::GenerateOk {
                        id,
                        in_reply_to: msg_id
                    },
                    src: msg.dest,
                    dest: msg.src
                }
            }
        })
    }
}


pub fn main() {
    run(UniqueUuid::default()).unwrap();
}

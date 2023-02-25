use serde::{Deserialize, Serialize};
use maelstrom_common::{
    Actor, 
    Maelstrom, 
    Message,
    formatted_log,
};
use serde_json::Value;
use std::sync::atomic::AtomicUsize;

macro_rules! init_log {
    ($($msg: expr),*) => {
        formatted_log!("INIT", $($msg),*);
    };
}

macro_rules! echo_log {
    ($($msg: expr),*) => {
        formatted_log!("ECHO", $($msg),*);
    };
}

macro_rules! state_log {
    ($($msg: expr),*) => {
        formatted_log!("STATE", $($msg),*);
    };
}

#[derive(Debug, Default)]
pub struct Echo {
    // Store a counter for all messages processed.
    counter: AtomicUsize,
    // Store our ID when a client initializes us.
    node_id: Option<String>
}


impl Echo {
    pub fn new() -> Self {
        Self::default()
    }
}


impl Actor for Echo {
    type InboundMessage = Value;
    type OutboundMessage = Value;
    fn handle_message(&mut self, msg: Message<Self::InboundMessage>) -> Option<Message<Self::OutboundMessage>> {
        let payload = msg.body;
        
        self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let msg_id = payload.get("msg_id").unwrap().as_u64().unwrap();
        let r#type = payload.get("type").unwrap().as_str().unwrap();

        // It sucks that these messages are so loosely typed. :sadcat:
        let augment_with = match r#type {
            "init" => {
                let node_id = payload.get("node_id").unwrap().as_str().unwrap();
                self.node_id = Some(node_id.to_string());
                init_log!("Initialized node: {}", node_id);

                serde_json::json!({
                    "type": "init_ok",
                    "msg_id": self.counter.load(
                        std::sync::atomic::Ordering::Relaxed
                    ),
                    "in_reply_to": msg_id
                })
            },
            "echo" => {
                echo_log!("Echoing back: {}", payload.get("echo").unwrap().as_str().unwrap());
                serde_json::json!({
                    "type": "echo_ok",
                    "msg_id": self.counter.load(
                        std::sync::atomic::Ordering::Relaxed
                    ),
                    "in_reply_to": msg_id
                })
            },
            _ => {
                panic!("Unknown message type: {}", r#type);
            }
        };

        let mut payload = payload.clone();
        let payload_as_object = payload.as_object_mut().unwrap();

        for (k, v) in augment_with.as_object().unwrap() {
            payload_as_object.insert(k.clone(), v.clone());
        }

        state_log!("{:#?}", self);

        Some(Message::new(msg.dest, msg.src, payload))
    }
}


pub fn main() {
    Maelstrom::new(Echo::new()).start().unwrap();
}

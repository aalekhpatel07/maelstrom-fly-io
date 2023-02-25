use maelstrom_common::{
    Actor, 
    Envelope,
    formatted_log, 
    merge,
    run
};
use serde_json::Value;

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
    // Store our ID when a client initializes us.
    node_id: Option<String>
}


impl Actor for Echo {
    type InboundMessage = Value;
    type OutboundMessage = Value;
    
    fn handle_message(&mut self, msg: Envelope<Self::InboundMessage>) -> Option<Envelope<Self::OutboundMessage>> {
        let mut payload = msg.body;
        
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
                    "in_reply_to": msg_id
                })
            },
            "echo" => {
                echo_log!("Echoing back: {}", payload.get("echo").unwrap().as_str().unwrap());
                serde_json::json!({
                    "type": "echo_ok",
                    "in_reply_to": msg_id
                })
            },
            _ => {
                panic!("Unknown message type: {}", r#type);
            }
        };
        state_log!("{:#?}", self);
        merge(&mut payload, &augment_with);
        Some(Envelope::new(msg.dest, msg.src, payload))
    }
}


pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Echo::default())?;
    Ok(())
}

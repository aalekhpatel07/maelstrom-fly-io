//! # Maelstrom
//! 

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io;
use std::sync::mpsc::channel;
use std::thread::spawn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message<B> {
    pub src: String,
    pub dest: String,
    pub body: B
}

impl<B> Message<B> 
where
    B: Clone + Serialize + DeserializeOwned
{
    pub fn new(src: String, dest: String, body: B) -> Self {
        Message {
            src,
            dest,
            body
        }
    }
}

#[macro_export]
macro_rules! formatted_log {
    ($prefix: expr, $($msg: expr),*) => {
        eprintln!("{}: {}", format!("{}", $prefix), format!($($msg),*));
    };
}

#[macro_export]
macro_rules! in_log {
    ($($msg: expr),*) => {
        formatted_log!("IN", $($msg),*);
    };
}

#[macro_export]
macro_rules! out_log {
    ($($msg: expr),*) => {
        formatted_log!("OUT", $($msg),*);
    };
}

#[macro_export]
macro_rules! proc_log {
    ($($msg: expr),*) => {
        formatted_log!("COMPUTE", $($msg),*);
    };
}


pub trait Actor {
    type InboundMessage: Clone + Serialize + DeserializeOwned + core::fmt::Debug + Send + Sync + 'static;
    type OutboundMessage: Clone + Serialize + DeserializeOwned + core::fmt::Debug + Send + Sync + 'static;
    fn handle_message(&mut self, msg: Message<Self::InboundMessage>) -> Option<Message<Self::OutboundMessage>>;
}

#[derive(Debug)]
pub struct Maelstrom<A> 
{
    pub actor: A
}

impl<A> Maelstrom<A> 
{
    pub fn new(actor: A) -> Self {
        Maelstrom {
            actor
        }
    }
}


impl<A> Maelstrom<A>
where
    A: Actor,
{
    pub fn start(mut self) -> Result<(), Box<dyn std::error::Error>>{

        let (inbound_msg_tx, inbound_msg_rx) = channel::<Message<A::InboundMessage>>();
        let (outbound_msg_tx, outbound_msg_rx) = channel::<Message<A::OutboundMessage>>();

        spawn(move || {
            let mut buffer = String::new();
            loop {
                io::stdin().read_line(&mut buffer).expect("Failed to read stdin.");
                in_log!("Just read: {}", buffer);
                let Ok(msg) = serde_json::from_str(&buffer) else {
                    in_log!("Failed to deserialize: {}", buffer);
                    continue;
                };
                inbound_msg_tx.send(msg).unwrap();
                in_log!("Sent for processing.");
                buffer.clear();
            }
        });

        spawn(move || {
            loop {
                let msg: _ = outbound_msg_rx.recv().unwrap();
                let msg_str = serde_json::to_string(&msg).unwrap();
                out_log!("Will write: {}", &msg_str);
                println!("{}", msg_str);
                out_log!("Written successfully");
            }
        });

        loop {
            let msg: _ = inbound_msg_rx.recv().unwrap();
            proc_log!("Processing: {:#?}", &msg);
            let response = self.actor.handle_message(msg);
            if let Some(response) = response {
                proc_log!("Processed successfully");
                outbound_msg_tx.send(response).unwrap();
            }
            else {
                proc_log!("Processed but returned None.");
            }
        }
    }
}
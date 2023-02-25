//! # Maelstrom
//! 

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::{self, Write};
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
                let msg: Message<A::InboundMessage> = serde_json::from_str(&buffer).unwrap();
                inbound_msg_tx.send(msg).unwrap();
                buffer.clear();
            }
        });

        spawn(move || {
            loop {
                let msg: _ = outbound_msg_rx.recv().unwrap();
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                handle.write_all(
                    serde_json::to_string(&msg).unwrap().as_bytes()
                ).unwrap();
            }
        });

        loop {
            let msg: _ = inbound_msg_rx.recv().unwrap();
            let response = self.actor.handle_message(msg);
            if let Some(response) = response {
                outbound_msg_tx.send(response).unwrap();
            }
        }
    }
}
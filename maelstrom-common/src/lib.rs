//! [Maelstrom] is a workbench for testing toy implementations of distributed systems.
//!
//! This crate abstracts away the boilerplate of setting up the stdin/stdout for a node
//! in a distributed system, and provides a few useful utilities for writing handlers.
//!
//! This crate is inspired from and primarily written for the [Fly.io Distributed Systems] challenge.
//!
//! # Usage
//!
//! To use this crate, you'll need to implement the `Actor` trait. This trait has two associated types,
//! `InboundMessage` and `OutboundMessage`. These types are used to define the types of messages that
//! your actor can receive and send, respectively. These types must implement `Clone`, `Serialize`, `DeserializeOwned`,
//! `Debug`, `Send`, `Sync`, and be `'static`.
//!
//! ## Example
//!
//! ```no_run
//! use maelstrom_common::{Actor, Envelope};
//! use serde_json::Value;
//!
//! #[derive(Debug, Default)]
//! pub struct Echo {
//!    // Store our ID when a client initializes us.
//!   node_id: Option<String>
//! }
//!
//! #[derive(Debug, Serialize, Deserialize, Clone)]
//! #[serde(tag = "type")]
//! pub enum Request {
//!    #[serde(rename = "init")]
//!    Init {
//!        msg_id: usize,
//!        node_id: String,
//!    },
//!    #[serde(rename = "echo")]
//!    Echo {
//!        echo: String,
//!        msg_id: usize
//!    }
//!}
//!
//!#[derive(Debug, Serialize, Deserialize, Clone)]
//!#[serde(tag = "type")]
//!pub enum Response {
//!    #[serde(rename = "init_ok")]
//!    InitOk {
//!        in_reply_to: usize,
//!    },
//!    #[serde(rename = "echo_ok")]
//!    EchoOk {
//!        echo: String,
//!        in_reply_to: usize
//!    }
//!}
//!
//! impl Echo {
//!  pub fn new() -> Self {
//!     Self::default()
//!  }
//! }
//!
//! impl Actor for Echo {
//!    type InboundMessage = Request;
//!    type OutboundMessage = Response;
//!    
//!    fn handle_message(&mut self, msg: Envelope<Self::InboundMessage>) -> Option<Envelope<Self::OutboundMessage>> {
//!
//!        Some(match msg.body {
//!            Request::Init { msg_id, node_id } => {
//!                self.node_id = Some(node_id.clone());
//!                eprintln!("[INIT] Initialized node: {}", node_id);
//!                Envelope {
//!                    body: Response::InitOk { in_reply_to: msg_id },
//!                    src: msg.dest,
//!                    dest: msg.src
//!                }
//!            },
//!            Request::Echo { echo, msg_id } => {
//!                eprintln!("[ECHO] Echoing back: {}, in reply to: {}", echo, msg_id);
//!                Envelope {
//!                    src: msg.dest,
//!                    dest: msg.src,
//!                    body: Response::EchoOk { echo, in_reply_to: msg_id }
//!                }
//!            },
//!        })
//!    }
//! }
//! ```
//!
//!
//! [Maelstrom]: (https://github.com/jepsen-io/maelstrom)
//! [Fly.io Distributed Systems]: (https://fly.io/dist-sys/)
//!
//!

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{self};
use std::sync::mpsc::channel;
use std::thread::spawn;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Envelope<B> {
    pub src: String,
    pub dest: String,
    pub body: B,
}

impl<B> Envelope<B>
where
    B: Clone + Serialize + DeserializeOwned,
{
    pub fn new(src: String, dest: String, body: B) -> Self {
        Envelope { src, dest, body }
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
    type InboundMessage: Clone
        + Serialize
        + DeserializeOwned
        + core::fmt::Debug
        + Send
        + Sync
        + 'static;
    type OutboundMessage: Clone
        + Serialize
        + DeserializeOwned
        + core::fmt::Debug
        + Send
        + Sync
        + 'static;
    fn handle_message(
        &mut self,
        msg: Envelope<Self::InboundMessage>,
    ) -> Option<Envelope<Self::OutboundMessage>>;

    fn send_message(
        &self,
        msg: Envelope<Self::InboundMessage>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = serde_json::to_string(&msg).expect("to be able to serialize message");
        out_log!("[INTERNAL] Serialized outbound: {}", serialized);
        println!("{}", serialized);
        Ok(())
    }
}

#[derive(Debug)]
pub struct Maelstrom<A> {
    pub actor: A,
}

impl<A> Maelstrom<A> {
    pub fn new(actor: A) -> Self {
        Maelstrom { actor }
    }
}

/// Run the Maelstrom runtime implicitly.
pub fn run<A>(actor: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: Actor,
{
    Maelstrom::new(actor).start()
}

impl<A> Maelstrom<A>
where
    A: Actor,
{
    pub fn start(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (inbound_msg_tx, inbound_msg_rx) = channel::<Envelope<A::InboundMessage>>();
        let (outbound_msg_tx, outbound_msg_rx) = channel::<Envelope<A::OutboundMessage>>();

        spawn(move || {
            let mut buffer = String::new();
            loop {
                io::stdin()
                    .read_line(&mut buffer)
                    .expect("Failed to read stdin.");
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

        spawn(move || loop {
            let msg: _ = outbound_msg_rx.recv().unwrap();
            let msg_str = serde_json::to_string(&msg).unwrap();
            out_log!("Will write: {}", &msg_str);
            println!("{}", msg_str);
            out_log!("Written successfully");
        });

        loop {
            let msg: _ = inbound_msg_rx.recv().unwrap();
            proc_log!("Processing: {:#?}", &msg);
            let response = self.actor.handle_message(msg);
            if let Some(response) = response {
                proc_log!("Processed successfully");
                outbound_msg_tx.send(response).unwrap();
            } else {
                proc_log!("Processed but returned None.");
            }
        }
    }
}

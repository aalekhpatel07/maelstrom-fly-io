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
//! use maelstrom_common::{run, Actor, Envelope};
//! use serde::{Deserialize, Serialize};
//! use thiserror::Error;
//!
//! #[derive(Error, Debug)]
//! pub enum Error {
//!     #[error("The Echo challenge does not expect to receive any response kind of message but received: {0}")]
//!     ReceivedUnexpectedResponseMessage(String)
//! }
//!
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! #[serde(tag = "type")]
//! pub enum Message {
//!     #[serde(rename = "init")]
//!     Init {
//!         #[serde(skip_serializing_if = "Option::is_none")]
//!         msg_id: Option<usize>,
//!         node_id: String
//!     },
//!     #[serde(rename = "echo")]
//!     Echo {
//!         echo: String,
//!         #[serde(skip_serializing_if = "Option::is_none")]
//!         msg_id: Option<usize>
//!     },
//!     #[serde(rename = "init_ok")]
//!     InitOk {
//!         #[serde(skip_serializing_if = "Option::is_none")]
//!         in_reply_to: Option<usize>
//!     },
//!     #[serde(rename = "echo_ok")]
//!     EchoOk {
//!         echo: String,
//!         #[serde(skip_serializing_if = "Option::is_none")]
//!         in_reply_to: Option<usize>
//!     },
//! }
//!
//! #[derive(Debug, Default)]
//! pub struct Echo {
//!     // Store our ID when a client initializes us.
//!     node_id: Option<String>,
//! }
//!
//! impl Actor for Echo {
//!     type Message = Message;
//!     type Error = Error;
//!
//!     fn handle_message(
//!         &mut self,
//!         msg: Envelope<Self::Message>,
//!         outbound_msg_tx: std::sync::mpsc::Sender<Envelope<Self::Message>>,
//!     ) -> Result<(), Self::Error> {
//!         match msg.body {
//!             Message::Init { msg_id, ref node_id } => {
//!                 self.node_id = Some(node_id.clone());
//!                 outbound_msg_tx.send(
//!                     msg.reply(Message::InitOk { in_reply_to: msg_id })
//!                 ).unwrap();
//!                 Ok(())
//!             },
//!             Message::Echo { ref echo, msg_id } => {
//!                 outbound_msg_tx.send(
//!                     msg.reply(
//!                     Message::EchoOk { echo: echo.to_owned(), in_reply_to: msg_id }
//!                     )
//!                 ).unwrap();
//!                 Ok(())
//!             },
//!             _ => Err(Error::ReceivedUnexpectedResponseMessage(format!("{:#?}", serde_json::to_string_pretty(&msg))))
//!         }
//!     }
//! }
//!
//! pub fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     run(Echo::default())?;
//!     Ok(())
//! }
//! ```
//!
//! [Maelstrom]: (https://github.com/jepsen-io/maelstrom)
//! [Fly.io Distributed Systems]: (https://fly.io/dist-sys/)
//!
//!

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread::spawn;

#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope<B> {
    pub src: String,
    pub dest: String,
    pub body: B,
}

impl<B> Envelope<B>
where
    B: Serialize + DeserializeOwned,
{
    pub fn new(src: &str, dest: &str, body: B) -> Self {
        Envelope {
            src: src.to_owned(),
            dest: dest.to_owned(),
            body,
        }
    }
    pub fn reply<T>(&self, body: T) -> Envelope<T> {
        Envelope {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body,
        }
    }
}

pub trait Actor {
    type Message: Serialize + DeserializeOwned + Send + Sync + 'static;
    type Error: std::error::Error + 'static;

    fn handle_message(
        &mut self,
        msg: Envelope<Self::Message>,
        outbound_msg_tx: Sender<Envelope<Self::Message>>,
    ) -> Result<(), Self::Error>;
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
        let (inbound_msg_tx, inbound_msg_rx) = channel();
        let (outbound_msg_tx, outbound_msg_rx) = channel();

        spawn(move || {
            let mut buffer = String::new();
            loop {
                io::stdin()
                    .read_line(&mut buffer)
                    .expect("Failed to read stdin.");
                let msg = serde_json::from_str(&buffer).unwrap();
                inbound_msg_tx.send(msg).unwrap();
                buffer.clear();
            }
        });

        spawn(move || {
            let mut stdout = io::stdout().lock();
            loop {
                let msg: _ = outbound_msg_rx.recv().unwrap();
                let msg_str = serde_json::to_string(&msg).unwrap();
                writeln!(stdout, "{}", msg_str).unwrap();
            }
        });

        loop {
            let msg: _ = inbound_msg_rx.recv().unwrap();
            self.actor.handle_message(msg, outbound_msg_tx.clone())?;
        }
    }
}

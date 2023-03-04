//! [Maelstrom](https://github.com/jepsen-io/maelstrom) is a workbench for testing toy implementations of distributed systems.
//!
//! This crate abstracts away the boilerplate of setting up the stdin/stdout for a node
//! in a distributed system, and provides a few useful utilities for writing handlers.
//!
//! This crate is inspired from and primarily written for the [Fly.io Distributed Systems challenges](https://fly.io/dist-sys/).
//!
//! # Usage
//!
//! To use this crate, you'll create a node that is capable of handling
//! some rpcs. Define the rpc messages with a serializable `Message` enum 
//! and define any meaningful error type that can stop the maelstrom test early
//! in case of something going terribly wrong with the node.
//! 
//! The node must implement the [HandleMessage] trait, which requires
//! a `handle_message` function that takes an [Envelope] and a [Sender] for optionally
//! sending any messages.
//!
//! ## Example
//!
//! Let's create a simple echo node that responds to `init` and `echo` messages.
//! This also corresponds to the [Echo challenge](https://fly.io/dist-sys/1/) in the [Fly.io Distributed Systems challenge set](https://fly.io/dist-sys/).
//! 
//! ```no_run
//! use maelstrom_common::{run, HandleMessage, Envelope};
//! use serde::{Deserialize, Serialize};
//! use core::panic;
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
//! impl HandleMessage for Echo {
//!     type Message = Message;
//!     type Error = std::io::Error;
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
//!             _ => panic!("{}", format!("Unexpected message: {:#?}", serde_json::to_string_pretty(&msg)))
//!         }
//!     }
//! }
//!
//! # pub fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     run(Echo::default())?;
//! #    Ok(())
//! # }
//! ```

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread::spawn;

/// A formal structure for any message sent between nodes
/// or clients in a maelstrom orchestrated distributed system.
#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope<B> {
    /// An identifier for the sender of the message.
    pub src: String,
    /// An identifier for the receiver of the message.
    pub dest: String,
    /// The message payload. Typically, any inbound messages
    /// from a client will contain a `msg_id` field, and outbound
    /// messages will contain an `in_reply_to` field. This is not a
    /// hard requirement for inter-node communication, but it is required
    /// for inter-client communication. Also, payloads for messages to and
    /// from clients should contain a `type` field, which is used to
    /// determine the type of rpc being communicated.
    pub body: B,
}

impl<B> Envelope<B>
where
    B: Serialize + DeserializeOwned,
{
    /// Create a new envelope with the given source, destination, and body.
    pub fn new(src: &str, dest: &str, body: B) -> Self {
        Envelope {
            src: src.to_owned(),
            dest: dest.to_owned(),
            body,
        }
    }
    /// Create a reply to this envelope, with a given body.
    /// The source and destination of the envelope will be swapped
    /// as this is a reply.
    pub fn reply<T>(&self, body: T) -> Envelope<T> {
        Envelope {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body,
        }
    }
}

/// A single node should be able to handle messages
/// of a given type, and return an error if something goes wrong.
pub trait HandleMessage {
    /// The type of message this node can handle.
    /// Typically, this is a discriminated union of all
    /// rpc requests and response types that can be
    /// communicated by client-node or node-node.
    type Message: Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Any custom error to panic with and stop the maelstrom
    /// runtime early to indicate a problem with the node.
    type Error: std::error::Error + 'static;

    fn handle_message(
        &mut self,
        msg: Envelope<Self::Message>,
        outbound_msg_tx: Sender<Envelope<Self::Message>>,
    ) -> Result<(), Self::Error>;
}

/// A thin wrapper around a node that handles
/// rpcs and dumps outbound messages to stdout
/// and reads inbound messages from stdin.
#[derive(Debug)]
pub struct Maelstrom<A> {
    /// The node that handles messages.
    pub node: A,
}

impl<A> Maelstrom<A> {
    pub fn new(node: A) -> Self {
        Maelstrom { node }
    }
}

/// Run the Maelstrom runtime implicitly
/// for the given node that can handle rpcs.
pub fn run<A>(node: A) -> Result<(), Box<dyn std::error::Error>>
where
    A: HandleMessage,
{
    Maelstrom::new(node).start()
}

impl<A> Maelstrom<A>
where
    A: HandleMessage,
{
    /// Start the Maelstrom runtime by
    /// accepting messages from stdin, processing messages,
    /// and dumping any responses to stdout.
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
            self.node.handle_message(msg, outbound_msg_tx.clone())?;
        }
    }
}

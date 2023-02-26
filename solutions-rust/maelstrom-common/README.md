# Maelstrom

[jepsen-io/maelstrom](https://github.com/jepsen-io/maelstrom) is a workbench for testing toy implementations of distributed systems.

This crate abstracts away the boilerplate of setting up the stdin/stdout for a node in a distributed system, and provides a few useful utilities for writing handlers.

*Note*: This crate is inspired from and primarily written for the [Fly.io Distributed Systems challenge](https://fly.io/dist-sys/).



## Usage

TLDR; Your node is an actor in the system that communicates with `Maelstrom` using enveloped messages.

You'll need to implement the `Actor` trait. This trait has two associated types, `InboundMessage` and `OutboundMessage`. These types are used to define the types of messages that your actor can receive and send, respectively. 

Write a `handle_message` that processes the requests coming in from clients and return the response appropriately.


## Example

A solution to the [first challenge](https://fly.io/dist-sys/1/).

```rust
use maelstrom_common::{
    Actor, 
    Envelope,
    run
};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Request {
    #[serde(rename = "init")]
    Init {
        msg_id: usize,
        node_id: String,
    },
    #[serde(rename = "echo")]
    Echo {
        echo: String,
        msg_id: usize
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Response {
    #[serde(rename = "init_ok")]
    InitOk {
        in_reply_to: usize,
    },
    #[serde(rename = "echo_ok")]
    EchoOk {
        echo: String,
        in_reply_to: usize
    }
}


#[derive(Debug, Default)]
pub struct Echo {
    // Store our ID when a client initializes us.
    node_id: Option<String>
}


impl Actor for Echo {
    type InboundMessage = Request;
    type OutboundMessage = Response;
    
    fn handle_message(&mut self, msg: Envelope<Self::InboundMessage>) -> Option<Envelope<Self::OutboundMessage>> {

        Some(match msg.body {
            Request::Init { msg_id, node_id } => {
                self.node_id = Some(node_id.clone());
                eprintln!("[INIT] Initialized node: {}", node_id);
                Envelope {
                    body: Response::InitOk { in_reply_to: msg_id },
                    src: msg.dest,
                    dest: msg.src
                }
            },
            Request::Echo { echo, msg_id } => {
                eprintln!("[ECHO] Echoing back: {}, in reply to: {}", echo, msg_id);
                Envelope {
                    src: msg.dest,
                    dest: msg.src,
                    body: Response::EchoOk { echo, in_reply_to: msg_id }
                }
            },
        })
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Echo::default())?;
    Ok(())
}
```

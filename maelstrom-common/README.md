# Maelstrom

[jepsen-io/maelstrom](https://github.com/jepsen-io/maelstrom) is a workbench for testing toy implementations of distributed systems.

This crate abstracts away the boilerplate of setting up the stdin/stdout for a node in a distributed system, and provides a few useful utilities for writing handlers.

*Note*: This crate is inspired from and primarily written for the [Fly.io Distributed Systems challenge](https://fly.io/dist-sys/).



## Usage

TLDR; Your node is an actor in the system that communicates with `Maelstrom` using enveloped messages.

You'll need to implement the `Actor` trait. This trait has two associated types, `InboundMessage` and `OutboundMessage`. These types are used to define the types of messages that your actor can receive and send, respectively. 

Write a `handle_message` that processes the requests coming in from clients and return the response appropriately.


## Example

```rust
use maelstrom_common::{Actor, Envelope};
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Echo {
   // Store our ID when a client initializes us.
  node_id: Option<String>
}


impl Actor for Echo {
    // We'll work with loosely a typed Json value.
    type InboundMessage = Value;
    // Same here.
    type OutboundMessage = Value;
   
    fn handle_message(
        &mut self, 
        msg: Envelope<Self::InboundMessage>
    ) -> Option<Envelope<Self::OutboundMessage>> {

        let payload = msg.body;

        // Echo the payload back but wrapped
        // in a return envelope by swapping 
        // the destination and source.
        Some(Envelope::new(msg.dest, msg.src, payload))
    }
}


pub fn main() {
    maelstrom_common::run(
        Echo::default()
    ).unwrap();
}
```
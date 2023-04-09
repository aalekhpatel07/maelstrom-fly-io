# Maelstrom

[jepsen-io/maelstrom](https://github.com/jepsen-io/maelstrom) is a workbench for testing toy implementations of distributed systems.

This crate abstracts away the boilerplate of setting up the stdin/stdout for a node
in a distributed system, and provides a few useful utilities for writing handlers.

This crate is inspired from and primarily written for the [Fly.io Distributed Systems challenges](https://fly.io/dist-sys/).

# Usage

To use this crate, you'll create a node that is capable of handling
some rpcs. Define the rpc messages with a serializable `Message` enum
and define any meaningful error type that can stop the maelstrom test early
in case of something going terribly wrong with the node.

The node must implement the [HandleMessage] trait, which requires
a `handle_message` function that takes an [Envelope] and a [Sender] for optionally
sending any messages.
## Example

Let's create a simple echo node that responds to `init` and `echo` messages.
This also corresponds to the [Echo challenge](https://fly.io/dist-sys/1/) in the [Fly.io Distributed Systems challenge set](https://fly.io/dist-sys/).

```rust
use maelstrom_common::{run, HandleMessage, Envelope};
use serde::{Deserialize, Serialize};
use core::panic;
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    #[serde(rename = "init")]
    Init {
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<usize>,
        node_id: String
    },
    #[serde(rename = "echo")]
    Echo {
        echo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        msg_id: Option<usize>
    },
    #[serde(rename = "init_ok")]
    InitOk {
        #[serde(skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<usize>
    },
    #[serde(rename = "echo_ok")]
    EchoOk {
        echo: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        in_reply_to: Option<usize>
    },
}
#[derive(Debug, Default)]
pub struct Echo {
    // Store our ID when a client initializes us.
    node_id: Option<String>,
}
impl HandleMessage for Echo {
    type Message = Message;
    type Error = std::io::Error;
    fn handle_message(
        &mut self,
        msg: Envelope<Self::Message>,
        outbound_msg_tx: std::sync::mpsc::Sender<Envelope<Self::Message>>,
    ) -> Result<(), Self::Error> {
        match msg.body {
            Message::Init { msg_id, ref node_id } => {
                self.node_id = Some(node_id.clone());
                outbound_msg_tx.send(
                    msg.reply(Message::InitOk { in_reply_to: msg_id })
                ).unwrap();
                Ok(())
            },
            Message::Echo { ref echo, msg_id } => {
                outbound_msg_tx.send(
                    msg.reply(
                    Message::EchoOk { echo: echo.to_owned(), in_reply_to: msg_id }
                    )
                ).unwrap();
                Ok(())
            },
            _ => panic!("{}", format!("Unexpected message: {:#?}", serde_json::to_string_pretty(&msg)))
        }
    }
}
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    run(Echo::default())?;
   Ok(())
}
```

## Contributing

[Issues](https://github.com/aalekhpatel07/maelstrom-fly-io/issues/new), [Pull Requests](https://github.com/aalekhpatel07/maelstrom-fly-io/pulls), and [Github stars](https://github.com/aalekhpatel07/maelstrom-fly-io) are always appreciated.

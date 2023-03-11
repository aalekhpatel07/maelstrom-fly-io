use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A body contains identifiers for messages
/// and replies if it is a communication between
/// the client and the server. For messages between
/// servers, msg_id's and in_reply_to's are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<M> {
    /// The id that the client gives us for any rpc it makes.
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_id: Option<usize>,

    /// The message our rpc response corresponds to.
    #[serde(skip_serializing_if = "Option::is_none")]
    in_reply_to: Option<usize>,

    /// The actual payload.
    #[serde(flatten)]
    message: M,
}

impl<M> Body<M> {
    pub fn msg_id(&self) -> Option<usize> {
        self.msg_id
    }
    pub fn in_reply_to(&self) -> Option<usize> {
        self.in_reply_to
    }
    pub fn message(&self) -> &M {
        &self.message
    }
}

static MESSAGE_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Envelope<M> {
    pub src: String,
    pub dest: String,
    body: Body<M>,
}

impl<M> Envelope<M>
where
    M: Serialize,
{
    /// Create a new envelope from the source to the destination,
    /// (optionally) as a reply to another message, that contains a given body.
    pub fn new(src: &str, dest: &str, in_reply_to: Option<usize>, message: M) -> Envelope<M> {
        Self {
            src: src.to_owned(),
            dest: dest.to_owned(),
            body: Body {
                msg_id: Some(MESSAGE_ID.fetch_add(1, Ordering::SeqCst)),
                in_reply_to,
                message,
            },
        }
    }

    /// Returns whether this envelope has messages meant
    /// for inter-server communication.
    pub fn is_internal(&self) -> bool {
        self.src.starts_with('n')
    }

    /// Generate a reply for us envelope that contains
    /// the specified body.
    pub fn reply(&self, message: M) -> Envelope<M> {
        Envelope {
            src: self.dest.clone(),
            dest: self.src.clone(),
            body: Body {
                msg_id: Some(MESSAGE_ID.fetch_add(1, Ordering::SeqCst)),
                in_reply_to: self.msg_id(),
                message,
            },
        }
    }

    /// Send messages out to stdout.
    pub fn send(&self) {
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(&mut stdout, self).unwrap();
        stdout.write_all(b"\n").unwrap();
        stdout.flush().unwrap();
    }
}

/// So that whatever pub api Body offers,
/// Envelope can too.
impl<M> Deref for Envelope<M> {
    type Target = Body<M>;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

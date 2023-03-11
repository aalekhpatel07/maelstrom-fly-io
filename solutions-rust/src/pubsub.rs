use serde::{de::DeserializeOwned, Serialize};
use std::sync::mpsc::{channel, Receiver, Sender};

use std::fmt::Debug;
use std::thread::spawn;

use crate::envelope::Envelope;

#[derive(Debug)]
pub struct PubSub<M> {
    rx: Receiver<Envelope<M>>,
    subscribers: Vec<Sender<Envelope<M>>>,
}

impl<M> PubSub<M>
where
    M: DeserializeOwned + Serialize + Clone + Send + Sync + 'static + Debug,
{
    pub fn new(rx: Receiver<Envelope<M>>) -> Self {
        Self {
            rx,
            subscribers: vec![],
        }
    }

    pub fn subscribe(&mut self) -> Receiver<Envelope<M>> {
        let (tx, rx) = channel();
        self.subscribers.push(tx);
        rx
    }

    pub fn start(self) {
        let (tx, rx) = channel::<Envelope<M>>();

        spawn(move || {
            for msg in rx {
                for subscriber in &self.subscribers {
                    subscriber.send(msg.clone()).unwrap();
                }
            }
        });

        spawn(move || {
            while let Ok(line) = self.rx.recv() {
                tx.send(line).unwrap();
            }
        });
    }
}

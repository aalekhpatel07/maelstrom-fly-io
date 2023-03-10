use broadcast::Message;
use std::{
    sync::mpsc::{
        channel, 
        Sender, 
        Receiver
    },
    io::stdin, 
    thread::spawn
};

use maelstrom_common::{
    PostOffice, 
    Envelope,
    listen
};


pub fn main() {

    let (line_tx, line_rx) = channel();
    let mut post_office = PostOffice::<Envelope<Message>>::new(line_rx);

    let _rx = post_office.subscribe();
    post_office.start();

    listen(line_tx);
}

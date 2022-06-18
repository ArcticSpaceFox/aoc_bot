use std::num::NonZeroU64;

use twilight_model::util::Timestamp;

#[derive(Debug)]
pub enum Event {
    Ping(Message),
    AdventOfCode(Message),
    FourtyTwo(Message),
    TopThree(Message),
    Shutdown,
}

#[derive(Debug)]
pub struct Message {
    pub channel_id: NonZeroU64,
    pub author: Option<Author>,
    pub timestamp: Option<Timestamp>,
}

#[derive(Debug)]
pub struct Author {
    pub id: NonZeroU64,
    pub name: String,
}

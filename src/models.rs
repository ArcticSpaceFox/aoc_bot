use std::num::NonZeroU64;

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
}

#[derive(Debug)]
pub struct Author {
    pub id: NonZeroU64,
    pub name: String,
}

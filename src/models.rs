#[derive(Debug)]
pub enum Event {
    Ping(Message),
    AdventOfCode(Message),
    FourtyTwo(Message),
    Shutdown,
}

#[derive(Debug)]
pub struct Message {
    pub channel_id: u64,
    pub author: Option<Author>,
}

#[derive(Debug)]
pub struct Author {
    pub id: u64,
    pub name: String,
}

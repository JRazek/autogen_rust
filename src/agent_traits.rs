use futures::{Sink, Stream};

pub trait AgentProxy<Message>: Stream<Item = Message> + Sink<Message> {}

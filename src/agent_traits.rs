use std::vec::IntoIter;

use futures::{Sink, Stream};

//not equivalent to python autogen UserProxyAgent
//this just provides a facade to an agent
pub trait AgentProxy<Message>: Stream<Item = Message> + Sink<Message> {}

impl<Message, S> AgentProxy<Message> for S where S: Stream<Item = Message> + Sink<Message> {}

pub trait Agent<Message> {
    type Proxy: AgentProxy<Message>;
    fn take_turn(&self, chat_history: impl IntoIterator<Item = Message>) -> Self::Proxy;
}

use std::vec::IntoIter;

use futures::{Sink, Stream};

//not equivalent to python autogen UserProxyAgent
//this just provides a facade to an agent

pub trait AgentProxyStream<Message>: Stream<Item = Message> {}

pub trait AgentProxySink<Message>: Sink<Message> {}

impl<Message, S> AgentProxySink<Message> for S where S: Sink<Message> {}

impl<Message, S> AgentProxyStream<Message> for S where S: Stream<Item = Message> {}

pub trait Agent<Message> {
    type ProxyStream: AgentProxyStream<Message>;
    type ProxySink: AgentProxySink<Message>;

    fn take_turn(&self, chat_history: impl IntoIterator<Item = Message>) -> Self::ProxyStream;
    fn receive_turn(&self, chat_history: impl IntoIterator<Item = Message>) -> Self::ProxySink;
}

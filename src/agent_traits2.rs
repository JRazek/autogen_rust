use futures::{Sink, Stream};

use async_trait::async_trait;

#[async_trait]
pub trait Agent<M> {
    type AgentProxyStream: Stream<Item = M>;
    type AgentProxySink: Sink<M>;

    fn sink(&mut self) -> Self::AgentProxySink;
    fn stream(&mut self) -> Self::AgentProxyStream;
}

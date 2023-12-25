use futures::{Sink, Stream};

use async_trait::async_trait;

#[async_trait]
pub trait Agent<M> {
    async fn receive(&mut self, stream: impl Stream<Item = M>);
    async fn send(&mut self, sink: impl Sink<M>);
}

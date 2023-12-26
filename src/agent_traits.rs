use futures::{Sink, Stream};

use async_trait::async_trait;

#[async_trait]
pub trait Agent<Mrx, Mtx> {
    type Error;

    async fn receive(&mut self, stream: impl Stream<Item = Mrx> + Unpin + Send);
    async fn send(&mut self, sink: impl Sink<Mtx> + Unpin + Send) -> Result<(), Self::Error>;
}

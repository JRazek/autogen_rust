use async_trait::async_trait;

//receive/send should accept just a Mrx and not stream/sink. If required, may simply accept stream/sink as an
//Mrx. Now its restricting usage.
#[async_trait]
pub trait Agent<Mrx, Mtx> {
    type Error;

    async fn receive(&mut self, mrx: Mrx);
    async fn send(&mut self) -> Result<Mtx, Self::Error>;
}

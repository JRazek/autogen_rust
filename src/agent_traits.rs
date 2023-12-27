use async_trait::async_trait;

#[async_trait]
pub trait Agent<Mrx, Mtx> {
    type Error;

    //Mrx and Mtx may include the information about the sender and receiver
    //this differs from the original autogen.
    async fn receive(&mut self, mrx: Mrx);
    async fn send(&mut self) -> Result<Mtx, Self::Error>;
}

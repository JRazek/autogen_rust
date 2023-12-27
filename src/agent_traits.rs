use async_trait::async_trait;

#[async_trait]
pub trait ConsumerAgent<Mrx> {
    type Error;

    async fn receive(&mut self, mrx: Mrx) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait RespondingAgent<Mrx> {
    type Mtx;
    type Error;

    async fn receive_and_reply(&mut self, mrx: Mrx) -> Result<Self::Mtx, Self::Error>;
}

#[async_trait]
pub trait ProducerAgent {
    type Mtx;
    type Error;

    /// Based on the current state of Agent, reply with a message.
    async fn generate_prompt(&mut self) -> Result<Self::Mtx, Self::Error>;
}

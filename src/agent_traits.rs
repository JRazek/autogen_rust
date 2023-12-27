use async_trait::async_trait;

#[async_trait]
pub trait Agent<Mrx, Mtx> {
    type Error;

    /// Receive a message from the agent's environment.
    /// Agent is responsible for keeping the appropriate context and state of previous messages.
    async fn receive(&mut self, mrx: Mrx);

    /// Based on the current state of Agent, reply with a message.
    async fn reply(&mut self) -> Result<Mtx, Self::Error>;
}

#[async_trait]
pub trait ProducerAgent<Mtx> {
    type Error;

    /// Based on the current state of Agent, reply with a message.
    async fn generate_prompt(&mut self) -> Result<Mtx, Self::Error>;
}

#[async_trait]
pub trait ConsumerAgent<Mrx> {
    type Error;

    async fn receive(&mut self, mrx: Mrx) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait RespondingAgent<Mrx, Mtx> {
    type Error;

    async fn receive_and_reply(&mut self, mrx: Mrx) -> Result<Mtx, Self::Error>;
}

pub trait ConsumerAgent {
    type Mrx;
    type Error;

    async fn receive_message(&mut self, mrx: Self::Mrx) -> Result<(), Self::Error>;
}

pub trait ProducerAgent {
    type Mtx;
    type Error;

    /// Based on the current state of Agent, reply with a message.
    async fn send_message(&mut self) -> Result<Self::Mtx, Self::Error>;
}

pub trait NamedAgent {
    fn name(&self) -> &str;
}

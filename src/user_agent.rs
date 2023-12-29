use async_trait::async_trait;

#[async_trait]
pub trait UserAgent {
    type Mrx;
    type Mtx;

    type SendingError;
    type ReceivingError;

    async fn receive_message(&self, mrx: Self::Mrx) -> Result<(), Self::ReceivingError>;
    async fn send_message(&self) -> Result<Self::Mtx, Self::SendingError>;
}

use crate::agent_traits::RespondingAgent;

pub enum RespondingAgentError<S, R> {
    Sending(S),
    Receiving(R),
}

#[async_trait]
impl<A, Mrx, Mtx> RespondingAgent for A
where
    A: UserAgent<Mrx = Mrx, Mtx = Mtx> + Send + Sync + 'static,
    Mrx: Send + 'static,
{
    type Mrx = Mrx;
    type Mtx = Mtx;
    type Error = RespondingAgentError<A::SendingError, A::ReceivingError>;

    async fn receive_and_reply(&mut self, mrx: Mrx) -> Result<Self::Mtx, Self::Error> {
        self.receive_message(mrx)
            .await
            .map_err(RespondingAgentError::Receiving)?;

        self.send_message()
            .await
            .map_err(RespondingAgentError::Sending)
    }
}

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

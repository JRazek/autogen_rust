use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Agent {0} failed to send message")]
    SchedulerInvalidBounds(usize),

    #[error("Agent {0} failed to send message")]
    AgentTurnTxSendError(String),

    #[error("Agent {0} failed to receive message")]
    AgentTurnDoneRxRecvError(String),
}

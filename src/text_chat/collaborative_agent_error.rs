use crate::agent_traits::{ConsumerAgent, ProducerAgent};

pub enum CollaborativeAgentError<C, R>
where
    C: ProducerAgent,
    R: ConsumerAgent,
{
    Sending(C::Error),
    Receiving(R::Error),
    TryFromMessage,
    TryIntoString,
}

use std::fmt::Debug;

impl<C, R> Debug for CollaborativeAgentError<C, R>
where
    C: ProducerAgent,
    R: ConsumerAgent,
    C::Error: Debug,
    R::Error: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaborativeAgentError::Sending(e) => write!(f, "Sending error: {:?}", e),
            CollaborativeAgentError::Receiving(e) => write!(f, "Receiving error: {:?}", e),
            CollaborativeAgentError::TryFromMessage => write!(f, "TryFromMessage error"),
            CollaborativeAgentError::TryIntoString => write!(f, "TryIntoString error"),
        }
    }
}

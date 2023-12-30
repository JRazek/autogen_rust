use crate::agent_traits::{ConsumerAgent, ProducerAgent};

pub enum ChatUserAgentError<C, R>
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

impl<C, R> Debug for ChatUserAgentError<C, R>
where
    C: ProducerAgent,
    R: ConsumerAgent,
    C::Error: Debug,
    R::Error: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatUserAgentError::Sending(e) => write!(f, "Sending error: {:?}", e),
            ChatUserAgentError::Receiving(e) => write!(f, "Receiving error: {:?}", e),
            ChatUserAgentError::TryFromMessage => write!(f, "TryFromMessage error"),
            ChatUserAgentError::TryIntoString => write!(f, "TryIntoString error"),
        }
    }
}

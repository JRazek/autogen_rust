use super::chat_user_agent::RequestCodeFeedback;
use super::collaborative_agent::CollaborativeAgent;
use crate::agent_traits::{ConsumerAgent, RespondingAgent};

use super::code::CodeExecutor;

pub enum CollaborativeChatError<UA, CA, E>
where
    UA: RespondingAgent,
    UA: ConsumerAgent,
    UA: RequestCodeFeedback,

    CA: CollaborativeAgent,

    E: CodeExecutor,
{
    ConsumerAgent(<UA as ConsumerAgent>::Error),
    RespondingAgent(<UA as RespondingAgent>::Error),
    RequestCodeFeedback(<UA as RequestCodeFeedback>::Error),
    CollaborativeAgent(CA::Error),
    CodeExecutor(E::Error),
}

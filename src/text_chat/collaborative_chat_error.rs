use super::collaborative_agent::CollaborativeAgent;
use super::chat_user_agent::RequestCodeFeedback;
use crate::agent_traits::{ConsumerAgent, RespondingAgent};

pub enum CollaborativeChatError<UA, CA>
where
    UA: RespondingAgent,
    UA: ConsumerAgent,
    UA: RequestCodeFeedback,

    CA: CollaborativeAgent,
{
    ConsumerAgent(<UA as ConsumerAgent>::Error),
    RespondingAgent(<UA as RespondingAgent>::Error),
    RequestCodeFeedback(<UA as RequestCodeFeedback>::Error),
    CollaborativeAgent(CA::Error),
}

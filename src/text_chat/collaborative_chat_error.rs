use super::chat_user_agent::ChatUserAgent;
use super::collaborative_agent::CollaborativeAgent;

use super::code::CodeExecutor;

pub enum CollaborativeChatError<UA, CA, E>
where
    UA: ChatUserAgent,

    CA: CollaborativeAgent,

    E: CodeExecutor,
{
    ChatUserAgent(UA::Error),
    CollaborativeAgent(CA::Error),
    CodeExecutor(E::Error),
}

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

impl<UA, CA, E> std::fmt::Debug for CollaborativeChatError<UA, CA, E>
where
    UA: ChatUserAgent,
    <UA as ChatUserAgent>::Error: std::fmt::Debug,

    CA: CollaborativeAgent,
    <CA as CollaborativeAgent>::Error: std::fmt::Debug,

    E: CodeExecutor,
    <E as CodeExecutor>::Error: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollaborativeChatError::ChatUserAgent(e) => {
                write!(f, "ChatUserAgent({:?})", e)
            }
            CollaborativeChatError::CollaborativeAgent(e) => {
                write!(f, "CollaborativeAgent({:?})", e)
            }
            CollaborativeChatError::CodeExecutor(e) => {
                write!(f, "CodeExecutor({:?})", e)
            }
        }
    }
}

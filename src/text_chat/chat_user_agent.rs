use async_trait::async_trait;

use crate::agent_traits::{ConsumerAgent, NamedAgent, RespondingAgent};
use crate::user_agent::UserAgent;

use super::code::CodeBlock;

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
}

#[async_trait]
pub trait RequestCodeFeedback {
    type Error;

    async fn request_code_block_feedback(
        &mut self,
        sender: String,
        comment: &str,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error>;
}

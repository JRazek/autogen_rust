use async_trait::async_trait;

use crate::agent_traits::{ConsumerAgent, NamedAgent, RespondingAgent};

use super::collaborative_agent::CollaborativeAgentResponse;

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

use super::code::CodeBlockExecutionResult;

#[async_trait]
pub trait ChatUserAgent {
    type Error;

    async fn receive_and_reply(
        &mut self,
        sender: &str,
        message: &str,
    ) -> Result<String, Self::Error>;

    async fn silent_receive(
        &mut self,
        sender: &str,
        response: &CollaborativeAgentResponse,
    ) -> Result<(), Self::Error>;

    async fn request_code_block_feedback(
        &mut self,
        sender: &str,
        comment: &str,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error>;

    async fn receive_collaborative_agent_response(
        &mut self,
        response: &CollaborativeAgentResponse,
    ) -> Result<(), Self::Error>;

    async fn receive_code_execution_result(
        &mut self,
        result: &CodeBlockExecutionResult,
    ) -> Result<(), Self::Error>;
}

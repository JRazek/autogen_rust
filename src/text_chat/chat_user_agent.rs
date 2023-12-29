use async_trait::async_trait;

use super::collaborative_agent::CollaborativeAgentResponse;

use super::code::CodeBlock;

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
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

use crate::user_agent::UserAgent;

pub enum UserAgentError<S, R> {
    Sending(S),
    Receiving(R),
}

pub struct SenderMessage<'a> {
    sender: &'a str,
    message: &'a str,
}

/// This is a convenience implementation of ChatUserAgent for any UserAgent.
#[async_trait]
impl<UA, Mrx, Mtx> ChatUserAgent for UA
where
    UA: UserAgent<Mrx = Mrx, Mtx = Mtx> + Send,
    for<'a> Mrx: From<SenderMessage<'a>>,
    Mtx: Into<String>,
{
    type Error = UserAgentError<UA::SendingError, UA::ReceivingError>;

    async fn receive_and_reply(
        &mut self,
        sender: &str,
        message: &str,
    ) -> Result<String, Self::Error> {
        todo!()
    }

    async fn silent_receive(
        &mut self,
        sender: &str,
        response: &CollaborativeAgentResponse,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn request_code_block_feedback(
        &mut self,
        sender: &str,
        comment: &str,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error> {
        todo!()
    }

    async fn receive_collaborative_agent_response(
        &mut self,
        response: &CollaborativeAgentResponse,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn receive_code_execution_result(
        &mut self,
        result: &CodeBlockExecutionResult,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

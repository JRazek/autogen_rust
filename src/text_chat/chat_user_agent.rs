use super::collaborative_agent::CollaborativeAgentResponse;

use super::code::CodeBlock;

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
}

use super::code::CodeBlockExecutionResult;

use crate::agent_traits::{ConsumerAgent, ProducerAgent};

pub trait ChatUserAgent {
    type Error;

    async fn receive_and_reply(
        &mut self,
        sender: &str,
        message: &str,
    ) -> Result<String, Self::Error>;

    async fn silent_receive_collaborative_agent_response(
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

    async fn receive_code_execution_result(
        &mut self,
        result: &CodeBlockExecutionResult,
    ) -> Result<(), Self::Error>;
}

pub enum UserAgentError<C, R>
where
    C: ProducerAgent,
    R: ConsumerAgent,
{
    Sending(C::Error),
    Receiving(R::Error),
    TryFromMessage,
    TryIntoString,
}

pub enum Message<'a> {
    Text {
        sender: &'a str,
        message: &'a str,
    },
    CollaborativeAgentResponse {
        sender: &'a str,
        response: &'a CollaborativeAgentResponse,
    },
    CodeBlockFeedback {
        sender: &'a str,
        comment: &'a str,
        code_block: &'a CodeBlock,
    },
    CodeBlockExecutionResult(&'a CodeBlockExecutionResult),
}

/// This is a convenience implementation of ChatUserAgent for any Agent that implements
/// ConsumerAgent and ProducerAgent.
impl<UA, Mrx, Mtx> ChatUserAgent for UA
where
    UA: ConsumerAgent<Mrx = Mrx> + ProducerAgent<Mtx = Mtx>,
    UA: Send,

    for<'a> Mrx: TryFrom<Message<'a>> + Send,
    Mtx: TryInto<String>,
    Mtx: TryInto<CodeBlockFeedback>,
{
    type Error = UserAgentError<UA, UA>;

    async fn receive_and_reply(
        &mut self,
        sender: &str,
        message: &str,
    ) -> Result<String, Self::Error> {
        let message = Message::Text { sender, message };

        let message = Mrx::try_from(message).map_err(|_| UserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(UserAgentError::Receiving)?;

        let response = self.send_message().await.map_err(UserAgentError::Sending)?;

        let response = response
            .try_into()
            .map_err(|_| UserAgentError::TryIntoString)?;

        Ok(response)
    }

    async fn silent_receive_collaborative_agent_response(
        &mut self,
        sender: &str,
        response: &CollaborativeAgentResponse,
    ) -> Result<(), Self::Error> {
        let message = Message::CollaborativeAgentResponse { sender, response };

        let message = Mrx::try_from(message).map_err(|_| UserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(UserAgentError::Receiving)?;

        Ok(())
    }

    async fn request_code_block_feedback(
        &mut self,
        sender: &str,
        comment: &str,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error> {
        let message = Message::CodeBlockFeedback {
            sender,
            comment,
            code_block,
        };

        let message = Mrx::try_from(message).map_err(|_| UserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(UserAgentError::Receiving)?;

        let response = self.send_message().await.map_err(UserAgentError::Sending)?;

        let response = response
            .try_into()
            .map_err(|_| UserAgentError::TryIntoString)?;

        Ok(response)
    }

    async fn receive_code_execution_result(
        &mut self,
        result: &CodeBlockExecutionResult,
    ) -> Result<(), Self::Error> {
        let message = Message::CodeBlockExecutionResult(result);

        let message = Mrx::try_from(message).map_err(|_| UserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(UserAgentError::Receiving)?;

        Ok(())
    }
}

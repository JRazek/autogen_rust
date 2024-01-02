use super::collaborative_agent::CollaborativeAgentResponse;

use super::code::CodeBlock;

use super::code::CodeBlockExecutionResult;

use crate::agent_traits::{ConsumerAgent, ProducerAgent};

use super::chat_user_agent_error::ChatUserAgentError;

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
}

/// This trait is used by the collaborative chat to communicate with the user.
/// Even though it is implemented for all the ConsumerAgents and ProducerAgents,
/// I believe it is better to have a separate trait for this purpose.
/// Consumer/Producer force user to dynamically distiguish between the types of queries.
/// If used with stdin/stdout Consumer/Producer apprach will probably be better (unless we want
/// custom formatting for different inputs), but in the case of tests or other implementations it might be easier and more explicit to use this trait

pub trait ChatUserAgent {
    type Error;

    async fn receive_and_reply(
        &mut self,
        sender: String,
        message: String,
    ) -> Result<String, Self::Error>;

    async fn silent_receive_collaborative_agent_response(
        &mut self,
        sender: String,
        response: CollaborativeAgentResponse,
    ) -> Result<(), Self::Error>;

    async fn request_code_block_feedback(
        &mut self,
        sender: String,
        comment: String,
        code_block: CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error>;

    async fn receive_code_execution_result(
        &mut self,
        result: CodeBlockExecutionResult,
    ) -> Result<(), Self::Error>;
}

pub enum Message {
    Text {
        sender: String,
        message: String,
    },
    CollaborativeAgentResponse {
        sender: String,
        response: CollaborativeAgentResponse,
    },
    CodeBlockFeedback {
        sender: String,
        comment: String,
        code_block: CodeBlock,
    },
    CodeBlockExecutionResult(CodeBlockExecutionResult),
}

/// This is a convenience implementation of ChatUserAgent for any Agent that implements
/// ConsumerAgent and ProducerAgent.
impl<UA, Mrx, Mtx> ChatUserAgent for UA
where
    UA: ConsumerAgent<Mrx = Mrx> + ProducerAgent<Mtx = Mtx>,
    UA: Send,

    Mrx: TryFrom<Message> + Send,
    Mtx: TryInto<String>,
    Mtx: TryInto<CodeBlockFeedback>,
{
    type Error = ChatUserAgentError<UA, UA>;

    async fn receive_and_reply(
        &mut self,
        sender: String,
        message: String,
    ) -> Result<String, Self::Error> {
        let message = Message::Text { sender, message };

        let message = Mrx::try_from(message).map_err(|_| ChatUserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(ChatUserAgentError::Receiving)?;

        let response = self
            .send_message()
            .await
            .map_err(ChatUserAgentError::Sending)?;

        let response = response
            .try_into()
            .map_err(|_| ChatUserAgentError::TryIntoString)?;

        Ok(response)
    }

    async fn silent_receive_collaborative_agent_response(
        &mut self,
        sender: String,
        response: CollaborativeAgentResponse,
    ) -> Result<(), Self::Error> {
        let message = Message::CollaborativeAgentResponse { sender, response };

        let message = Mrx::try_from(message).map_err(|_| ChatUserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(ChatUserAgentError::Receiving)?;

        Ok(())
    }

    async fn request_code_block_feedback(
        &mut self,
        sender: String,
        comment: String,
        code_block: CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error> {
        let message = Message::CodeBlockFeedback {
            sender,
            comment,
            code_block,
        };

        let message = Mrx::try_from(message).map_err(|_| ChatUserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(ChatUserAgentError::Receiving)?;

        let response = self
            .send_message()
            .await
            .map_err(ChatUserAgentError::Sending)?;

        let response = response
            .try_into()
            .map_err(|_| ChatUserAgentError::TryIntoString)?;

        Ok(response)
    }

    async fn receive_code_execution_result(
        &mut self,
        result: CodeBlockExecutionResult,
    ) -> Result<(), Self::Error> {
        let message = Message::CodeBlockExecutionResult(result);

        let message = Mrx::try_from(message).map_err(|_| ChatUserAgentError::TryFromMessage)?;

        self.receive_message(message)
            .await
            .map_err(ChatUserAgentError::Receiving)?;

        Ok(())
    }
}

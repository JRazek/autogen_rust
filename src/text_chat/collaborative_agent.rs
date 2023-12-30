use super::code::{CodeBlock, CodeBlockExecutionResult};

use crate::agent_traits::{ConsumerAgent, ProducerAgent};

use super::collaborative_agent_error::CollaborativeAgentError;

/// This should correspond to the response from the LLM.
/// User: Please write Hello World in Python and then execute it.
///
/// CollaborativeAgent: Certainly I can write Hello World in Python. Here it is:
/// ```python
/// print("Hello World")
/// ```
///
/// Which will translate to:
///
/// CommentedCodeBlock {
///    comment: "Certainly I can write Hello World in Python. Here it is:".to_string(),
///    code_block: CodeBlock {
///        code: "print(\"Hello World\")".to_string(),
///        language: Language::Python,
///    },
///    request_execution: true,
/// }

#[derive(Debug, Clone)]
pub struct CommentedCodeBlock {
    pub comment: String,
    pub code_block: CodeBlock,
    pub request_execution: bool,
}

/// Agent may simply respond with a text message or with a code blocks.

#[derive(Debug, Clone)]
pub enum CollaborativeAgentResponse {
    Text(String),
    CommentedCodeBlock(CommentedCodeBlock),
}

pub trait CollaborativeAgent {
    // Shared error should be the sufficient for both functions.
    type Error;

    async fn receive_and_reply(
        &mut self,
        sender: String,
        message: String,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;

    /// We always request reply from the agent if execution was denied.
    /// User feedback is obligatory thus the agent may respond to that and potentially fix the
    /// issue.
    ///
    ///
    /// ```ignore
    /// use autogen::text_chat::code::CodeBlock;
    ///
    /// let reply = collaborative_agent.receive_and_reply("Please write Hello World in Python and then execute it.".to_string())
    ///
    /// assert_eq!(reply,
    ///     CollaborativeAgentResponse::CodeBlock(
    ///         CommentedCodeBlock {
    ///            comment: "Certainly I can write Hello World in Python. Here it is:".to_string(),
    ///            code_block: CodeBlock {
    ///                code: "launch_missiles()".to_string(),
    ///                language: "python".to_string(),
    ///            },
    ///            request_execution: true,
    ///         }
    ///     )
    /// );
    ///
    /// let fixed_code = collaborative_agent.deny_code_block_execution("Please do not nuke us.".to_string());
    ///
    /// ```
    ///
    /// we may repeat the same process until agent decides that it no longer has an interest in
    /// destroying the world.
    ///

    async fn deny_code_block_execution(
        &mut self,
        code_block: CodeBlock,
        feedback: String,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;

    async fn receive_code_and_reply_to_execution_result(
        &mut self,
        code_execution_result: CodeBlockExecutionResult,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;
}

pub enum Message {
    Text {
        sender: String,
        message: String,
    },
    CodeExecutionDenied {
        comment: String,
        code_block: CodeBlock,
    },
    CodeExecutionResult(CodeBlockExecutionResult),
}

/// This may be used when the agent returns output as a string or any other type.
/// One may imagine that 3rd party LLM provides a JSON response.
/// In that case we may simply implement the communication via ConsumerAgent and ProducerAgent and
/// then use after implementing TryFrom<Message> and TryInto<CollaborativeAgentResponse>, we may
/// use this trait implementation in collaborative chat.

impl<CA, Mrx, Mtx> CollaborativeAgent for CA
where
    CA: ConsumerAgent<Mrx = Mrx> + ProducerAgent<Mtx = Mtx>,
    CA: Send,

    Mrx: TryFrom<Message> + Send,
    Mtx: TryInto<CollaborativeAgentResponse>,
{
    type Error = CollaborativeAgentError<CA, CA>;

    async fn receive_and_reply(
        &mut self,
        sender: String,
        message: String,
    ) -> Result<CollaborativeAgentResponse, Self::Error> {
        let sender = sender.to_owned();
        let message = message.to_owned();

        let message = Message::Text { sender, message };

        send_and_get_reply(message, self).await
    }
    async fn deny_code_block_execution(
        &mut self,
        code_block: CodeBlock,
        feedback: String,
    ) -> Result<CollaborativeAgentResponse, Self::Error> {
        let feedback = feedback.to_owned();

        let message = Message::CodeExecutionDenied {
            comment: feedback,
            code_block,
        };

        send_and_get_reply(message, self).await
    }

    async fn receive_code_and_reply_to_execution_result(
        &mut self,
        code_execution_result: CodeBlockExecutionResult,
    ) -> Result<CollaborativeAgentResponse, Self::Error> {
        let message = Message::CodeExecutionResult(code_execution_result);

        send_and_get_reply(message, self).await
    }
}

/// Helper function.
async fn send_and_get_reply<CA, Mrx, Mtx>(
    message: Message,
    ca: &mut CA,
) -> Result<CollaborativeAgentResponse, CollaborativeAgentError<CA, CA>>
where
    CA: ConsumerAgent<Mrx = Mrx> + ProducerAgent<Mtx = Mtx>,
    CA: Send,

    Mrx: TryFrom<Message> + Send,
    Mtx: TryInto<CollaborativeAgentResponse>,
{
    let message = Mrx::try_from(message).map_err(|_| CollaborativeAgentError::TryFromMessage)?;

    ca.receive_message(message)
        .await
        .map_err(CollaborativeAgentError::Receiving)?;

    let reply = ca
        .send_message()
        .await
        .map_err(CollaborativeAgentError::Sending)?;

    let reply = reply
        .try_into()
        .map_err(|_| CollaborativeAgentError::TryIntoString)?;

    Ok(reply)
}

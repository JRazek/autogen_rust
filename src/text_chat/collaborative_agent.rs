use async_trait::async_trait;

use super::code::{CodeBlock, CodeBlockExecutionResult};

use crate::agent_traits::{ConsumerAgent, ProducerAgent};

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

#[derive(Clone)]
pub struct CommentedCodeBlock {
    pub comment: String,
    pub code_block: CodeBlock,
    pub request_execution: bool,
}

/// Agent may simply respond with a text message or with a code blocks.

#[derive(Clone)]
pub enum CollaborativeAgentResponse {
    Text(String),
    CommentedCodeBlock(CommentedCodeBlock),
}

#[async_trait]
pub trait CollaborativeAgent {
    // Shared error should be the sufficient for both functions.
    type Error;

    async fn receive_and_reply(
        &mut self,
        sender: &str,
        message: &str,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;

    /// We always request reply from the agent if execution was denied.
    /// User feedback is obligatory thus the agent may respond to that and potentially fix the
    /// issue.
    ///
    /// Example:
    ///
    /// let reply = collaborative_agent.request_reply("Please write Hello World in Python and then execute it.".to_string())
    ///
    /// assert_eq!(reply,
    ///     CollaborativeAgentResponse::CodeBlock(
    ///         CommentedCodeBlock {
    ///            comment: "Certainly I can write Hello World in Python. Here it is:".to_string(),
    ///            code_block: CodeBlock {
    ///                code: "launch_missiles()".to_string(),
    ///                language: Language::Python,
    ///            },
    ///            request_execution: true,
    ///         }
    ///     )
    /// );
    ///
    /// let fixed_code = collaborative_agent.denied_code_block_execution("Please do not nuke us.".to_string());
    /// // we may repeat the same process until agent decides that it no longer has an interest in
    /// // destroying the world.
    ///

    async fn deny_code_block_execution(
        &mut self,
        code_block: &CodeBlock,
        feedback: &str,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;

    async fn receive_code_and_reply_to_execution_result(
        &mut self,
        code_execution_result: &CodeBlockExecutionResult,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;
}

pub enum CollaborativeAgentError<C, R>
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
    CodeExecutionDenied {
        comment: &'a str,
        code_block: &'a CodeBlock,
    },
    CodeExecutionResult(&'a CodeBlockExecutionResult),
}

///This may be used when the agent returns output as a string or any other type.
#[async_trait]
impl<CA, Mrx, Mtx> CollaborativeAgent for CA
where
    CA: ConsumerAgent<Mrx = Mrx> + ProducerAgent<Mtx = Mtx>,
    CA: Send,

    for<'a> Mrx: TryFrom<Message<'a>> + Send,
    Mtx: TryInto<CollaborativeAgentResponse>,
{
    type Error = CollaborativeAgentError<CA, CA>;

    async fn receive_and_reply(
        &mut self,
        sender: &str,
        message: &str,
    ) -> Result<CollaborativeAgentResponse, Self::Error> {
        let message = Message::Text { sender, message };

        send_and_get_reply(message, self).await
    }
    async fn deny_code_block_execution(
        &mut self,
        code_block: &CodeBlock,
        feedback: &str,
    ) -> Result<CollaborativeAgentResponse, Self::Error> {
        let message = Message::CodeExecutionDenied {
            comment: feedback,
            code_block,
        };

        send_and_get_reply(message, self).await
    }

    async fn receive_code_and_reply_to_execution_result(
        &mut self,
        code_execution_result: &CodeBlockExecutionResult,
    ) -> Result<CollaborativeAgentResponse, Self::Error> {
        let message = Message::CodeExecutionResult(code_execution_result);

        send_and_get_reply(message, self).await
    }
}

async fn send_and_get_reply<CA, Mrx, Mtx>(
    message: Message<'_>,
    ca: &mut CA,
) -> Result<CollaborativeAgentResponse, CollaborativeAgentError<CA, CA>>
where
    CA: ConsumerAgent<Mrx = Mrx> + ProducerAgent<Mtx = Mtx>,
    CA: Send,

    for<'a> Mrx: TryFrom<Message<'a>> + Send,
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

use async_trait::async_trait;

use super::code::CodeBlock;

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

use super::collaborative_chat::UserTextMessage;

#[async_trait]
pub trait CollaborativeAgent {
    type Error;

    async fn receive_and_reply(
        &mut self,
        message: UserTextMessage,
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
        code_block: &CommentedCodeBlock,
        feedback: String,
    ) -> Result<CollaborativeAgentResponse, Self::Error>;
}

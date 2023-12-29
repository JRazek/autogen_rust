use autogen::agent_traits::{ConsumerAgent, ProducerAgent};
use autogen::text_chat::chat_user_agent::ChatUserAgent;
use autogen::text_chat::code::{CodeBlock, CodeBlockExecutionResult};
use autogen::text_chat::collaborative_agent::{
    CollaborativeAgent, CollaborativeAgentResponse, CommentedCodeBlock,
};

use autogen::text_chat::chat_user_agent::CodeBlockFeedback;

use async_trait::async_trait;

use async_std::io;

use tracing::debug;

enum Error {
    Io(io::Error),
}

//make it as a trait
/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
struct LocalUserAgent;

#[async_trait]
impl ProducerAgent for LocalUserAgent {
    type Mtx = LocalMessage;
    type Error = Error;

    async fn send_message(&mut self) -> Result<Self::Mtx, Self::Error> {
        debug!("Enter send_message");
        let mut input = String::new();
        io::stdin().read_line(&mut input).await.map_err(Error::Io)?;

        Ok(LocalMessage {
            message: input.trim().to_string(),
        })
    }
}

#[async_trait]
impl ConsumerAgent for LocalUserAgent {
    type Mrx = LocalMessage;
    type Error = Error;

    async fn receive_message(&mut self, mrx: Self::Mrx) -> Result<(), Self::Error> {
        debug!("Enter receive_message");
        println!("{}", mrx.message);
        Ok(())
    }
}

struct LocalMessage {
    message: String,
}

impl Into<String> for LocalMessage {
    fn into(self) -> String {
        self.message
    }
}

impl From<String> for LocalMessage {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl Into<CodeBlockFeedback> for LocalMessage {
    fn into(self) -> CodeBlockFeedback {
        match self.message.as_str().trim() {
            "allow" => CodeBlockFeedback::AllowExecution,
            v => CodeBlockFeedback::DenyExecution {
                reason: v.to_string(),
            },
        }
    }
}

use autogen::text_chat::chat_user_agent::Message as ChatUserAgentMessage;

impl<'a> From<ChatUserAgentMessage<'a>> for LocalMessage {
    fn from(value: ChatUserAgentMessage<'a>) -> Self {
        let res = match value {
            ChatUserAgentMessage::Text { sender, message } => Self {
                message: format!("{}: {}", sender, message),
            },
            ChatUserAgentMessage::CollaborativeAgentResponse { sender, response } => match response
            {
                CollaborativeAgentResponse::Text(text) => Self {
                    message: format!("{}: {}", sender, text),
                },
                CollaborativeAgentResponse::CommentedCodeBlock(CommentedCodeBlock {
                    comment,
                    code_block,
                    request_execution,
                }) => Self {
                    message: format!(
                        "sender: {}, request_execution: {}\n comment:{}\n code_block: {}",
                        sender, request_execution, comment, code_block.code
                    ),
                },
            },
            ChatUserAgentMessage::CodeBlockFeedback {
                sender,
                comment,
                code_block,
            } => Self {
                message: format!(
                    "sender: {}, comment: {}, code_block: {}",
                    sender, comment, code_block.code
                ),
            },
            ChatUserAgentMessage::CodeBlockExecutionResult(code_block_execution_result) => {
                match code_block_execution_result {
                    CodeBlockExecutionResult::Success(info) => Self {
                        message: format!("code_block_execution_result: {:?}", info),
                    },
                    CodeBlockExecutionResult::Failure(info) => Self {
                        message: format!("code_block_execution_result: {:?}", info),
                    },
                }
            }
        };

        res
    }
}

struct LlmMock {
    request_index: usize,
}

#[async_trait]
impl ConsumerAgent for LlmMock {
    type Mrx = LocalMessage;
    type Error = Error;

    async fn receive_message(&mut self, _: LocalMessage) -> Result<(), Self::Error> {
        let response = match self.request_index {
            0 => r#"
                I hate python. Hope you like Rust!.
                ```rust
                    fn main() {
                        println!("Hello, world!");
                    }
                ```"#
                .to_string(),
            _ => "sorry, im out of ideas..".to_string(),
        };

        self.request_index += 1;

        Ok(())
    }
}

impl Into<CollaborativeAgentResponse> for LocalMessage {
    fn into(self) -> CollaborativeAgentResponse {
        todo!()
    }
}


fn main() {
    let user_agent = LocalUserAgent;

}

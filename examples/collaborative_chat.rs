use autogen::agent_traits::{ConsumerAgent, NamedAgent, ProducerAgent};
use autogen::text_chat::code::{CodeBlock, CodeBlockExecutionResult, CodeExecutor};
use autogen::text_chat::collaborative_agent::{CollaborativeAgentResponse, CommentedCodeBlock};

use autogen::text_chat::chat_user_agent::CodeBlockFeedback;

use async_std::io;

use tracing::{debug, info};

#[derive(Debug)]
enum Error {
    Io(io::Error),
}

#[derive(Clone)]
struct LocalUserAgent;

impl NamedAgent for LocalUserAgent {
    fn name(&self) -> &str {
        "LocalUserAgent"
    }
}

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

impl From<ChatUserAgentMessage> for LocalMessage {
    fn from(value: ChatUserAgentMessage) -> Self {
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
                        "sender: {},\nrequest_execution: {}\ncomment: {}\ncode_block: {}",
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
                    "You are asked for feedback on the code:\nsender: {},\ncomment: {},\ncode_block: {}\n\nIf you want to allow execution, type \"allow\". Otherwise, type the reason.",
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

use autogen::text_chat::collaborative_agent::Message as CollaborativeAgentMessage;

struct LlmMock {
    request_index: usize,
    denied_execution: Option<()>,
}

impl NamedAgent for LlmMock {
    fn name(&self) -> &str {
        "LlmMock"
    }
}

impl Default for LlmMock {
    fn default() -> Self {
        Self {
            request_index: 0,
            denied_execution: None,
        }
    }
}

impl ConsumerAgent for LlmMock {
    type Mrx = CollaborativeAgentMessage;
    type Error = Error;

    async fn receive_message(&mut self, m: Self::Mrx) -> Result<(), Self::Error> {
        info!("LlmMock received message {}", self.request_index);

        match m {
            CollaborativeAgentMessage::CodeExecutionDenied { .. } => {
                self.denied_execution = Some(())
            }
            CollaborativeAgentMessage::Text { .. }
            | CollaborativeAgentMessage::CodeExecutionResult { .. } => {}
        }

        Ok(())
    }
}

impl ProducerAgent for LlmMock {
    type Mtx = CollaborativeAgentResponse;
    type Error = Error;

    async fn send_message(&mut self) -> Result<Self::Mtx, Self::Error> {
        match self.denied_execution {
            Some(()) => Ok(CollaborativeAgentResponse::Text(
                "I'm a rouge agent! You will not get Python from me!".to_string(),
            )),
            None => {
                let response = match self.request_index {
                    0 => CollaborativeAgentResponse::CommentedCodeBlock(CommentedCodeBlock {
                        code_block: CodeBlock {
                            code: r#"
                                fn main() {
                                    println!("Hello, world!");
                                }
                            "#
                            .to_string(),
                            language: "rust".to_string(),
                        },
                        comment: "I hate python. Hope you like Rust!".to_string(),
                        request_execution: true,
                    }),
                    1 => CollaborativeAgentResponse::Text("Glad you liked it!".to_string()),
                    _ => CollaborativeAgentResponse::Text("sorry, im out of ideas..".to_string()),
                };
                self.request_index += 1;

                Ok(response)
            }
        }
    }
}

use autogen::text_chat::collaborative_chat::collaborative_chat;

struct LocalCodeExecutor;

impl CodeExecutor for LocalCodeExecutor {
    type Error = Error;

    async fn execute_code_block(
        &self,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockExecutionResult, Self::Error> {
        debug!("Enter execute_code_block");
        debug!("code_block: {:?}", code_block);

        Ok(CodeBlockExecutionResult::Success("".to_string()))
    }
}

use autogen::text_chat::collaborative_chat::SystemAgent;

struct LocalSystemAgent;

impl SystemAgent for LocalSystemAgent {
    fn initial_message(&self) -> String {
        "hello, this is a collaborative chat. You may only ask agent to write hello world in Python"
            .to_string()
    }
}

use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() {
    let user_agent = LocalUserAgent;
    let llm_mock = LlmMock::default();
    let system_agent = LocalSystemAgent;

    let executor = LocalCodeExecutor;
    let cancellation_token = CancellationToken::new();

    collaborative_chat(
        user_agent,
        llm_mock,
        system_agent,
        executor,
        cancellation_token,
    )
    .await
    .unwrap();
}

#![feature(trait_alias)]
#![feature(result_option_inspect)]
#![feature(async_iterator)]
#![feature(async_iter_from_iter)]

mod chat;
mod code_traits;
mod group_chat;
mod user_agent;

mod agent_traits;
use group_chat::scheduler::RoundRobin;
use group_chat::GroupChat;

use async_trait::async_trait;

use code_traits::{FencedCodeBlockExtractor, NativeCodeExecutor};

use user_agent::{ExecutionResponse, LocalUserAgent, Message, UserProxyAgentExecutorError};

use agent_traits::RespondingAgent;

struct LlmMock {
    request_index: usize,
}

#[async_trait]
impl RespondingAgent<String> for LlmMock {
    type Mtx = Message;
    type Error = std::io::Error;

    async fn receive_and_reply(&mut self, _: String) -> Result<Message, Self::Error> {
        let response = match self.request_index {
            0 => "Hello, I'm a helpful agent.".to_string(),
            1 => "```rust\nfn main() {\nprintln!(\"Hello, world!\");\n}\n```".to_string(),
            _ => panic!("Unexpected request"),
        };

        self.request_index += 1;

        Ok(Message::Text(response))
    }
}

#[async_trait]
impl RespondingAgent<Result<Vec<ExecutionResponse>, UserProxyAgentExecutorError>> for LlmMock {
    type Mtx = Message;
    type Error = std::io::Error;

    async fn receive_and_reply(
        &mut self,
        _: Result<Vec<ExecutionResponse>, UserProxyAgentExecutorError>,
    ) -> Result<Message, Self::Error> {
        Ok(Message::Text("nice..".to_string()))
    }
}

use chat::collaborative_chat;
use tracing::debug;

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )
    .unwrap();

    let user_agent = LocalUserAgent;
    let native_code_executor = NativeCodeExecutor;
    let code_extractor = FencedCodeBlockExtractor;

    let llm_mock = LlmMock { request_index: 0 };

    debug!("Starting chat");
    collaborative_chat(user_agent, code_extractor, native_code_executor, llm_mock)
        .await
        .unwrap();
}

use crate::agent_traits::Agent;

use async_trait::async_trait;
use futures::{Sink, Stream, StreamExt};

use crate::code_traits::UserCodeExecutor;

use crate::code_traits::CodeBlock;

use super::UserAgent;

pub struct UserProxyAgentExecutor<E>
where
    E: UserCodeExecutor<CodeBlock = CodeBlock>,
{
    executor: E,
    code_blocks: Vec<CodeBlock>,
}

impl<E> UserProxyAgentExecutor<E>
where
    E: UserCodeExecutor<CodeBlock = CodeBlock>,
{
    pub fn new(executor: E) -> Self {
        Self {
            executor,
            code_blocks: Vec::new(),
        }
    }
}

pub enum ExecutionResponse {
    Success,
    Error(String),
}

impl<Executor> UserProxyAgentExecutor<Executor>
where
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    <Executor as UserCodeExecutor>::Response: Send,
{
    /// User may choose what to do with an ExecutionResponse.
    /// e.g. when error is encountered, user may abort the execution of futher code blocks.
    pub fn run_code(&mut self) -> impl Stream<Item = ExecutionResponse> + Send + '_ {
        let stream2 = futures::stream::iter(self.code_blocks.drain(..)).then(|code_block| async {
            let mtx = self.executor.execute_code_block(code_block).await;
            mtx
        });

        stream2
    }
}

pub enum UserProxyAgentExecutorError {
    SendError,
}

pub enum Message {
    Text(String),
}

use crate::code_traits::CodeExtractor;

#[async_trait]
impl<Executor, Extractor> Agent<Message, ExecutionResponse>
    for (UserAgent, Extractor, UserProxyAgentExecutor<Executor>)
where
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    Extractor: CodeExtractor<Message, CodeBlock = CodeBlock> + Send,
    <Executor as UserCodeExecutor>::Response: Send,
{
    type Error = UserProxyAgentExecutorError;
    async fn receive(&mut self, message: Message) {
        let (user_agent, extractor, user_proxy_agent_executor) = self;

        //may be optimized to process while receiving. Now just collect all messages first.
    }

    async fn send(&mut self) -> Result<ExecutionResponse, Self::Error> {
        todo!()
    }
}

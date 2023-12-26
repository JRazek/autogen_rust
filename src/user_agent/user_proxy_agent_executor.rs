use crate::agent_traits::Agent;

use async_trait::async_trait;
use futures::{Sink, Stream, StreamExt};

use crate::code_traits::UserCodeExecutor;

use crate::code_traits::CodeBlock;

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

#[async_trait]
impl<Executor> Agent<CodeBlock, ExecutionResponse> for UserProxyAgentExecutor<Executor>
where
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    <Executor as UserCodeExecutor>::Response: Send,
{
    type Error = UserProxyAgentExecutorError;
    async fn receive(&mut self, stream: impl Stream<Item = CodeBlock> + Unpin + Send) {
        let blocks = stream.collect::<Vec<_>>().await;

        self.code_blocks.extend(blocks);
    }

    async fn send(
        &mut self,
        sink: impl Sink<ExecutionResponse> + Unpin + Send,
    ) -> Result<(), Self::Error> {
        let stream = self.run_code();
        stream
            .map(Ok)
            .forward(sink)
            .await
            .map_err(|_| UserProxyAgentExecutorError::SendError)
    }
}

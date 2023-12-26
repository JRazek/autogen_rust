use crate::agent_traits::Agent;

use super::agent_traits::{CodeExtractor, UserCodeExecutor};

use async_trait::async_trait;
use futures::{Sink, Stream, StreamExt};

pub struct UserProxyAgentExecutor<Executor, C>
where
    Executor: UserCodeExecutor<CodeBlock = C>,
{
    executor: Executor,
    code_blocks: Vec<C>,
}

impl<Executor, Mtx, C> UserProxyAgentExecutor<Executor, C>
where
    Executor: UserCodeExecutor<CodeBlock = C, Response = Mtx> + Send + Sync,
    Mtx: Send,
    C: Send + 'static,
    <Executor as UserCodeExecutor>::Response: Send,
{
    pub fn run_code(&mut self) -> impl Stream<Item = Mtx> + Send + '_ {
        let stream2 = futures::stream::iter(self.code_blocks.drain(..)).then(|code_block| async {
            let mtx = self.executor.execute_code_block(code_block).await;
            mtx
        });

        stream2
    }
}

#[async_trait]
impl<Executor, Mtx, C> Agent<C, Mtx> for UserProxyAgentExecutor<Executor, C>
where
    Executor: UserCodeExecutor<CodeBlock = C, Response = Mtx> + Send + Sync,
    Mtx: Send,
    C: Send + 'static,
    <Executor as UserCodeExecutor>::Response: Send,
{
    async fn receive(&mut self, stream: impl Stream<Item = C> + Unpin + Send) {
        let blocks = stream.collect::<Vec<_>>().await;

        self.code_blocks.extend(blocks);
    }

    async fn send(&mut self, sink: impl Sink<Mtx> + Unpin + Send) {
        let stream = self.run_code();
        stream.map(Ok).forward(sink).await;
    }
}


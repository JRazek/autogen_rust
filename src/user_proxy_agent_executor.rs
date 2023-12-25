use crate::agent_traits::Agent;

use super::agent_traits::{CodeExtractor, UserCodeExecutor};

use async_trait::async_trait;
use futures::{Sink, Stream, StreamExt};

pub struct UserProxyAgentExecutor<Extractor, Executor, M, C>
where
    Extractor: CodeExtractor<M, CodeBlock = C>,
    Executor: UserCodeExecutor<CodeBlock = C>,
{
    extractor: Extractor,
    executor: Executor,
    messages: Vec<M>,
}

impl<Extractor, Executor, Mrx, Mtx, C> UserProxyAgentExecutor<Extractor, Executor, Mrx, C>
where
    Extractor: CodeExtractor<Mrx, CodeBlock = C> + Send,
    Executor: UserCodeExecutor<CodeBlock = C, Response = Mtx> + Send + Sync,
    Mtx: Send,
    Mrx: Send,
    C: Send + 'static,
    <Executor as UserCodeExecutor>::Response: Send,
{
    pub fn run_code(&mut self) -> impl Stream<Item = Mtx> + Send + '_ {
        let code_blocks = self.extractor.extract_code_blocks(self.messages.drain(..));

        let stream2 = futures::stream::iter(code_blocks).then(|code_block| async {
            let mtx = self.executor.execute_code_block(code_block).await;
            mtx
        });

        stream2
    }
}

#[async_trait]
impl<Extractor, Executor, Mtx, Mrx, C> Agent<Mtx, Mrx>
    for UserProxyAgentExecutor<Extractor, Executor, Mtx, C>
where
    Extractor: CodeExtractor<Mtx, CodeBlock = C> + Send,
    Executor: UserCodeExecutor<CodeBlock = C> + Send,
    Mtx: Send,
{
    async fn receive(&mut self, stream: impl Stream<Item = Mtx> + Unpin + Send) {
        let messages = stream.collect::<Vec<_>>().await;
        self.messages.extend(messages);
    }

    async fn send(&mut self, sink: impl Sink<Mrx> + Unpin + Send) {}
}

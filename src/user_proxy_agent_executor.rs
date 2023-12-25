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

#[async_trait]
impl<Extractor, Executor, M, C> Agent<M, M> for UserProxyAgentExecutor<Extractor, Executor, M, C>
where
    Extractor: CodeExtractor<M, CodeBlock = C> + Send,
    Executor: UserCodeExecutor<CodeBlock = C> + Send,
    M: Send,
{
    async fn receive(&mut self, stream: impl Stream<Item = M> + Unpin + Send) {
        let messages = stream.collect::<Vec<_>>().await;
        self.messages.extend(messages);
    }

    async fn send(&mut self, sink: impl Sink<M> + Unpin + Send) {}
}

use futures::{Sink, Stream};

use async_trait::async_trait;

#[async_trait]
pub trait Agent<Mtx, Mrx> {
    async fn receive(&mut self, stream: impl Stream<Item = Mtx> + Unpin + Send);
    async fn send(&mut self, sink: impl Sink<Mrx> + Unpin + Send);
}

pub trait CodeExtractor<M> {
    type CodeBlock;
    fn extract_code_blocks(&self, messages: impl Iterator<Item = M>) -> Vec<Self::CodeBlock>;
}

#[async_trait]
pub trait UserCodeExecutor {
    type CodeBlock;
    type Response;

    async fn execute_code_block(&self, code_block: Self::CodeBlock) -> Self::Response;
}

//#[async_trait]
//pub trait UserAgent<M>: Agent<M> {
//    type UserProxyAgent: Agent<M>;
//}

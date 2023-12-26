use async_trait::async_trait;

pub mod fenced_code_block_extractor;

pub mod native_code_executor;

pub use fenced_code_block_extractor::*;

pub use native_code_executor::*;

pub trait CodeExtractor<M> {
    type CodeBlock;
    fn extract_code_blocks(&self, _: M) -> Vec<Self::CodeBlock>;
}

#[async_trait]
pub trait UserCodeExecutor {
    type CodeBlock;
    type Response;

    async fn execute_code_block(&self, code_block: Self::CodeBlock) -> Self::Response;
}

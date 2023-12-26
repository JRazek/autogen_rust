use async_trait::async_trait;

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

pub struct FencedCodeBlockExtractor;

impl CodeExtractor<String> for FencedCodeBlockExtractor {
    type CodeBlock = String;

    fn extract_code_blocks(&self, messages: impl Iterator<Item = String>) -> Vec<Self::CodeBlock> {
        let string = messages.collect::<Vec<_>>().join("");

        todo!()
    }
}

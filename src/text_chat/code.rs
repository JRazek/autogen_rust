use async_trait::async_trait;

#[derive(Clone)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}

pub enum CodeBlockExecutionResult {
    Success(String),
    Failure(String),
}

#[async_trait]
pub trait CodeExecutor {
    type Error;

    async fn execute_code_block(
        &self,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockExecutionResult, Self::Error>;
}

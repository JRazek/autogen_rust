/// Language is kept as a string for now.
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}

#[derive(Debug, Clone)]
pub enum CodeBlockExecutionResult {
    Success(String),
    Failure(String),
}

/// One may define a custom executor for a specific platform.
/// For example, one may define executor that runs on local machine,
/// remote machine, or a docker container.
///
/// This is very similar to the UserProxyAgent, however since its a trait, it does not care about
/// any specific implementation and configs as the Python version does.

pub trait CodeExecutor {
    type Error;

    async fn execute_code_block(
        &self,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockExecutionResult, Self::Error>;
}

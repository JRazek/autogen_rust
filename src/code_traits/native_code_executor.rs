use async_trait::async_trait;
use tracing::info;

use super::{CodeBlock, Language, UserCodeExecutor};

pub struct NativeCodeExecutor;

use pyo3::prelude::*;

#[async_trait]
impl UserCodeExecutor for NativeCodeExecutor {
    type CodeBlock = CodeBlock;
    type Response = PyResult<()>;

    async fn execute_code_block(&self, code_block: Self::CodeBlock) -> Self::Response {
        match code_block.language {
            Language::Python => {
                info!("Executing python code block: {}", code_block.code);
            }
            Language::Rust => {
                info!("Compiling and running rust code block: {}", code_block.code);
            }
        }

        Ok(())
    }
}

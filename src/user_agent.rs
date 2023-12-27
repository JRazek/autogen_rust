#![allow(dead_code)]

use super::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};

mod user_proxy_agent_executor;

pub use user_proxy_agent_executor::*;

use async_trait::async_trait;

/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
pub struct UserAgent;

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
}

impl UserAgent {
    fn with_user_proxy<Extractor, Executor>(
        _user_proxy_agent_executor: UserProxyAgentExecutor<Executor>,
    ) -> Self
    where
        Extractor: CodeExtractor<String, CodeBlock = CodeBlock>,
        Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = CodeBlock>,
    {
        todo!()
    }

    pub async fn request_code_block_feedback(&self, code_block: &CodeBlock) -> CodeBlockFeedback {
        todo!()
    }
}

use crate::agent_traits::Agent;

#[async_trait]
impl Agent<String, String> for UserAgent {
    type Error = ();

    async fn receive(&mut self, _message: String) {
        todo!()
    }

    async fn send(&mut self) -> Result<String, Self::Error> {
        todo!()
    }
}

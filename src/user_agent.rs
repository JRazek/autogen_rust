#![allow(dead_code)]

use super::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};

mod user_proxy_agent_executor;

pub use user_proxy_agent_executor::*;

/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
pub struct UserAgent {}

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
}

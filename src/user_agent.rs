#![allow(dead_code)]

use crate::user_proxy_agent_executor::UserProxyAgentExecutor;

use super::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};

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

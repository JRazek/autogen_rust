use crate::agent_traits::RespondingAgent;

use async_trait::async_trait;

use crate::code_traits::UserCodeExecutor;

use crate::code_traits::CodeBlock;

use super::{RequestCodeFeedback, UserAgent};

use crate::user_agent::CodeBlockFeedback;

#[derive(Debug)]
pub enum ExecutionResponse {
    Success,
    ExecutionError(String),
}

#[derive(Debug)]
pub enum UserProxyAgentExecutorError {
    DeniedExecution(String),
}

#[derive(Debug)]
pub enum Message {
    Text(String),
}

use crate::code_traits::CodeExtractor;

use tracing::debug;

#[async_trait]
impl<UA, Executor, Extractor> RespondingAgent<Message> for (UA, Extractor, Executor)
where
    UA: UserAgent<String, Mtx = String> + Send + Sync,
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    Extractor: CodeExtractor<String, CodeBlock = CodeBlock> + Send,
    <Executor as UserCodeExecutor>::Response: Send,
    <UA as UserAgent<String>>::Error: std::error::Error,
{
    type Mtx = Vec<ExecutionResponse>;
    type Error = UserProxyAgentExecutorError;
    async fn receive_and_reply(
        &mut self,
        message: Message,
    ) -> Result<Vec<ExecutionResponse>, Self::Error> {
        let (user_agent, extractor, user_proxy_agent_executor) = self;

        debug!("called receive_and_reply in UserProxyAgentExecutor");

        match message {
            Message::Text(message) => {
                let code_blocks = extractor.extract_code_blocks(message);

                debug!("decoded following code blocks: {:?}", code_blocks);

                let mut results = vec![];

                for code_block in code_blocks {
                    let feedback = user_agent
                        .request_code_block_feedback(&code_block)
                        .await
                        .unwrap();
                    match feedback {
                        CodeBlockFeedback::AllowExecution => {
                            let execution_response = user_proxy_agent_executor
                                .execute_code_block(&code_block)
                                .await;

                            results.push(execution_response);
                        }
                        CodeBlockFeedback::DenyExecution { reason } => {
                            return Err(UserProxyAgentExecutorError::DeniedExecution(reason));
                        }
                    }
                }

                Ok(results)
            }
            _ => unimplemented!(),
        }
    }
}

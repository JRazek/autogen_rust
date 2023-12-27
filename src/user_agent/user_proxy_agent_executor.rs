use crate::agent_traits::{ConsumerAgent, RespondingAgent};

use async_trait::async_trait;

use crate::code_traits::UserCodeExecutor;

use crate::code_traits::CodeBlock;

use super::{RequestCodeFeedback, UserAgent};

use crate::user_agent::CodeBlockFeedback;

pub struct UserProxyAgentExecutor<E>
where
    E: UserCodeExecutor<CodeBlock = CodeBlock>,
{
    pub executor: E,
}

impl<E> UserProxyAgentExecutor<E>
where
    E: UserCodeExecutor<CodeBlock = CodeBlock>,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

enum ExecutionResponse {
    Success,
    Error(String),
}

pub enum UserProxyAgentExecutorError {
    SendError,
    DeniedExecution(String),
    ExecutionError(String),
}

pub enum Message {
    Text(String),
}

use crate::code_traits::CodeExtractor;

#[async_trait]
impl<UA, Executor, Extractor> RespondingAgent<Message>
    for (UA, Extractor, UserProxyAgentExecutor<Executor>)
where
    UA: UserAgent<String, Mtx = String, Error = ()> + Send + Sync,
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    Extractor: CodeExtractor<Message, CodeBlock = CodeBlock> + Send,
    <Executor as UserCodeExecutor>::Response: Send,
{
    type Mtx = ();
    type Error = UserProxyAgentExecutorError;
    async fn receive_and_reply(&mut self, message: Message) -> Result<(), Self::Error> {
        let (user_agent, extractor, user_proxy_agent_executor) = self;

        let code_blocks = extractor.extract_code_blocks(message);

        for code_block in code_blocks {
            let feedback = user_agent
                .request_code_block_feedback(&code_block)
                .await
                .unwrap();
            match feedback {
                CodeBlockFeedback::AllowExecution => {
                    let execution_response = user_proxy_agent_executor
                        .executor
                        .execute_code_block(&code_block)
                        .await;
                    match execution_response {
                        ExecutionResponse::Success => {
                            //send success message
                        }
                        ExecutionResponse::Error(e) => {
                            user_agent.receive(e.clone()).await.unwrap();
                            return Err(UserProxyAgentExecutorError::ExecutionError(e));
                        }
                    }
                }
                CodeBlockFeedback::DenyExecution { reason } => {
                    return Err(UserProxyAgentExecutorError::DeniedExecution(reason));
                }
            }
        }

        Ok(())
    }
}

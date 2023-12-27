use crate::agent_traits::Agent;

use async_trait::async_trait;
use futures::{Sink, Stream, StreamExt};

use crate::code_traits::UserCodeExecutor;

use crate::code_traits::CodeBlock;

use super::UserAgent;

use crate::user_agent::CodeBlockFeedback;
use futures::stream::iter as async_iter;

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

pub enum ExecutionResponse {
    Success,
    Error(String),
}

pub enum UserProxyAgentExecutorError {
    SendError,
}

pub enum Message {
    Text(String),
}

use crate::code_traits::CodeExtractor;

#[async_trait]
impl<Executor, Extractor> Agent<Message, ExecutionResponse>
    for (UserAgent, Extractor, UserProxyAgentExecutor<Executor>)
where
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    Extractor: CodeExtractor<Message, CodeBlock = CodeBlock> + Send,
    <Executor as UserCodeExecutor>::Response: Send,
{
    type Error = UserProxyAgentExecutorError;
    async fn receive(&mut self, message: Message) {
        let (user_agent, extractor, user_proxy_agent_executor) = self;

        let code_blocks = extractor.extract_code_blocks(message);

        for code_block in code_blocks {
            let feedback = user_agent.request_code_block_feedback(&code_block).await;
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
                            user_agent.receive(e).await;
                        }
                    }
                }
                CodeBlockFeedback::DenyExecution { reason } => {
                    //TODO inform the sender about the reason
                    break;
                }
            }
        }

        //may be optimized to process while receiving. Now just collect all messages first.
    }

    async fn reply(&mut self) -> Result<ExecutionResponse, Self::Error> {
        todo!()
    }
}

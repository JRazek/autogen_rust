#![allow(dead_code)]

use crate::agent_traits::RespondingAgent;
use crate::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};
use crate::user_agent::{ExecutionResponse, Message, UserAgent, UserProxyAgentExecutorError};

pub async fn collaborative_chat<UA, Extractor, Executor, A>(
    user_agent: UA,
    code_extractor: Extractor,
    user_code_executor: Executor,
    mut conversational_agent: A,
) -> Result<(), Box<dyn std::error::Error>>
where
    UA: UserAgent<String, Mtx = String> + Send + Sync,
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    Extractor: CodeExtractor<String, CodeBlock = CodeBlock> + Send,
    <Executor as UserCodeExecutor>::Response: Send,
    <UA as UserAgent<String>>::Error: std::error::Error + 'static,

    A: RespondingAgent<String, Mtx = Message>,
    A: RespondingAgent<Result<Vec<ExecutionResponse>, UserProxyAgentExecutorError>, Mtx = Message>,
    <A as RespondingAgent<Result<Vec<ExecutionResponse>, UserProxyAgentExecutorError>>>::Error:
        std::error::Error + 'static,
    <A as RespondingAgent<String>>::Error: std::error::Error + 'static,
{
    let mut code_executor = (user_agent, code_extractor, user_code_executor);
    loop {
        let user_agent = &mut code_executor.0;
        let prompt = user_agent.receive_from_user().await?;

        let prompt_response = conversational_agent.receive_and_reply(prompt).await?;

        let execution_response = code_executor.receive_and_reply(prompt_response).await;

        conversational_agent
            .receive_and_reply(execution_response)
            .await?;
    }
}

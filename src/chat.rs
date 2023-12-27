#![allow(dead_code)]

use crate::agent_traits::{ConsumerAgent, ProducerAgent, RespondingAgent};
use crate::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};
use crate::user_agent::{ExecutionResponse, Message, UserAgent, UserProxyAgentExecutorError};

pub async fn collaborative_chat<UA, Extractor, Executor, A>(
    mut execution_agent: (UA, Extractor, Executor),
    mut conversational_agent: A,
) -> Result<(), Box<dyn std::error::Error>>
where
    UA: UserAgent<String, Mtx = String> + Send + Sync,
    Executor: UserCodeExecutor<CodeBlock = CodeBlock, Response = ExecutionResponse> + Send + Sync,
    Extractor: CodeExtractor<Message, CodeBlock = CodeBlock> + Send,
    <Executor as UserCodeExecutor>::Response: Send,
    <UA as UserAgent<String>>::Error: std::error::Error + 'static,

    A: RespondingAgent<String, Mtx = Message>,
    A: RespondingAgent<Result<Vec<ExecutionResponse>, UserProxyAgentExecutorError>, Mtx = Message>,
    <A as RespondingAgent<Result<Vec<ExecutionResponse>, UserProxyAgentExecutorError>>>::Error:
        std::error::Error + 'static,
    <A as RespondingAgent<String>>::Error: std::error::Error + 'static,
{
    loop {
        let user_agent = &mut execution_agent.0;
        let prompt = user_agent.receive_from_user().await?;

        let prompt_response = conversational_agent.receive_and_reply(prompt).await?;

        let execution_response = execution_agent.receive_and_reply(prompt_response).await;

        conversational_agent
            .receive_and_reply(execution_response)
            .await?;
    }
}

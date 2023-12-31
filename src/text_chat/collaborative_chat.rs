use super::chat_user_agent::CodeBlockFeedback;
use super::collaborative_agent::{CollaborativeAgent, CollaborativeAgentResponse};
use crate::agent_traits::NamedAgent;

use super::chat_user_agent::ChatUserAgent;

use super::code::CodeExecutor;

use tracing::debug;

use super::collaborative_chat_error::CollaborativeChatError;

use tokio_util::sync::CancellationToken;

pub trait SystemAgent {
    fn initial_message(&self) -> String;
}

impl<SA: SystemAgent> NamedAgent for SA {
    fn name(&self) -> &str {
        "system"
    }
}

/// Regarding Assignment requirement to provide grouping chat for collaboration.
/// Even though this accepts a single collaborative agent, it is not a problem to create a specific implementation of an agent that would accumulate multiple collaborative agents.

/// This function is the main entry point for the collaborative chat.
pub async fn collaborative_chat<UA, CA, SA, E>(
    mut user_agent: UA,
    mut collaborative_agent: CA,
    system_agent: SA,
    executor: E,
    cancellation_token: CancellationToken,
) -> Result<(), CollaborativeChatError<UA, CA, E>>
where
    UA: ChatUserAgent,
    UA: NamedAgent,

    CA: CollaborativeAgent,
    CA: NamedAgent,

    SA: SystemAgent,

    E: CodeExecutor,
{
    debug!("starting chat..");

    debug!("sending welcome message..");
    let ua_response = user_agent
        .receive_and_reply(
            system_agent.name().to_string(),
            system_agent.initial_message(),
        )
        .await
        .map_err(CollaborativeChatError::ChatUserAgent)?;

    debug!("sending user message to collaborative_agent..");
    let mut ca_response = collaborative_agent
        .receive_and_reply(user_agent.name().to_string(), ua_response.clone())
        .await
        .map_err(CollaborativeChatError::CollaborativeAgent)?;

    while !cancellation_token.is_cancelled() {
        match ca_response {
            CollaborativeAgentResponse::CommentedCodeBlock(ref commented_code_block) => {
                user_agent
                    .silent_receive_collaborative_agent_response(
                        collaborative_agent.name().to_string(),
                        ca_response.clone(),
                    )
                    .await
                    .map_err(CollaborativeChatError::ChatUserAgent)?;

                match commented_code_block.request_execution {
                    true => {
                        debug!("code execution requested. Sending code block to user_agent..");

                        let ua_feedback = user_agent
                            .request_code_block_feedback(
                                collaborative_agent.name().to_string(),
                                commented_code_block.comment.clone(),
                                commented_code_block.code_block.clone(),
                            )
                            .await
                            .map_err(CollaborativeChatError::ChatUserAgent)?;

                        match ua_feedback {
                            CodeBlockFeedback::AllowExecution => {
                                debug!("code execution allowed. Executing code..");

                                let execution_result = executor
                                    .execute_code_block(&commented_code_block.code_block)
                                    .await
                                    .map_err(CollaborativeChatError::CodeExecutor)?;

                                debug!("sending execution result to user_agent..");
                                user_agent
                                    .receive_code_execution_result(execution_result.clone())
                                    .await
                                    .map_err(CollaborativeChatError::ChatUserAgent)?;

                                debug!("sending execution result to collaborative_agent..");

                                ca_response = collaborative_agent
                                    .receive_code_and_reply_to_execution_result(execution_result)
                                    .await
                                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
                            }
                            CodeBlockFeedback::DenyExecution { reason } => {
                                debug!("code execution denied. Sending reason to collaborative_agent..");

                                ca_response = collaborative_agent
                                    .deny_code_block_execution(
                                        commented_code_block.code_block.clone(),
                                        reason,
                                    )
                                    .await
                                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
                            }
                        }
                    }
                    false => {
                        debug!("code execution not requested. Skipping feedback phase..");
                        todo!()
                    }
                }
            }
            CollaborativeAgentResponse::Text(text) => {
                debug!("sending text to user_agent..");
                let ua_response = user_agent
                    .receive_and_reply(collaborative_agent.name().to_string(), text)
                    .await
                    .map_err(CollaborativeChatError::ChatUserAgent)?;

                debug!("sending user_agent response to collaborative_agent..");
                ca_response = collaborative_agent
                    .receive_and_reply(user_agent.name().to_string(), ua_response)
                    .await
                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
            }
        }
    }

    Ok(())
}

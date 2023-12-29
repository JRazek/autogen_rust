use super::chat_user_agent::CodeBlockFeedback;
use super::collaborative_agent::{CollaborativeAgent, CollaborativeAgentResponse};
use crate::agent_traits::NamedAgent;

use super::chat_user_agent::ChatUserAgent;

use super::code::CodeExecutor;

use tracing::debug;

use super::collaborative_chat_error::CollaborativeChatError;

use tokio_util::sync::CancellationToken;

pub async fn collaborative_chat<UA, CA, E>(
    mut user_agent: UA,
    mut collaborative_agent: CA,
    executor: E,
    cancellation_token: CancellationToken,
) -> Result<(), CollaborativeChatError<UA, CA, E>>
where
    UA: ChatUserAgent,
    UA: NamedAgent,

    CA: CollaborativeAgent,
    CA: NamedAgent,

    E: CodeExecutor,
{
    debug!("starting chat..");

    debug!("sending welcome message..");
    let ua_response = user_agent
        .receive_and_reply(
            "system",
            "hello, this is a collaborative chat. You may ask collaborative_agent for help.",
        )
        .await
        .map_err(CollaborativeChatError::ChatUserAgent)?;

    debug!("sending user message to collaborative_agent..");
    let mut ca_response = collaborative_agent
        .receive_and_reply(user_agent.name(), &ua_response)
        .await
        .map_err(CollaborativeChatError::CollaborativeAgent)?;

    while !cancellation_token.is_cancelled() {
        match ca_response {
            CollaborativeAgentResponse::CommentedCodeBlock(ref commented_code_block) => {
                user_agent
                    .silent_receive_collaborative_agent_response(collaborative_agent.name(), &ca_response)
                    .await
                    .map_err(CollaborativeChatError::ChatUserAgent)?;

                match commented_code_block.request_execution {
                    true => {
                        debug!("code execution requested. Sending code block to user_agent..");

                        let ua_feedback = user_agent
                            .request_code_block_feedback(
                                collaborative_agent.name(),
                                &commented_code_block.comment,
                                &commented_code_block.code_block,
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
                                    .receive_code_execution_result(&execution_result)
                                    .await
                                    .map_err(CollaborativeChatError::ChatUserAgent)?;

                                debug!("sending execution result to collaborative_agent..");

                                ca_response = collaborative_agent
                                    .receive_code_and_reply_to_execution_result(&execution_result)
                                    .await
                                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
                            }
                            CodeBlockFeedback::DenyExecution { reason } => {
                                debug!("code execution denied. Sending reason to collaborative_agent..");

                                ca_response = collaborative_agent
                                    .deny_code_block_execution(
                                        &commented_code_block.code_block,
                                        &reason,
                                    )
                                    .await
                                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
                            }
                        }
                    }
                    false => {
                        debug!("code execution not requested. Skipping feedback phase..");
                    }
                }
            }
            CollaborativeAgentResponse::Text(ref text) => {
                debug!("sending text to user_agent..");
                let ua_response = user_agent
                    .receive_and_reply(collaborative_agent.name(), &text)
                    .await
                    .map_err(CollaborativeChatError::ChatUserAgent)?;

                debug!("sending user_agent response to collaborative_agent..");
                ca_response = collaborative_agent
                    .receive_and_reply(user_agent.name(), &ua_response)
                    .await
                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
            }
        }
    }

    Ok(())
}

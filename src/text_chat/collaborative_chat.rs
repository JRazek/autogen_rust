use super::chat_user_agent::{CodeBlockFeedback, RequestCodeFeedback};
use super::collaborative_agent::{CollaborativeAgent, CollaborativeAgentResponse};
use crate::agent_traits::{ConsumerAgent, NamedAgent, RespondingAgent};

use crate::user_agent::{RespondingAgentError, UserAgent};

use tracing::debug;

use super::collaborative_chat_error::CollaborativeChatError;

#[derive(Clone)]
pub struct UserTextMessage {
    pub sender: String,
    pub message: String,
}

pub async fn collaborative_chat<UA, CA>(
    mut user_agent: UA,
    mut collaborative_agent: CA,
) -> Result<(), CollaborativeChatError<UA, CA>>
where
    UA: RespondingAgent<Mrx = UserTextMessage, Mtx = UserTextMessage> + Send + Sync + 'static,
    UA: ConsumerAgent<Mrx = CollaborativeAgentResponse>,
    UA: RequestCodeFeedback,
    UA: NamedAgent,

    CA: CollaborativeAgent,
    CA: NamedAgent,
{
    debug!("starting chat..");

    debug!("sending welcome message..");
    let ua_response = user_agent
        .receive_and_reply(UserTextMessage {
            sender: "system".to_string(),
            message:
                "hello, this is a collaborative chat. You may ask collaborative_agent for help."
                    .to_string(),
        })
        .await
        .map_err(CollaborativeChatError::RespondingAgent)?;

    let mut ca_response = collaborative_agent
        .receive_and_reply(ua_response)
        .await
        .map_err(CollaborativeChatError::CollaborativeAgent)?;

    loop {
        debug!("sending user message to collaborative_agent..");

        match ca_response {
            CollaborativeAgentResponse::CommentedCodeBlock(ref commented_code_block) => {
                user_agent
                    .receive(ca_response.clone())
                    .await
                    .map_err(CollaborativeChatError::ConsumerAgent)?;

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
                            .map_err(CollaborativeChatError::RequestCodeFeedback)?;
                        match ua_feedback {
                            CodeBlockFeedback::AllowExecution => {
                                debug!("code execution allowed. Executing code..");
                                break;
                            }
                            CodeBlockFeedback::DenyExecution { reason } => {
                                debug!(
                                "code execution denied. Sending reason to collaborative_agent.."
                            );

                                ca_response = collaborative_agent
                                    .deny_code_block_execution(
                                        &commented_code_block.code_block,
                                        reason,
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
                    .receive_and_reply(UserTextMessage {
                        sender: collaborative_agent.name(),
                        message: text.clone(),
                    })
                    .await
                    .map_err(CollaborativeChatError::RespondingAgent)?;

                debug!("sending user_agent response to collaborative_agent..");
                ca_response = collaborative_agent
                    .receive_and_reply(ua_response)
                    .await
                    .map_err(CollaborativeChatError::CollaborativeAgent)?;
            }
        }
    }

    Ok(())
}

use super::collaborative_agent::CommentedCodeBlock;

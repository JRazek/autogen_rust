#![allow(dead_code)]

use super::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};

mod local_user_agent;
mod user_proxy_agent_executor;

pub use user_proxy_agent_executor::*;

use async_trait::async_trait;

use crate::agent_traits::{Agent, ConsumerAgent, RespondingAgent};

//make it as a trait
/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
pub struct UserAgent;

#[async_trait]
pub trait UserAgent2<Mrx, Mtx> {
    type Error;

    async fn send_to_user(&self, message: Mrx) -> Result<(), Self::Error>;
    async fn receive_from_user(&self) -> Result<Mtx, Self::Error>;
}

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
}

#[async_trait]
pub trait RequestCodeFeedback {
    type Error;
    async fn request_code_block_feedback(
        &self,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error>;
}

#[async_trait]
impl<U: UserAgent2<String, String, Error = E> + Sync, E> RequestCodeFeedback for U {
    type Error = E;

    async fn request_code_block_feedback(
        &self,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, E> {
        let message = format!(
            r#"You are asked for permission to compile/execute the following code block:\n{:?}"#,
            code_block
        );

        self.send_to_user(message).await?;

        let mut response = "".to_string();

        while response != "y" && response != "n" {
            self.send_to_user("Please enter y or n".to_string()).await?;
            let response = self.receive_from_user().await?;
        }

        match response.as_str() {
            "y" => Ok(CodeBlockFeedback::AllowExecution),
            "n" => {
                self.send_to_user("Please enter the reason for denying execution".to_string())
                    .await?;

                let reason = self.receive_from_user().await?;

                Ok(CodeBlockFeedback::DenyExecution { reason })
            }
            _ => unreachable!(),
        }
    }
}

#[async_trait]
impl<A> RespondingAgent<String, String> for A
where
    A: UserAgent2<String, String> + Send + Sync,
{
    type Error = A::Error;

    async fn receive_and_reply(&mut self, message: String) -> Result<String, Self::Error> {
        self.send_to_user(message).await?;
        self.receive_from_user().await
    }
}

#[async_trait]
impl<A> ConsumerAgent<String> for A
where
    A: UserAgent2<String, String> + Send + Sync,
{
    type Error = A::Error;

    async fn receive(&mut self, message: String) -> Result<(), Self::Error> {
        self.send_to_user(message).await
    }
}

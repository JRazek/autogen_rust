#![allow(dead_code)]

use super::code_traits::CodeBlock;

mod local_user_agent;
mod user_proxy_agent_executor;

pub use user_proxy_agent_executor::*;

use async_trait::async_trait;

use crate::agent_traits::{ConsumerAgent, RespondingAgent};

#[async_trait]
pub trait UserAgent2<Mrx> {
    type Error;
    type Mtx;

    async fn send_to_user(&mut self, message: Mrx) -> Result<(), Self::Error>;
    async fn receive_from_user(&mut self) -> Result<Self::Mtx, Self::Error>;
}

pub enum CodeBlockFeedback {
    AllowExecution,
    DenyExecution { reason: String },
}

#[async_trait]
pub trait RequestCodeFeedback {
    type Error;
    async fn request_code_block_feedback(
        &mut self,
        code_block: &CodeBlock,
    ) -> Result<CodeBlockFeedback, Self::Error>;
}

#[async_trait]
impl<A, Mrx, Mtx> RespondingAgent<Mrx, Mtx> for A
where
    A: UserAgent2<Mrx, Mtx = Mtx> + Send + Sync,
    Mrx: Send + 'static,
    Mtx: Send,
{
    type Error = A::Error;

    async fn receive_and_reply(&mut self, message: Mrx) -> Result<Mtx, Self::Error> {
        self.send_to_user(message).await?;
        self.receive_from_user().await
    }
}

#[async_trait]
impl<A, Mrx> ConsumerAgent<Mrx> for A
where
    A: UserAgent2<Mrx> + Send + Sync,
    Mrx: Send + 'static,
{
    type Error = A::Error;

    async fn receive(&mut self, message: Mrx) -> Result<(), Self::Error> {
        self.send_to_user(message).await
    }
}

#[async_trait]
impl<U, E> RequestCodeFeedback for U
where
    U: UserAgent2<String, Mtx = String, Error = E> + Send + Sync,
{
    type Error = E;

    async fn request_code_block_feedback(
        &mut self,
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
            response = self.receive_from_user().await?;
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

#[cfg(test)]
mod tests {

    use super::*;

    struct UserAgentMock {
        i: i32,
    }

    impl Default for UserAgentMock {
        fn default() -> Self {
            Self { i: 0 }
        }
    }

    #[async_trait]
    impl UserAgent2<String> for UserAgentMock {
        type Error = ();
        type Mtx = String;

        async fn send_to_user(&mut self, message: String) -> Result<(), Self::Error> {
            println!("{}", message);
            Ok(())
        }

        async fn receive_from_user(&mut self) -> Result<String, Self::Error> {
            let response = match self.i {
                0 => "n".to_string(),
                1 => "this is the reason".to_string(),
                _ => unreachable!(),
            };

            self.i += 1;

            Ok(response)
        }
    }

    #[tokio::test]
    async fn test_user_agent() {
        let mut user_agnet = UserAgentMock::default();

        let code_block = CodeBlock {
            code: "println!(\"Hello World!\");".to_string(),
            language: crate::code_traits::Language::Rust,
        };

        let feedback = user_agnet
            .request_code_block_feedback(&code_block)
            .await
            .unwrap();

        match feedback {
            CodeBlockFeedback::DenyExecution { reason } => {
                assert_eq!(reason, "this is the reason".to_string());
            }
            _ => unreachable!(),
        }
    }
}

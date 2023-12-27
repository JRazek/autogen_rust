#![allow(dead_code)]

use crate::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};

use super::UserAgent2;

use async_trait::async_trait;

use async_std::io;

//make it as a trait
/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
pub struct LocalUserAgent;

#[async_trait]
impl UserAgent2<String, String> for LocalUserAgent {
    type Error = ();

    async fn send_to_user(&self, message: String) -> Result<(), Self::Error> {
        println!("user received a message: {}", message);
        Ok(())
    }

    ///Reads a line from stdin
    async fn receive_from_user(&self) -> Result<String, Self::Error> {
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).await?;

        Ok(line)
    }
}

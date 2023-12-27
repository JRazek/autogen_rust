#![allow(dead_code)]

use crate::code_traits::{CodeBlock, CodeExtractor, UserCodeExecutor};

use super::UserAgent;

use async_trait::async_trait;

use async_std::io;

//make it as a trait
/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
pub struct LocalUserAgent;

#[async_trait]
impl UserAgent<String> for LocalUserAgent {
    type Error = io::Error;
    type Mtx = String;

    async fn send_to_user(&mut self, message: String) -> Result<(), Self::Error> {
        println!("user received a message: {}", message);
        Ok(())
    }

    ///Reads a line from stdin
    async fn receive_from_user(&mut self) -> Result<String, Self::Error> {
        let stdin = io::stdin();
        let mut line = String::new();
        stdin.read_line(&mut line).await?;

        Ok(line)
    }
}
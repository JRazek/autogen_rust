use std::ops::DerefMut;
use std::pin::Pin;

use futures::channel::mpsc::{SendError, Sender};
use futures::{Sink, SinkExt, Stream, StreamExt};

use crate::agent_traits::Agent;
use crate::chat::MessageContent;

use async_std::io::{stdin, BufReader};
use async_std::io::{Lines, Stdin};

type StdinLines = Lines<BufReader<Stdin>>;

use super::chat::ChatMessage;

#[derive(Clone)]
pub struct UserAgent;

impl Agent<ChatMessage> for UserAgent {
    type Proxy = UserAgentProxy;

    fn take_turn(&self, chat_history: impl IntoIterator<Item = ChatMessage>) -> Self::Proxy {
        use async_std::io::prelude::BufReadExt;

        let mut lines_stream = BufReader::new(stdin()).lines();

        let (tx, rx) = futures::channel::mpsc::channel(1);

        tokio::spawn(rx.for_each(|msg| async move {
            println!("UserAgentProxy received message: {:?}", msg);
        }));

        UserAgentProxy {
            rx: lines_stream,
            tx,
        }
    }
}

pub struct UserAgentProxy {
    rx: StdinLines,
    tx: Sender<ChatMessage>,
}

impl Stream for UserAgentProxy {
    type Item = ChatMessage;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx).map(|opt| match opt {
            Some(Ok(line)) => Some(ChatMessage {
                sender: "user".to_string(),
                message: MessageContent::Text(line),
            }),
            _ => None,
        })
    }
}

impl Sink<ChatMessage> for UserAgentProxy
where
    for<'a> Pin<&'a mut UserAgentProxy>: DerefMut<Target = Self>,
{
    type Error = SendError;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_ready_unpin(cx)
    }

    fn start_send(
        mut self: std::pin::Pin<&mut Self>,
        item: ChatMessage,
    ) -> Result<(), Self::Error> {
        self.tx.start_send_unpin(item)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_flush_unpin(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_close_unpin(cx)
    }
}

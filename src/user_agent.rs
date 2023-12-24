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
    type ProxyStream = UserAgentProxyStream;
    type ProxySink = UserAgentProxySink;

    fn take_turn(&self, _chat_history: impl IntoIterator<Item = ChatMessage>) -> Self::ProxyStream {
        use async_std::io::prelude::BufReadExt;

        let mut lines_stream = BufReader::new(stdin()).lines();

        UserAgentProxyStream { rx: lines_stream }
    }

    fn receive_turn(&self, chat_history: impl IntoIterator<Item = ChatMessage>) -> Self::ProxySink {
        todo!()
    }
}

pub struct UserAgentProxyStream {
    rx: StdinLines,
}

pub struct UserAgentProxySink {
    tx: Sender<ChatMessage>,
}

impl Stream for UserAgentProxyStream {
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

impl Sink<ChatMessage> for UserAgentProxySink
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

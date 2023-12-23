use std::ops::DerefMut;
use std::pin::Pin;

use futures::{Sink, SinkExt, Stream, StreamExt};

use crate::chat::MessageContent;

use async_std::io::BufReader;
use async_std::io::{Lines, Stdin};

type StdinLines = Lines<BufReader<Stdin>>;

use super::chat::ChatMessage as Message;

pub struct UserAgentProxy<T> {
    rx: StdinLines,
    tx: T,
}

impl<T> UserAgentProxy<T> {}

impl<S> Stream for UserAgentProxy<S>
where
    for<'a> Pin<&'a mut UserAgentProxy<S>>: DerefMut<Target = Self>,
{
    type Item = Message;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx).map(|opt| match opt {
            Some(Ok(line)) => Some(Message {
                sender: "user".to_string(),
                message: MessageContent::Text(line),
            }),
            _ => None,
        })
    }
}

impl<S> Sink<Message> for UserAgentProxy<S>
where
    for<'a> Pin<&'a mut UserAgentProxy<S>>: DerefMut<Target = Self>,
    S: Sink<Message> + Unpin,
{
    type Error = <S as Sink<Message>>::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.tx.poll_ready_unpin(cx)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
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

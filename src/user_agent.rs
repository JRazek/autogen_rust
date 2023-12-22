use std::ops::DerefMut;
use std::pin::Pin;

use futures::{
    channel::mpsc::{SendError, Sender},
    Sink, SinkExt, Stream, StreamExt,
};

use crate::{agent_traits::AgentProxy, chat::TextChat};

pub enum Message {
    Text(String),
}

pub struct UserAgent<S> {
    rx: S,
    tx: Sender<Message>,
}

impl<S> UserAgent<S> {
    pub fn with_stream(stream: S) -> Self {
        todo!()
    }
}

impl<S> Stream for UserAgent<S>
where
    S: Stream<Item = Message> + Unpin,
{
    type Item = Message;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_next_unpin(cx)
    }
}

impl<S> Sink<Message> for UserAgent<S>
where
    for<'a> Pin<&'a mut UserAgent<S>>: DerefMut<Target = Self>,
{
    type Error = SendError;

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

impl<S> AgentProxy<Message> for UserAgent<S>
where
    for<'a> Pin<&'a mut UserAgent<S>>: DerefMut<Target = Self>,
    S: Stream<Item = Message> + Unpin,
{
}

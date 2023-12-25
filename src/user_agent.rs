use std::pin::Pin;

use futures::channel::mpsc::{SendError, Sender};
use futures::{Sink, SinkExt, Stream, StreamExt};

use async_std::io::{prelude::BufReadExt, stdin, BufReader};
use async_std::io::{Lines, Stdin};

type StdinLines = Lines<BufReader<Stdin>>;

#[derive(Clone)]
pub struct UserAgent;

//impl Agent<ChatMessage> for UserAgent {
//    fn stream(&self, _chat_history: impl IntoIterator<Item = ChatMessage>) -> Self::ProxyStream {
//        let lines_stream = BufReader::new(stdin()).lines();
//
//        UserAgentProxyStream { rx: lines_stream }
//    }
//
//    fn sink(
//        &self,
//        _chat_history: impl IntoIterator<Item = ChatMessage>,
//    ) -> Self::ProxySink {
//        let (tx, rx) = futures_mpsc::channel(1);
//
//        tokio::spawn(rx.for_each(|msg| async move {
//            println!("UserAgent received message: {:?}", msg);
//        }));
//
//        UserAgentProxySink { tx }
//    }
//}


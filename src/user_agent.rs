use std::pin::Pin;

use futures::channel::mpsc::{SendError, Sender};
use futures::{Sink, SinkExt, Stream, StreamExt};

use async_std::io::{prelude::BufReadExt, stdin, BufReader};
use async_std::io::{Lines, Stdin};

use crate::agent_traits::{CodeExtractor, UserCodeExecutor};
use crate::user_proxy_agent_executor::UserProxyAgentExecutor;

type StdinLines = Lines<BufReader<Stdin>>;

pub enum ExecutionResponse {

}

#[derive(Clone)]
pub struct UserAgent<M> {
    _m: std::marker::PhantomData<M>,
}

//jak dokladnie wygląda komunikacji w autogenie w przypadku pisania kodu przez llma?

////impl<M> UserAgent<M> {
////    fn with_user_proxy<Extractor, Executor, C>(
////        user_proxy_agent_executor: UserProxyAgentExecutor<Extractor, Executor, M, C>,
////    ) -> Self
////    where
////        Extractor: CodeExtractor<M, CodeBlock = M>,
////        Executor: UserCodeExecutor<CodeBlock = C, Response = M>,
////    {
////        todo!()
////    }
////}
////
//////impl Agent<ChatMessage> for UserAgent {
//////    fn stream(&self, _chat_history: impl IntoIterator<Item = ChatMessage>) -> Self::ProxyStream {
//////        let lines_stream = BufReader::new(stdin()).lines();
//////
//////        UserAgentProxyStream { rx: lines_stream }
//////    }
//////
//////    fn sink(
//////        &self,
//////        _chat_history: impl IntoIterator<Item = ChatMessage>,
//////    ) -> Self::ProxySink {
//////        let (tx, rx) = futures_mpsc::channel(1);
//////
//////        tokio::spawn(rx.for_each(|msg| async move {
//////            println!("UserAgent received message: {:?}", msg);
//////        }));
//////
//////        UserAgentProxySink { tx }
//////    }
//////}

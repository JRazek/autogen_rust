#![allow(dead_code)]

use crate::user_proxy_agent_executor::UserProxyAgentExecutor;

use super::code_traits::{CodeExtractor, UserCodeExecutor};

pub enum ExecutionResponse {
    
}

/// UserAgent is a struct that represents a user of the system which can run code.
#[derive(Clone)]
pub struct UserAgent<M> {
    _m: std::marker::PhantomData<M>,
}

impl<M> UserAgent<M> {
    fn with_user_proxy<Extractor, Executor, C>(
        _user_proxy_agent_executor: UserProxyAgentExecutor<Executor, C>,
    ) -> Self
    where
        Extractor: CodeExtractor<M, CodeBlock = M>,
        Executor: UserCodeExecutor<CodeBlock = C, Response = M>,
    {
        todo!()
    }
}

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

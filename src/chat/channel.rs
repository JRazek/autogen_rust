use std::error::Error;

use crate::agent_traits::{AgentProxySink, AgentProxyStream};

use futures::{SinkExt, StreamExt};

use tracing::trace;

async fn chat_channel<T, Stream, Sink>(
    agent_proxy_stream: Stream,
    agent_proxy_sink: Sink,
) -> Result<(), impl Error>
where
    Stream: AgentProxyStream<T>,
    Sink: AgentProxySink<T>,
    <Sink as futures::Sink<T>>::Error: Error,
{
    let res = agent_proxy_stream.map(Ok).forward(agent_proxy_sink).await;

    trace!("chat_channel finished");

    res
}

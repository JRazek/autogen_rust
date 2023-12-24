#![feature(trait_alias)]
#![feature(result_option_inspect)]

mod agent_traits;
mod chat;
mod user_agent;

use async_std::io::prelude::*;
use async_std::io::BufReader;

use agent_traits::AgentProxySink;
use async_std::io::stdin;
use chat::scheduler::RoundRobinScheduler;
use user_agent::UserAgent;
use user_agent::UserAgentProxyStream;

use chat::ChatMessage as Message;

use chat::TextChat;

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let user_agent = UserAgent;

    let mut chat = TextChat::default();

    chat.spawn_agent("user1".to_string(), user_agent);

    chat.run(RoundRobinScheduler::default()).await.unwrap();
}

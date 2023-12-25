#![feature(trait_alias)]
#![feature(result_option_inspect)]

mod agent_traits;
mod chat;
mod user_agent;

mod agent_traits2;
mod chat2;

use async_std::io::prelude::*;
use async_std::io::BufReader;

use agent_traits::AgentProxySink;
use async_std::io::stdin;
use user_agent::UserAgent;
use chat2::scheduler::RoundRobin;
use chat2::Chat;

#[tokio::main]
async fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let user_agent = UserAgent;

    let chat: Chat<String> = Chat::new(RoundRobin::default()).await;

    //    chat.spawn_agent("user1".to_string(), user_agent);
    //
    //    chat.run(RoundRobinScheduler::default()).await.unwrap();
}

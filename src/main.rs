#![feature(trait_alias)]
#![feature(result_option_inspect)]

mod agent_traits;
mod chat;
mod user_agent;

use agent_traits::AgentProxy;
use user_agent::UserAgentProxy;

use chat::ChatMessage as Message;

use chat::TextChat;

use futures::StreamExt;

fn main() {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    )
    .unwrap();

    let chat = TextChat::default();
}

#![feature(trait_alias)]
#![feature(result_option_inspect)]

mod chat;
mod user_agent;

mod agent_traits;
use chat::scheduler::RoundRobin;
use chat::Chat;
use user_agent::UserAgent;

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

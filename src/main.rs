#![feature(trait_alias)]
#![feature(result_option_inspect)]

mod group_chat;
mod user_agent;

mod agent_traits;
use group_chat::scheduler::RoundRobin;
use group_chat::GroupChat;
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

    let chat: GroupChat<String> = GroupChat::new(RoundRobin::default()).await;

    //    chat.spawn_agent("user1".to_string(), user_agent);
    //
    //    chat.run(RoundRobinScheduler::default()).await.unwrap();
}

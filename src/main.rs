#![feature(trait_alias)]

mod agent_traits;
mod chat;
mod user_agent;

use agent_traits::AgentProxy;
use user_agent::UserAgent;

use chat::TextChat;

struct LlmAgent;

fn main() {
    let stdin_lock = std::io::stdin();
    let user_agent = UserAgent::with_stream(stdin_lock);

    let chat = TextChat;
}

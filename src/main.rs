#![feature(trait_alias)]

mod agent_traits;
mod chat;
mod user_agent;

use agent_traits::AgentProxy;
use user_agent::UserAgentProxy;

use chat::ChatMessage as Message;

use chat::TextChat;

use futures::StreamExt;

fn main() {
    let chat = TextChat::default();
}

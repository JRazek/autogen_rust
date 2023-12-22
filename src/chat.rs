use std::sync::mpsc::Sender;

use crate::{agent_traits::AgentProxy, user_agent::Message};

use futures::channel::mpsc::SendError;

pub struct AgentChatHandle<Y> {
    //    agent: T,
    chat_tx: Sender<Y>,
}

pub struct ChatMessage<T> {
    sender: String,
    recipient: String,
    message: T,
}

pub struct TextChat;

use futures::{SinkExt, StreamExt};

impl TextChat {
    fn spawn_agent<A: AgentProxy<ChatMessage<T>>, T>(
        &self,
        agent_name: impl ToOwned<Owned = String>,
        agent_proxy: A,
    ) -> AgentChatHandle<Message> {
        let (agent_tx, agent_rx) = agent_proxy.split();

        let rx_task = async move {};

        todo!()
    }
}

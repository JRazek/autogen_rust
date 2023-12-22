use crate::agent_traits::{Agent, AgentProxy};

use futures::channel::mpsc::{channel, Sender};

pub struct ChatMessage {
    sender: String,
    recipient: String,
    message: MessageContent,
}

pub enum MessageContent {
    Text(String),
}

type ChatHistory = Vec<ChatMessage>;

pub struct TextChat {
    agents_turn_tx: Vec<Sender<ChatHistory>>,
}

impl Default for TextChat {
    fn default() -> Self {
        Self {
            agents_turn_tx: Vec::new(),
        }
    }
}

use futures::StreamExt;

impl TextChat {
    pub fn spawn_agent<A>(&mut self, agent_name: impl ToOwned<Owned = String>, agent: A)
    where
        A: Agent<ChatMessage> + Send + Clone + 'static,
    {
        let (turn_tx, turn_rx) = channel(1);

        self.agents_turn_tx.push(turn_tx);

        let turn_task = turn_rx.for_each(move |chat_history| {
            let agent = agent.clone();
            async move {
                let (agent_proxy_tx, agent_proxy_rx) = agent.take_turn(chat_history).split();

                todo!()
            }
        });

        tokio::spawn(turn_task);
    }
}

pub struct AgentChatHandle<T> {
    chat_tx: Sender<T>,
}

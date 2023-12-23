use crate::agent_traits::{Agent, AgentProxy};

use futures::channel::mpsc::{channel, Receiver, Sender};

use tracing::error;
use tracing::{debug, info, trace};

mod error;
mod scheduler;

#[derive(Clone, Debug)]
pub struct ChatMessage {
    sender: String,
    recipient: String,
    message: MessageContent,
}

#[derive(Clone, Debug)]
pub enum MessageContent {
    Text(String),
}

type ChatHistory = Vec<ChatMessage>;

pub struct TextChat {
    agents_turn_tx: Vec<(String, Sender<ChatHistory>)>,
    turn_done_tx: Sender<()>,
    turn_done_rx: Receiver<()>,
}

impl Default for TextChat {
    fn default() -> Self {
        let (turn_done_tx, turn_done_rx) = channel(1);

        Self {
            agents_turn_tx: Vec::new(),
            turn_done_tx,
            turn_done_rx,
        }
    }
}

use futures::SinkExt;
use futures::StreamExt;

use self::scheduler::Scheduler;

impl TextChat {
    pub fn spawn_agent<A>(&mut self, agent_name: impl ToOwned<Owned = String>, agent: A)
    where
        A: Agent<ChatMessage> + Send + Clone + 'static,
        <A as Agent<ChatMessage>>::Proxy: Send + Clone,
    {
        let agent_name = agent_name.to_owned();
        debug!("Spawning agent: {}", agent_name);

        let (turn_tx, turn_rx) = channel(1);

        self.agents_turn_tx.push((agent_name.clone(), turn_tx));

        let turn_done_tx = self.turn_done_tx.clone();
        let turn_task = turn_rx.for_each(move |chat_history| {
            let agent = agent.clone();

            let mut turn_done_tx = turn_done_tx.clone();
            let agent_name = agent_name.clone();

            async move {
                debug!("Agent {} taking turn..", agent_name);
                let (agent_proxy_tx, agent_proxy_rx) = agent.take_turn(chat_history).split();

                _ = turn_done_tx.send(()).await.inspect_err(|e| {
                    error!("Error sending turn done: {}", e);
                });
            }
        });

        tokio::spawn(turn_task);
    }

    async fn run_chat<S>(self, mut scheduler: S) -> Result<(), error::Error>
    where
        S: Scheduler,
    {
        let mut chat_history = Vec::new();

        let agent_names = self.agents_turn_tx.iter().map(|(name, _)| name.as_str());

        while let Some(idx) = scheduler.next_agent(agent_names.clone(), &chat_history) {
            match self.agents_turn_tx.get(idx) {
                Some((agent_name, tx)) => {
                    let mut tx = tx.clone();

                    if let Err(e) = tx.send(chat_history.clone()).await {
                        error!("Error sending chat history: {}", e);
                        return Err(error::Error::AgentTurnTxSendError(agent_name.clone()));
                    }
                }
                None => {
                    error!("Invalid agent index: {}", idx);
                    return Err(error::Error::SchedulerInvalidBounds(idx));
                }
            }
        }

        Ok(())
    }
}

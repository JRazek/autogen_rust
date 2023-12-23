use crate::agent_traits::{Agent, AgentProxy};

use futures::channel::mpsc as futures_mpsc;

use futures::channel::oneshot as futures_oneshot;

use tracing::error;
use tracing::{debug, info, trace};

mod channel;
mod error;
mod scheduler;

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub sender: String,
    pub message: MessageContent,
}

#[derive(Clone, Debug)]
pub enum MessageContent {
    Text(String),
}

type ChatHistory = Vec<ChatMessage>;

type TurnDoneSender = futures_oneshot::Sender<()>;
type TurnDoneReceiver = futures_oneshot::Receiver<()>;

pub struct TextChat {
    agents_turn_tx: Vec<(String, futures_mpsc::Sender<(TurnDoneSender, ChatHistory)>)>,
}

impl Default for TextChat {
    fn default() -> Self {
        Self {
            agents_turn_tx: Vec::new(),
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

        let (turn_tx, turn_rx) = futures_mpsc::channel(1);

        self.agents_turn_tx.push((agent_name.clone(), turn_tx));

        let turn_task = turn_rx.for_each(move |(turn_done_tx, chat_history)| {
            let agent = agent.clone();

            let agent_name = agent_name.clone();

            async move {
                debug!("Agent {} taking turn..", agent_name);
                let (agent_proxy_tx, agent_proxy_rx) = agent.take_turn(chat_history).split();

                // receive messages from agent

                _ = turn_done_tx.send(());
            }
        });

        tokio::spawn(turn_task);
    }

    async fn run_chat<S>(self, mut scheduler: S) -> Result<(), error::Error>
    where
        S: Scheduler,
    {
        let chat_history = Vec::new();

        let agent_names = self.agents_turn_tx.iter().map(|(name, _)| name.as_str());

        while let Some(idx) = scheduler.next_agent(agent_names.clone(), &chat_history) {
            match self.agents_turn_tx.get(idx) {
                Some((agent_name, tx)) => {
                    let mut tx = tx.clone();

                    let (turn_done_tx, turn_done_rx) = futures_oneshot::channel();
                    tx.send((turn_done_tx, chat_history.clone()))
                        .await
                        .map_err(|_| error::Error::AgentTurnTxSendError(agent_name.clone()))
                        .inspect_err(|e| {
                            error!("Error sending chat history: {}", e);
                        })?;

                    turn_done_rx
                        .await
                        .map_err(|_| error::Error::AgentTurnDoneRxRecvError(agent_name.clone()))
                        .inspect_err(|e| {
                            error!("Error receiving turn done: {}", e);
                        })?;

                    //TODO update chat history here
                }
                None => {
                    error!("Invalid agent index: {}", idx);
                    Err(error::Error::SchedulerInvalidBounds(idx))?;
                }
            }
        }

        Ok(())
    }
}

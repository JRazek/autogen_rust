#![allow(dead_code)]

use futures::channel::mpsc as futures_mpsc;

use futures::{SinkExt, StreamExt};

pub mod error;
pub mod scheduler;

use tracing::{debug, error};

use self::scheduler::Scheduler;

use crate::agent_traits::{ConsumerAgent, ProducerAgent};

use futures::channel::oneshot as futures_oneshot;

use error::GroupChatTaskError;

use std::fmt::Debug;
use tokio::task::JoinHandle;

pub struct GroupChat<M>
where
    M: Send + 'static,
{
    new_agent_tx: futures_mpsc::Sender<()>,
    new_agent_ack_rx: futures_mpsc::Receiver<futures_mpsc::Receiver<Ordering<M>>>,

    group_chat_task_result: JoinHandle<Result<(), GroupChatTaskError>>,
}

enum Ordering<M>
where
    M: Send + 'static,
{
    Send(futures_oneshot::Sender<Result<M, GroupChatTaskError>>),
    Receive(M, futures_oneshot::Sender<Result<(), GroupChatTaskError>>),
}

impl<M> GroupChat<M>
where
    M: Clone + Send + Sync + Debug + 'static,
{
    pub async fn new<S>(mut scheduler: S) -> GroupChat<M>
    where
        S: Scheduler + Send + 'static,
    {
        let (new_agent_tx, new_agent_rx) = futures_mpsc::channel(1);
        let (new_agent_ack_tx, new_agent_ack_rx) =
            futures_mpsc::channel::<futures_mpsc::Receiver<_>>(1);

        let group_chat_task_result = tokio::spawn(async move {
            let mut ordering_tx_vec: Vec<_> = new_agent_rx
                .then(|_| {
                    let mut new_agent_ack_tx = new_agent_ack_tx.clone();

                    async move {
                        debug!("New agent spawned");

                        let (ordering_tx, ordering_rx) = futures_mpsc::channel::<Ordering<M>>(1);

                        new_agent_ack_tx
                            .send(ordering_rx)
                            .await
                            .map_err(|_| GroupChatTaskError::SendError)?;

                        Ok::<_, GroupChatTaskError>(ordering_tx)
                    }
                })
                .map(|res| res)
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<_, _>>()
                .inspect_err(|err| error!("Error in new agent: {}", err))?;

            while let Some(agent_idx) = scheduler.next_agent(ordering_tx_vec.len()) {
                debug!("Next agent: {}", agent_idx);

                if agent_idx >= ordering_tx_vec.len() {
                    panic!("Invalid agent index: {}", agent_idx);
                }
                //this accounts for all the tasks + the current one.
                debug!(
                    "Sending Send(broadcast_tx) to ordering_tx_vec[{}]",
                    agent_idx
                );

                let (sender_response_tx, sender_response_rx) = futures_oneshot::channel();

                ordering_tx_vec[agent_idx]
                    .send(Ordering::Send(sender_response_tx))
                    .await
                    .unwrap();

                let sender_response = sender_response_rx.await.unwrap()?;

                use futures::stream::iter as async_iter;

                let receive_responses_rx_vec = async_iter(ordering_tx_vec.clone())
                    .enumerate()
                    .filter(|&(idx, _)| async move { idx != agent_idx })
                    .then(|(idx, mut tx)| {
                        let sender_response = sender_response.clone();
                        async move {
                            debug!("Sending Receive(broadcast_rx) to ordering_tx_vec[{}]", idx);

                            let (receive_response_tx, receive_response_rx) =
                                futures_oneshot::channel::<Result<_, _>>();

                            tx.send(Ordering::Receive(sender_response, receive_response_tx))
                                .await
                                .unwrap();

                            receive_response_rx
                        }
                    })
                    .collect::<Vec<futures_oneshot::Receiver<_>>>()
                    .await;

                async_iter(receive_responses_rx_vec)
                    .then(|rx| async move { rx.await.unwrap() })
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .collect::<Result<_, _>>()?;

                debug!("Agent round finished");
            }

            debug!("Chat task finished");

            Ok::<(), GroupChatTaskError>(())
        });

        GroupChat {
            new_agent_tx,
            new_agent_ack_rx,
            group_chat_task_result,
        }
    }

    pub async fn add_agent<A, E>(&mut self, mut agent: A)
    where
        A: ConsumerAgent<M, Error = E> + ProducerAgent<Mtx = M> + Send + 'static,
        E: error::ErrorT,
        <A as ProducerAgent>::Error: error::ErrorT,
    {
        self.new_agent_tx.send(()).await.expect("Chat task died");

        let mut ordering_rx = self
            .new_agent_ack_rx
            .next()
            .await
            .expect("No response from chat task");

        tokio::spawn(async move {
            while let Some(order) = ordering_rx.next().await {
                match order {
                    Ordering::Send(sender_response_tx) => {
                        debug!("Sending to agent");

                        let sender_response: Result<M, GroupChatTaskError> = agent
                            .generate_prompt()
                            .await
                            .map_err(|err| GroupChatTaskError::OtherError(err.into()));

                        sender_response_tx.send(sender_response).unwrap();
                    }
                    //TODO should handle responding agents here as well.
                    Ordering::Receive(messasge, receive_result_tx) => {
                        debug!("Receiving from agent");

                        let result = agent
                            .receive(messasge)
                            .await
                            .map_err(|err| GroupChatTaskError::OtherError(err.into()));

                        //might forward all errors here
                        receive_result_tx.send(result).unwrap();

                        debug!("Agent receive finished receiving. Waiting on barrier");
                    }
                }

                debug!("Agent round subroutine finished");
            }
        });
    }

    //just drop all the channels in Chat and allow the scheduled tasks to leave loop.
    pub async fn start(self) -> Result<(), GroupChatTaskError> {
        drop(self.new_agent_tx);
        drop(self.new_agent_ack_rx);

        self.group_chat_task_result.await.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{SinkExt, StreamExt};

    use async_trait::async_trait;

    use tracing::debug;

    #[derive(Clone)]
    struct TestAgent {
        to_send: &'static str,

        received: Vec<&'static str>,

        expected: Vec<&'static str>,

        task_done_tx: futures_mpsc::Sender<()>,
    }

    impl Drop for TestAgent {
        fn drop(&mut self) {
            debug!("dropping test agent");
        }
    }

    use thiserror::Error;

    #[derive(Debug, Error)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "")
        }
    }

    #[async_trait]
    impl ConsumerAgent<&'static str> for TestAgent {
        type Error = TestError;

        async fn receive(&mut self, msg: &'static str) -> Result<(), Self::Error> {
            debug!("Received message: {}", msg);

            self.received.push(msg);

            if self.received.len() == self.expected.len() {
                assert_eq!(self.received, self.expected);

                debug!("Agent receive barrier waiting done");

                self.task_done_tx.send(()).await.unwrap();
            }

            if self.received.len() > self.expected.len() {
                panic!("Received more messages than expected");
            }

            debug!("Agent receive finished receiving");

            Ok(())
        }
    }

    #[async_trait]
    impl ProducerAgent for TestAgent {
        type Mtx = &'static str;
        type Error = TestError;

        async fn generate_prompt(&mut self) -> Result<&'static str, Self::Error> {
            debug!("Agent send finished sending");

            Ok(self.to_send)
        }
    }

    #[tokio::test]
    async fn test_chat() {
        tracing::subscriber::set_global_default(
            tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(tracing::Level::TRACE)
                .finish(),
        )
        .unwrap();

        let mut chat = GroupChat::new(scheduler::RoundRobin::with_max_rounds(3)).await;

        let (task_done_tx, task_done_rx) = futures_mpsc::channel(10);

        let agent1 = TestAgent {
            to_send: "phrase1",
            expected: vec!["phrase2", "phrase3"],
            received: Vec::new(),
            task_done_tx: task_done_tx.clone(),
        };

        let agent2 = TestAgent {
            to_send: "phrase2",
            expected: vec!["phrase1", "phrase3"],
            received: Vec::new(),
            task_done_tx: task_done_tx.clone(),
        };

        let agent3 = TestAgent {
            to_send: "phrase3",
            expected: vec!["phrase1", "phrase2"],
            received: Vec::new(),
            task_done_tx,
        };

        chat.add_agent(agent1).await;
        chat.add_agent(agent2).await;
        chat.add_agent(agent3).await;

        chat.start().await.unwrap();

        debug!("Waiting for task done");
        assert_eq!(task_done_rx.collect::<Vec<_>>().await.len(), 3);
    }
}

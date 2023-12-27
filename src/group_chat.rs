#![allow(dead_code)]

use std::sync::Arc;

use crate::agent_traits::Agent;
use futures::channel::mpsc as futures_mpsc;
use tokio::sync::{broadcast, Barrier};

use futures::{SinkExt, StreamExt};

pub mod scheduler;

use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

use self::scheduler::Scheduler;

use futures::channel::oneshot as futures_oneshot;

pub struct GroupChat<M>
where
    M: Send + 'static,
{
    new_agent_tx: futures_mpsc::Sender<()>,
    new_agent_ack_rx: futures_mpsc::Receiver<futures_mpsc::Receiver<Ordering<M>>>,

    chat_task_dealine: CancellationToken,
}

pub trait Error: Send + Sync + 'static {}

struct SendError;

enum Ordering<M>
where
    M: Send + 'static,
{
    Send(futures_oneshot::Sender<Result<Result<M, SendError>, Box<dyn Error>>>),
    Receive(M, futures_oneshot::Sender<Result<(), Box<dyn Error>>>),
}

use std::fmt::Debug;

impl<M> GroupChat<M>
where
    M: Clone + Send + Sync + Debug + 'static,
{
    pub async fn new<S>(mut scheduler: S) -> GroupChat<M>
    where
        S: Scheduler + Send + 'static,
    {
        let (new_agent_tx, new_agent_rx) = futures_mpsc::channel(1);
        let (new_agent_ack_tx, new_agent_ack_rx) = futures_mpsc::channel(1);

        let chat_task_dealine = CancellationToken::new();

        let chat_task_dealine_child = chat_task_dealine.child_token();

        tokio::spawn(async move {
            let mut ordering_tx_vec: Vec<_> = new_agent_rx
                .then(|_| {
                    let mut new_agent_ack_tx = new_agent_ack_tx.clone();

                    async move {
                        debug!("New agent spawned");

                        let (ordering_tx, ordering_rx) = futures_mpsc::channel::<Ordering<M>>(1);

                        new_agent_ack_tx.send(ordering_rx).await.unwrap();

                        ordering_tx
                    }
                })
                .collect()
                .await;

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

                let Ok(sender_response) = sender_response_rx.await.unwrap() else {
                    panic!("Sender response error");
                };

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

                async_iter(receive_responses_rx_vec).for_each(|rx| async move {
                    debug!("Waiting for receive response");
                    let receive_response = rx.await.unwrap();

                    match receive_response {
                        Ok(()) => {
                            trace!("received response in receive");
                        }
                        Err(err) => {
                            error!("Error in receive");
                        }
                    }
                });

                debug!("Agent round finished");
            }

            debug!("Chat task finished");
            chat_task_dealine.cancel();
        });

        GroupChat {
            new_agent_tx,
            new_agent_ack_rx,
            chat_task_dealine: chat_task_dealine_child,
        }
    }

    pub async fn add_agent<A>(&mut self, mut agent: A)
    where
        A: Agent<M, M> + Send + 'static,
        <A as Agent<M, M>>::Error: std::error::Error + Send + Sync + 'static + 'static,
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

                        let sender_response: Result<M, SendError> =
                            agent.send().await.map_err(|_| SendError);

                        if let Err(_) = sender_response_tx.send(Ok(sender_response)) {
                            panic!("Chat task died");
                        }
                    }
                    Ordering::Receive(messasge, receive_result_tx) => {
                        debug!("Receiving from agent");

                        agent.receive(messasge).await;

                        //might forward all errors here
                        if let Err(_) = receive_result_tx.send(Ok(())) {
                            panic!("Chat task died");
                        }

                        debug!("Agent receive finished receiving. Waiting on barrier");
                    }
                }

                debug!("Agent round subroutine finished");
            }
        });
    }

    //just drop all the channels in Chat and allow the scheduled tasks to leave loop.
    pub async fn start(self) {
        drop(self.new_agent_tx);
        drop(self.new_agent_ack_rx);

        self.chat_task_dealine.cancelled().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::{Sink, SinkExt, Stream, StreamExt};

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

    #[async_trait]
    impl Agent<&'static str, &'static str> for TestAgent {
        type Error = ();

        async fn receive(&mut self, mut msg: &'static str) {
            debug!("Received message: {}", msg);

            if self.received.len() == self.expected.len() {
                assert_eq!(self.received, self.expected);

                debug!("Agent receive barrier waiting done");

                self.task_done_tx.send(()).await.unwrap();
            }

            debug!("Agent receive finished receiving");
        }

        async fn send(&mut self) -> Result<&'static str, ()> {
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

        chat.start().await;

        debug!("Waiting for task done");
        assert_eq!(task_done_rx.collect::<Vec<_>>().await.len(), 3);
    }
}

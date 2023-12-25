#![allow(dead_code)]

use std::sync::Arc;

use crate::agent_traits::Agent;
use futures::channel::mpsc as futures_mpsc;
use tokio::sync::{broadcast, Barrier};

use futures::{SinkExt, StreamExt};

pub mod scheduler;

use tokio_util::sync::CancellationToken;
use tracing::debug;

use self::scheduler::Scheduler;

pub struct GroupChat<M>
where
    M: Clone + Send + 'static,
{
    new_agent_tx: futures_mpsc::Sender<()>,
    new_agent_ack_rx: futures_mpsc::Receiver<futures_mpsc::Receiver<Ordering<M>>>,

    chat_task_dealine: CancellationToken,
}

enum Ordering<M>
where
    M: Clone + Send + 'static,
{
    Receive(broadcast::Receiver<M>, Arc<Barrier>),
    Send(broadcast::Sender<M>, Arc<Barrier>),
}

impl<M> GroupChat<M>
where
    M: Clone + Send + 'static,
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

                        let (ordering_tx, ordering_rx) = futures_mpsc::channel(1);

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
                let barrier = Arc::new(Barrier::new(ordering_tx_vec.len() + 1));

                let (broadcast_tx, _) = broadcast::channel(1024);

                debug!(
                    "Sending Send(broadcast_tx) to ordering_tx_vec[{}]",
                    agent_idx
                );

                ordering_tx_vec[agent_idx]
                    .send(Ordering::Send(broadcast_tx.clone(), barrier.clone()))
                    .await
                    .unwrap();

                use futures::stream::iter as async_iter;
                async_iter(ordering_tx_vec.clone())
                    .enumerate()
                    .filter(|&(idx, _)| async move { idx != agent_idx })
                    .for_each(|(idx, mut tx)| {
                        let broadcast_rx = broadcast_tx.subscribe();
                        let barrier = barrier.clone();

                        async move {
                            debug!("Sending Receive(broadcast_rx) to ordering_tx_vec[{}]", idx);
                            tx.send(Ordering::Receive(broadcast_rx, barrier))
                                .await
                                .unwrap();
                        }
                    })
                    .await;

                barrier.wait().await;
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
        A: Agent<M> + Send + 'static,
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
                    Ordering::Receive(mut broadcast_rx, barrier) => {
                        debug!("Receiving from agent");

                        let (mut futures_tx, futures_rx) = futures_mpsc::channel(10);

                        tokio::spawn(async move {
                            while let Ok(msg) = broadcast_rx.recv().await {
                                debug!("redirecting message from agent");
                                futures_tx.send(msg).await.unwrap();
                            }
                        });

                        agent.receive(futures_rx).await;

                        debug!("Agent receive finished receiving. Waiting on barrier");
                        barrier.wait().await;
                    }
                    Ordering::Send(broadcast_tx, barrier) => {
                        debug!("Sending to agent");

                        let (futures_tx, mut futures_rx) = futures_mpsc::channel(10);

                        tokio::spawn(async move {
                            while let Some(msg) = futures_rx.next().await {
                                debug!("redirecting message to agent");
                                if let Err(_) = broadcast_tx.send(msg) {
                                    panic!("Agent send failed");
                                }
                            }
                        });

                        agent.send(futures_tx).await;

                        debug!("Agent send finished receiving. Waiting on barrier");
                        barrier.wait().await;
                    }
                }

                debug!("Agent round subroutine finished");
            }
        });
    }

    //just drop all the channels in Chat and allow the scheduled tasks to leave loop.
    async fn start(self) {
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
        to_send: Vec<&'static str>,

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
    impl Agent<&'static str> for TestAgent {
        async fn receive(&mut self, mut stream: impl Stream<Item = &'static str> + Unpin + Send) {
            debug!("Agent receive started receiving");
            if let Some(msg) = stream.next().await {
                debug!("Received message: {}", msg);
                self.received.push(msg);
            }

            if self.received.len() == self.expected.len() {
                assert_eq!(self.received, self.expected);

                debug!("Agent receive barrier waiting done");

                self.task_done_tx.send(()).await.unwrap();
            }

            debug!("Agent receive finished receiving");
        }

        async fn send(&mut self, mut sink: impl Sink<&'static str> + Unpin + Send) {
            for msg in self.to_send.drain(..) {
                debug!("Sending message: {}", msg);
                if let Err(_) = sink.send(msg).await {
                    panic!("Agent send failed");
                }
            }

            debug!("Agent send finished sending");
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
            to_send: vec!["phrase1"],
            expected: vec!["phrase2", "phrase3"],
            received: Vec::new(),
            task_done_tx: task_done_tx.clone(),
        };

        let agent2 = TestAgent {
            to_send: vec!["phrase2"],
            expected: vec!["phrase1", "phrase3"],
            received: Vec::new(),
            task_done_tx: task_done_tx.clone(),
        };

        let agent3 = TestAgent {
            to_send: vec!["phrase3"],
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

use std::sync::Arc;

use crate::agent_traits2::Agent;
use futures::channel::mpsc as futures_mpsc;
use tokio::sync::{broadcast, Barrier, BarrierWaitResult};

use futures::{SinkExt, StreamExt};

mod ordering_channel;

pub mod scheduler;

use tokio_util::sync::CancellationToken;
use tracing::debug;

use self::scheduler::Scheduler;

pub struct Chat<M>
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

impl<M> Chat<M>
where
    M: Clone + Send + 'static,
{
    pub async fn new<S>(mut scheduler: S) -> Chat<M>
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

                let (broadcast_tx, broadcast_rx) = broadcast::channel(1024);

                debug!(
                    "Sending Send(broadcast_tx) to ordering_tx_vec[{}]",
                    agent_idx
                );

                //this accounts for all the tasks + the current one.
                let barrier = Arc::new(Barrier::new(ordering_tx_vec.len() + 1));

                ordering_tx_vec[agent_idx]
                    .send(Ordering::Send(broadcast_tx, barrier.clone()))
                    .await
                    .unwrap();

                use futures::stream::iter as async_iter;

                async_iter(ordering_tx_vec.clone())
                    .enumerate()
                    .filter(|&(idx, _)| async move { idx != agent_idx })
                    .for_each(|(idx, mut tx)| {
                        let broadcast_rx = broadcast_rx.resubscribe();
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
            }

            debug!("Chat task finished");
            chat_task_dealine.cancel();
        });

        Chat {
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

                        let stream =
                            futures::stream::poll_fn(move |_| match broadcast_rx.try_recv() {
                                Ok(msg) => std::task::Poll::Ready(Some(msg)),
                                Err(broadcast::error::TryRecvError::Closed) => {
                                    std::task::Poll::Ready(None)
                                }
                                Err(_) => std::task::Poll::Pending,
                            });

                        agent.receive(stream).await;
                        barrier.wait().await;
                    }
                    Ordering::Send(broadcast_tx, barrier) => {
                        debug!("Sending to agent");

                        let (futures_tx, mut futures_rx) = futures_mpsc::channel(1);

                        tokio::spawn(async move {
                            while let Some(msg) = futures_rx.next().await {
                                broadcast_tx.send(msg);
                            }
                        });

                        agent.send(futures_tx).await;
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
    }

    #[async_trait]
    impl Agent<&'static str> for TestAgent {
        async fn receive(&mut self, mut stream: impl Stream<Item = &'static str> + Unpin + Send) {
            if let Some(msg) = stream.next().await {
                debug!("Received message: {}", msg);
                self.received.push(msg);
            }

            if self.received.len() == self.expected.len() {
                assert_eq!(self.received, self.expected);
            }

            debug!("Agent receive finished receiving");
        }

        async fn send(&mut self, mut sink: impl Sink<&'static str> + Unpin + Send) {
            for msg in self.to_send.drain(..) {
                debug!("Sending message: {}", msg);
                sink.send(msg).await;
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

        let mut chat = Chat::new(scheduler::RoundRobin::with_max_rounds(3)).await;

        let agent1 = TestAgent {
            to_send: vec!["phrase1"],
            expected: vec!["phrase2, phrase3"],
            received: Vec::new(),
        };

        let agent2 = TestAgent {
            to_send: vec!["phrase2"],
            expected: vec!["phrase1", "phrase3"],
            received: Vec::new(),
        };

        let agent3 = TestAgent {
            to_send: vec!["phrase3"],
            expected: vec!["phrase1", "phrase2"],
            received: Vec::new(),
        };

        chat.add_agent(agent1).await;
        chat.add_agent(agent2).await;
        chat.add_agent(agent3).await;

        chat.start().await;
    }
}

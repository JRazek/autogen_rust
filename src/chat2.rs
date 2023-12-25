use crate::agent_traits2::Agent;
use futures::channel::mpsc as futures_mpsc;
use tokio::sync::broadcast;

use futures::{SinkExt, StreamExt};

mod ordering_channel;

pub mod scheduler;

use tracing::debug;

use self::scheduler::Scheduler;

pub struct Chat<M>
where
    M: Clone + Send + 'static,
{
    new_agent_tx: futures_mpsc::Sender<()>,
    new_agent_ack_rx: futures_mpsc::Receiver<futures_mpsc::Receiver<Ordering<M>>>,
}

enum Ordering<M>
where
    M: Clone + Send + 'static,
{
    Receive(broadcast::Receiver<M>),
    Send(broadcast::Sender<M>),
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

                ordering_tx_vec[agent_idx]
                    .send(Ordering::Send(broadcast_tx))
                    .await
                    .unwrap();

                use futures::stream::iter as async_iter;

                async_iter(ordering_tx_vec.clone())
                    .enumerate()
                    .filter(|&(idx, _)| async move { idx != agent_idx })
                    .for_each(|(idx, mut tx)| {
                        let broadcast_rx = broadcast_rx.resubscribe();
                        async move {
                            debug!("Sending Receive(broadcast_rx) to ordering_tx_vec[{}]", idx);
                            tx.send(Ordering::Receive(broadcast_rx.resubscribe()))
                                .await
                                .unwrap();
                        }
                    });
            }

            debug!("new_agent_rx closed. Chat will no longer accept new agents");
            debug!("Starting chat routines");
        });

        Chat {
            new_agent_tx,
            new_agent_ack_rx,
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
                    Ordering::Receive(mut rx) => {
                        debug!("Receiving from agent");

                        let stream = futures::stream::poll_fn(move |_| match rx.try_recv() {
                            Ok(msg) => std::task::Poll::Ready(Some(msg)),
                            Err(broadcast::error::TryRecvError::Closed) => {
                                std::task::Poll::Ready(None)
                            }
                            Err(_) => std::task::Poll::Pending,
                        });

                        agent.receive(stream).await;
                    }
                    Ordering::Send(tx) => {
                        debug!("Sending to agent");

                        let (futures_tx, mut futures_rx) = futures_mpsc::channel(1);

                        tokio::spawn(async move {
                            while let Some(msg) = futures_rx.next().await {
                                tx.send(msg);
                            }
                        });

                        agent.send(futures_tx).await;
                    }
                }
            }
        });
    }

    //just drop all the channels in Chat and allow the scheduled tasks to leave loop.
    async fn start(self) {}
}

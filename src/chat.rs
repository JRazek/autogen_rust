#![allow(dead_code)]

use tokio_util::sync::CancellationToken;

use crate::agent_traits::Agent;

pub struct Chat {
    chat_task_dealine: CancellationToken,
}

impl Chat {
    pub fn new<A1, A2, M1, M2>() -> Self
    where
        A1: Agent<M1, M2>,
        A2: Agent<M2, M1>,
    {
        Self {
            chat_task_dealine: CancellationToken::new(),
        }
    }
}

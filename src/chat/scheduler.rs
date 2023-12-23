use super::ChatHistory;

pub trait Scheduler {
    /// Returns the index of the next agent to take a turn.
    fn next_agent<'a>(
        &mut self,
        agent_names: impl ExactSizeIterator<Item = &'a str>,
        chat_history: &ChatHistory,
    ) -> Option<usize>;
}

pub struct RoundRobinScheduler {
    current_agent: usize,
}

impl Default for RoundRobinScheduler {
    fn default() -> Self {
        Self { current_agent: 0 }
    }
}

impl Scheduler for RoundRobinScheduler {
    fn next_agent<'a>(
        &mut self,
        agent_names: impl ExactSizeIterator<Item = &'a str>,
        _chat_history: &ChatHistory,
    ) -> Option<usize> {
        let next_agent = self.current_agent % agent_names.len();
        self.current_agent += 1;
        Some(next_agent)
    }
}

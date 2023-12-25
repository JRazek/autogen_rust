pub trait Scheduler {
    /// Returns the index of the next agent to take a turn.
    fn next_agent(&mut self, agents_count: usize) -> Option<usize>;
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
    fn next_agent(&mut self, agents_count: usize) -> Option<usize> {
        let next_agent = self.current_agent % agents_count;
        self.current_agent += 1;
        Some(next_agent)
    }
}

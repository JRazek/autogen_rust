pub trait Scheduler {
    /// Returns the index of the next agent to take a turn.
    fn next_agent(&mut self, agents_count: usize) -> Option<usize>;
}

pub struct RoundRobin {
    current_agent: usize,
    max_rounds: Option<usize>,
}

impl Default for RoundRobin {
    fn default() -> Self {
        Self {
            current_agent: 0,
            max_rounds: None,
        }
    }
}

impl RoundRobin {
    pub fn with_max_rounds(max_rounds: usize) -> Self {
        Self {
            current_agent: 0,
            max_rounds: Some(max_rounds),
        }
    }
}

impl Scheduler for RoundRobin {
    fn next_agent(&mut self, agents_count: usize) -> Option<usize> {
        let next_agent = self.current_agent % agents_count;
        self.current_agent += 1;

        match self.max_rounds {
            Some(max_rounds) if self.current_agent > max_rounds => None,
            _ => Some(next_agent),
        }
    }
}

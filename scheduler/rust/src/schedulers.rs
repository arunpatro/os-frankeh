use std::collections::VecDeque;

struct Scheduler {
    name: String,
    quantum: u64,
    runQ: VecDeque<Process>,
}

impl Scheduler {
    fn new(name: String) -> Self {
        Scheduler {
            name,
            quantum: 1e4,
            runQ: VecDeque::new(),
        }
    }

    fn add_process(&mut self, process: Process) {
        self.runQ.push_back(process);
    }

    fn get_next_process(&mut self) -> Option<Process> {
        self.runQ.pop_front()
    }
}
use crate::process::Process;

pub struct Scheduler {
    processes: Vec<String>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            processes: vec![],
            current: 0,
        }
    }

    pub fn add_process(&mut self, process: String) {
        self.processes.push(process);
    }

    pub fn get_next_process(&mut self) -> Option<&String> {
        if self.processes.is_empty() {
            None
        } else {
            let next_process = &self.processes[self.current];
            self.current = (self.current + 1) % self.processes.len();
            Some(next_process)
        }
    }
}

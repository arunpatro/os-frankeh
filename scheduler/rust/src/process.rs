enum State {
    CREATED,
    READY,
    RUNNG,
    BLOCK,
    DONE,
}

pub struct Process {
    id: usize,
    at: usize,
    tc: usize,
    cb: usize,
    io: usize,

    state: State,
    remaining: usize,
    io_remaining: usize,
    start: usize,
    end: usize,
    wait: usize,
    turnaround: usize,

    current_burst: usize,
    prempted: bool,
}

impl Process {
    pub fn new(id: usize, at: usize, tc: usize, cb: usize, io: usize) -> Process {
        Process {
            id,
            at,
            tc,
            cb,
            io,
            state: State::CREATED,
            remaining: tc,
            io_remaining: 0,
            start: 0,
            end: 0,
            wait: 0,
            turnaround: 0,
            current_burst: 0,
            prempted: false,
        }
    }
}
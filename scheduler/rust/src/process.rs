use crate::rand_generator::RandGenerator;

pub enum State {
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
    start: usize,
    end: usize,
    wait: usize,
    turnaround: usize,

    current_burst: usize,
    prempted: bool,
}

impl Process {
    pub fn new(id: usize, at: usize, tc: usize, cb: usize, io: usize, randgen:&RandGenerator, maxprio:usize) -> Process {
        Process {
            id,
            at,
            tc,
            cb,
            io,
            state: State::CREATED,
            remaining: tc,
            start: 0,
            end: 0,
            wait: 0,
            turnaround: 0,
            current_burst: 0,
            prempted: false,
        }
    }
}
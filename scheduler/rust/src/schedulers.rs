struct Scheduler {
    name: String,
    quantum: u64,
    runQ: VecDeque<Process>,
}
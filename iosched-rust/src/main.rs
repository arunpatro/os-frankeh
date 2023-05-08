use clap::{App, Arg};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
mod utils;
use utils::read_input_file;

// Define a struct to hold the flags
#[derive(Debug, Default)]
struct Flags {
    v_option: bool,
    q_option: bool,
    f_option: bool,
}

// Define a thread-local variable to hold the flags
thread_local!(static TFLAGS: RefCell<Flags> = RefCell::new(Flags::default()));

// prints the operations
macro_rules! v_trace {
    ($($arg:tt)*) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.v_option {
                println!("{}", format_args!($($arg)*));
            }
        });
    };
}

// prints the queue
macro_rules! q_trace {
    ($($arg:tt)*) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.q_option {
                println!("{}", format_args!($($arg)*));
            }
        });
    };
}

fn print_summary(io_operations: Rc<RefCell<Vec<io_operation>>>, stats: &Stats) {
    // printf("%5d: %5d %5d %5d\n", iop, req->arr_time, r->start_time, r->end_time);
    for (i, io) in io_operations.borrow().iter().enumerate() {
        println!(
            "{:5}: {:5} {:5} {:5}",
            i,
            io.time,
            io.start_time.unwrap(),
            io.completed_time.unwrap()
        );
    }

    println!(
        "SUM: {} {} {:.4} {:.2} {:.2} {}",
        stats.total_time,
        stats.total_movement,
        stats.io_utilization,
        stats.avg_turnaround_time,
        stats.avg_wait_time,
        stats.max_wait_time
    );
}

pub struct io_operation {
    time: usize,
    track: usize,

    // stats
    start_time: Option<usize>,
    completed_time: Option<usize>,
}

struct Stats {
    total_time: usize,
    total_movement: usize,
    io_utilization: f64,
    avg_turnaround_time: f64,
    avg_wait_time: f64,
    max_wait_time: usize,
}

trait IOScheduler {
    fn add(&mut self, io: usize);
    fn next(&mut self, track_head: usize) -> Option<usize>;
}

struct FIFO {
    queue: VecDeque<usize>,
}

impl FIFO {
    fn new() -> Self {
        FIFO {
            queue: VecDeque::new(),
        }
    }
}

impl IOScheduler for FIFO {
    fn add(&mut self, io: usize) {
        self.queue.push_back(io);
    }

    fn next(&mut self, _: usize) -> Option<usize> {
        self.queue.pop_front()
    }
}

struct SSTF {
    queue: VecDeque<usize>,
    io_operations: Rc<RefCell<Vec<io_operation>>>,
}

impl SSTF {
    fn new(io_operations: Rc<RefCell<Vec<io_operation>>>) -> Self {
        SSTF {
            queue: VecDeque::new(),
            io_operations: io_operations,
        }
    }
}

impl IOScheduler for SSTF {
    fn add(&mut self, io: usize) {
        self.queue.push_back(io);
    }

    fn next(&mut self, track_head: usize) -> Option<usize> {
        // iterate over queue and create a string that represents the queue
        let mut queue_string = String::new();

        // given the current track head position, find the closest track io request and return it
        // if there are multiple requests at the same distance, return the one that arrived first
        // if there are no requests in the queue, return None
        let mut min_distance = usize::MAX;
        let mut min_distance_index = 0;
        for (i, io) in self.queue.iter().enumerate() {
            let distance =
                (track_head as i64 - self.io_operations.borrow()[*io].track as i64).abs() as usize;
            if distance < min_distance {
                min_distance = distance;
                min_distance_index = i;
            }
            queue_string.push_str(&format!("{}:{} ", *io, distance));
        }
        if self.queue.len() > 0 {
            q_trace!(
                "\tGet: ({}) --> {}",
                queue_string,
                self.queue[min_distance_index]
            );
        }

        self.queue.remove(min_distance_index)
    }
}

struct LOOK {
    queue: VecDeque<usize>,
    io_operations: Rc<RefCell<Vec<io_operation>>>,
    dir: i8,
}

impl LOOK {
    fn new(io_operations: Rc<RefCell<Vec<io_operation>>>) -> Self {
        LOOK {
            queue: VecDeque::new(),
            io_operations: io_operations,
            dir: 1,
        }
    }
}

impl IOScheduler for LOOK {
    fn add(&mut self, io: usize) {
        self.queue.push_back(io);
    }

    fn next(&mut self, track_head: usize) -> Option<usize> {
        if self.queue.len() == 0 {
            return None;
        }

        let mut queue_string = String::new();
        let mut min_distance = usize::MAX;
        let mut min_distance_index: isize = -1;
        for (i, io) in self.queue.iter().enumerate() {
            if self.dir == 1 && self.io_operations.borrow()[*io].track < track_head {
                continue;
            } else if self.dir == -1 && self.io_operations.borrow()[*io].track > track_head {
                continue;
            }
            let distance =
                (track_head as i64 - self.io_operations.borrow()[*io].track as i64).abs() as usize;
            if distance < min_distance {
                min_distance = distance;
                min_distance_index = i as isize;
            }
            queue_string.push_str(&format!("{}:{} ", *io, distance));
        }

        // after iterating through the queue, if we didn't find a request in the current direction
        // then we need to change directions
        if min_distance_index == -1 {
            self.dir = -self.dir;
            q_trace!("\tGet: () --> change direction to {}", self.dir);
            return self.next(track_head);
        }

        let min_distance_index = min_distance_index as usize;
        q_trace!(
            "\tGet: ({}) --> {} dir={}",
            queue_string,
            self.queue[min_distance_index],
            self.dir
        );

        self.queue.remove(min_distance_index)
    }
}

struct CLOOK {
    queue: VecDeque<usize>,
    io_operations: Rc<RefCell<Vec<io_operation>>>,
}

impl CLOOK {
    fn new(io_operations: Rc<RefCell<Vec<io_operation>>>) -> Self {
        CLOOK {
            queue: VecDeque::new(),
            io_operations: io_operations,
        }
    }
}

impl IOScheduler for CLOOK {
    fn add(&mut self, io: usize) {
        self.queue.push_back(io);
    }

    fn next(&mut self, track_head: usize) -> Option<usize> {
        if self.queue.len() == 0 {
            return None;
        }

        let mut queue_string = String::new();
        let mut distances = Vec::new();
        for (i, io) in self.queue.iter().enumerate() {
            let distance = (self.io_operations.borrow()[*io].track as i64 - track_head as i64);
            distances.push(distance);
            queue_string.push_str(&format!("{}:{} ", *io, distance));
        }

        // after iterating through the queue,
        // first check for smallest non-negative distance
        // if nothing smallest exists, return smallest distance
        let smallest_positive_index = distances
            .iter()
            .enumerate()
            .filter(|&(_, &x)| x >= 0)
            .min_by_key(|&(_, &x)| x)
            .map(|(index, _)| index);

        if smallest_positive_index.is_some() {
            let smallest_positive_index = smallest_positive_index.unwrap();
            q_trace!(
                "\tGet: ({}) --> {}",
                queue_string,
                self.queue[smallest_positive_index]
            );
            return self.queue.remove(smallest_positive_index);
        } else {
            let smallest_index = distances
                .iter()
                .enumerate()
                .min_by_key(|&(_, &x)| x)
                .map(|(index, _)| index)
                .unwrap();
            q_trace!(
                "\tGet: ({}) --> go to bottom and pick {}",
                queue_string,
                self.queue[smallest_index]
            );
            return self.queue.remove(smallest_index);
        }
    }
}

struct FLOOK {
    queue: VecDeque<usize>,
    add_queue: VecDeque<usize>,
    io_operations: Rc<RefCell<Vec<io_operation>>>,
    dir: i8,
}

impl FLOOK {
    fn new(io_operations: Rc<RefCell<Vec<io_operation>>>) -> Self {
        FLOOK {
            queue: VecDeque::new(),
            add_queue: VecDeque::new(),
            io_operations: io_operations,
            dir: 1,
        }
    }
}

impl IOScheduler for FLOOK {
    fn add(&mut self, io: usize) {
        self.add_queue.push_back(io);
    }

    fn next(&mut self, track_head: usize) -> Option<usize> {
        if self.queue.len() == 0 {
            if self.add_queue.len() == 0 {
                return None;
            }
            std::mem::swap(&mut self.queue, &mut self.add_queue);
        }

        let mut queue_string = String::new();
        let mut min_distance = usize::MAX;
        let mut min_distance_index: isize = -1;
        for (i, io) in self.queue.iter().enumerate() {
            if self.dir == 1 && self.io_operations.borrow()[*io].track < track_head {
                continue;
            } else if self.dir == -1 && self.io_operations.borrow()[*io].track > track_head {
                continue;
            }
            let distance =
                (track_head as i64 - self.io_operations.borrow()[*io].track as i64).abs() as usize;
            if distance < min_distance {
                min_distance = distance;
                min_distance_index = i as isize;
            }
            queue_string.push_str(&format!("{}:{} ", *io, distance));
        }

        // after iterating through the queue, if we didn't find a request in the current direction
        // then we need to change directions
        if min_distance_index == -1 {
            self.dir = -self.dir;
            q_trace!("\tGet: () --> change direction to {}", self.dir);
            return self.next(track_head);
        }

        let min_distance_index = min_distance_index as usize;
        q_trace!(
            "\tGet: ({}) --> {} dir={}",
            queue_string,
            self.queue[min_distance_index],
            self.dir
        );

        self.queue.remove(min_distance_index)
    }
}

fn actual_main_fn(algorithm: &str, inputfile: &str) {
    // Read input file
    let io_requests = read_input_file(inputfile);
    let mut io_ptr = 0; // pointer to the next IO request to be processed

    // Create a new IO scheduler
    let mut scheduler: Box<dyn IOScheduler> = match algorithm {
        "N" => {
            Box::new(FIFO::new())
        }
        "S" => {
            Box::new(SSTF::new(io_requests.clone()))
        }
        "L" => {
            Box::new(LOOK::new(io_requests.clone()))
        }
        "C" => {
            Box::new(CLOOK::new(io_requests.clone()))
        }
        "F" => {
            Box::new(FLOOK::new(io_requests.clone()))
        }
        _ => {
            panic!("Invalid algorithm");
        }
    };

    let mut active_io: Option<usize> = None;
    let mut track_head = 0;

    // Simulation: Run the IO scheduler
    let mut time = 0;
    let mut stats = Stats {
        total_time: 0,
        total_movement: 0,
        io_utilization: 0.0,
        avg_turnaround_time: 0.0,
        avg_wait_time: 0.0,
        max_wait_time: 0,
    };
    v_trace!("TRACE");
    'simul: loop {
        // new io arrived
        if io_ptr < io_requests.borrow().len() && io_requests.borrow()[io_ptr].time == time {
            scheduler.add(io_ptr);
            v_trace!(
                "{}:{:6} add {}",
                time,
                io_ptr,
                io_requests.borrow()[io_ptr].track
            );
            io_ptr += 1;
        }

        // check active io for completion
        if let Some(io_id) = active_io {
            let io = &mut io_requests.borrow_mut()[io_id];
            if track_head == io.track {
                io.completed_time = Some(time);
                v_trace!("{}:{:6} finish {}", time, io_id, time - io.time);
                active_io = None;
            }
        }

        // if no active io, get next io from scheduler
        while active_io.is_none() {
            // If requests are pending
            // → Fetch the next request from IO-queue and start the new IO.
            if let Some(io_id) = scheduler.next(track_head) {
                let io = &mut io_requests.borrow_mut()[io_id];
                io.start_time = Some(time);
                active_io = Some(io_id);
                v_trace!("{}:{:6} issue {} {}", time, io_id, io.track, track_head);

                // Check if the new IO request has completed at the same time it was issued
                if track_head == io.track {
                    io.completed_time = Some(time);
                    v_trace!("{}:{:6} finish {}", time, io_id, time - io.time);
                    active_io = None;
                }
            } else if io_ptr >= io_requests.borrow().len() {
                // No more possible I/O requests, exit the simul loop
                break 'simul;
            } else {
                // No more pending I/O requests, exit the loop
                break;
            }
        }

        // Move the head
        if let Some(io_id) = active_io {
            let io = &io_requests.borrow()[io_id];
            if track_head < io.track {
                track_head += 1;
            } else if track_head > io.track {
                track_head -= 1;
            }
        }

        time += 1;
    }

    // Compute stats
    let io_requests_borrow = io_requests.borrow();
    let num_requests = io_requests_borrow.len() as f64;

    stats.total_time = time;
    stats.total_movement = io_requests_borrow
        .iter()
        .map(|io| io.completed_time.unwrap() - io.start_time.unwrap())
        .sum();
    stats.io_utilization = (stats.total_movement as f64) / (time as f64);
    stats.avg_turnaround_time = io_requests_borrow
        .iter()
        .map(|io| (io.completed_time.unwrap() - io.time) as f64)
        .sum::<f64>()
        / num_requests;
    stats.avg_wait_time = io_requests_borrow
        .iter()
        .map(|io| (io.start_time.unwrap() - io.time) as f64)
        .sum::<f64>()
        / num_requests;
    stats.max_wait_time = io_requests_borrow
        .iter()
        .map(|io| io.start_time.unwrap() - io.time)
        .max()
        .unwrap_or(0);

    // Print end stats
    print_summary(io_requests.clone(), &stats);
}

fn parse_args(actual_args: &Vec<String>) -> (String, String) {
    let matches = App::new("MMU program")
        .arg(
            Arg::with_name("algorithm")
                .short('s')
                .required(true)
                .help("algorithm")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v_flag")
                .short('v')
                .required(false)
                .help("v_flag")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("q_flag")
                .short('q')
                .required(false)
                .help("q_flag")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("f_flag")
                .short('f')
                .required(false)
                .help("f_flag")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("inputfile")
                .help("input file")
                .required(true)
                .index(1),
        )
        .get_matches_from(actual_args);

    let algorithm = matches.value_of("algorithm").unwrap().to_string();
    let inputfile = matches.value_of("inputfile").unwrap().to_string();

    TFLAGS.with(|tflags| {
        let mut tflags = tflags.borrow_mut();
        tflags.v_option = matches.is_present("v_flag");
        tflags.q_option = matches.is_present("q_flag");
        tflags.f_option = matches.is_present("f_flag");
    });

    (algorithm, inputfile)
}

fn get_default_args() -> Vec<String> {
    vec![
        "iosched".to_string(),
        "-sF".to_string(),
        "-v".to_string(),
        "-q".to_string(),
        "../lab4_assign/input1".to_string(),
    ]
}

fn main() {
    let default_args = get_default_args();
    let args = std::env::args().collect::<Vec<String>>();
    let actual_args = if args.len() > 1 { &args } else { &default_args };

    // Parse command line arguments
    let (algorithm, inputfile) = parse_args(actual_args);

    // Call your main function with the parsed arguments
    actual_main_fn(&algorithm, &inputfile);
}

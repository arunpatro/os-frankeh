use clap::{App, Arg};
use std::cell::RefCell;
use std::collections::VecDeque;
// use std::rc::Rc;
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

// prints the frame table
macro_rules! q_trace {
    ($frame_table:expr) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.q_option {
                // print_frame_table($frame_table);
            }
        });
    };
}

// prints the frame table at the end of each instruction
macro_rules! f_trace {
    ($frame_table:expr) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.f_option {
                // print_frame_table($frame_table);
            }
        });
    };
}

fn print_summary(io_operations: &Vec<io_operation>, stats: &Stats) {
    // printf("%5d: %5d %5d %5d\n", iop, req->arr_time, r->start_time, r->end_time);
    for (i, io) in io_operations.iter().enumerate() {
        println!(
            "{:5}: {:5} {:5} {:5}",
            i,
            io.time,
            io.start_time.unwrap(),
            io.completed_time.unwrap()
        );
    }

//     printf("SUM: %d %d %.4lf %.2lf %.2lf %d\n", total_time, tot_movement, io_utilization,
// total_time: tot_movement: io_utilization: avg_turnaround: avg_waittime: max_waittime:
//     avg_turnaround, avg_waittime, max_waittime);
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

trait IO_Scheduler {
    fn new(io_operations: Vec<io_operation>) -> Self;
    fn run(&mut self) -> Stats;
}

struct FIFO {
    io_operations: Vec<io_operation>,
}

impl FIFO {
    fn new(io_operations: Vec<io_operation>) -> Self {
        FIFO { io_operations }
    }
}
// impl IO_Scheduler for FIFO {
//     fn new(io_operations: Vec<io_operation>) -> Self {
//         FIFO { io_operations }
//     }

//     fn run(&mut self) -> Stats {

//     }
// }

fn actual_main_fn(algorithm: &str, inputfile: &str) {
    // Read input file
    let mut io_requests = read_input_file(inputfile);
    let mut io_ptr = 0; // pointer to the next IO request to be processed

    // Create a new IO scheduler
    // let mut io_scheduler = FIFO::new(io_operations);
    let mut io_queue: VecDeque<usize> = VecDeque::new();
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
    loop {
        // println!("time: {}", time);
        // if a new I/O arrived at the system at this current time
        //      → add request to IO-queue
        if io_ptr < io_requests.len() && io_requests[io_ptr].time == time {
            io_queue.push_back(io_ptr);
            v_trace!("{}: {} add {}", time, io_ptr, io_requests[io_ptr].track);
            io_ptr += 1;
        }
        
        // If an IO is active and completed at this time
        // → Compute relevant info and store in IO request for final summary
        if let Some(io_id) = active_io {
            let io = &mut io_requests[io_id];
            if track_head == io.track {
                io.completed_time = Some(time);
                v_trace!("{}: {} finish {}", time, io_id, time - io.start_time.unwrap());
                active_io = None;
            }
        }

        // If no IO request is active now
        // If requests are pending
        // → Fetch the next request from IO-queue and start the new IO.
        // Else if all IO from input file processed
        // → exit simulation
        if active_io.is_none() {
            if let Some(io_id) = io_queue.pop_front() {
                let io = &mut io_requests[io_id];
                io.start_time = Some(time);
                active_io = Some(io_id); 
                v_trace!("{}: {} issue {} {}", time, io_id, io.track, track_head);
            } else if io_ptr >= io_requests.len() {
                break;
            }
        }

        // If an IO is active
        // → Move the head by one unit in the direction it's going (to simulate seek)
        if let Some(io_id) = active_io {
            let io = &mut io_requests[io_id];
            if track_head < io.track {
                track_head += 1;
            } else if track_head > io.track {
                track_head -= 1;
            }
        }

        time += 1;
    }
    // Print end stats
    print_summary(&io_requests, &stats);
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
                .takes_value(true),
        )
        .arg(
            Arg::with_name("q_flag")
                .short('q')
                .required(false)
                .help("q_flag")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("f_flag")
                .short('f')
                .required(false)
                .help("f_flag")
                .takes_value(true),
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
        tflags.v_option = true; //matches.is_present("v_flag");
        tflags.q_option = matches.is_present("q_flag");
        tflags.f_option = matches.is_present("f_flag");
    });

    (algorithm, inputfile)
}

fn get_default_args() -> Vec<String> {
    vec![
        "ioshed".to_string(),
        "-sf".to_string(),
        // "-v".to_string(),
        "../lab4_assign/input0".to_string(),
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

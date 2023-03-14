mod process;
use process::Process;
use std::env;
// use std::path::PathBuf;
use regex::Regex;
mod schedulers;
use schedulers::{Scheduler};

mod rand_generator;
use rand_generator::RandGenerator;

use std::fs::File;
use std::io::Read;

fn main() {
    let args: Vec<String> = env::args().collect();

    let matches = clap::App::new("Scheduler algorithms for OS")
        .version("1.0")
        .author("Your Name")
        .arg(clap::Arg::with_name("inputfile")
            .long("inputfile")
            .takes_value(true)
            .default_value("../lab2_assign/input1")
            .help("Process array input file"))
        .arg(clap::Arg::with_name("rfile")
            .long("rfile")
            .takes_value(true)
            .default_value("../lab2_assign/rfile")
            .help("random number file"))
        .arg(clap::Arg::with_name("schedspec")
            .short("s")
            .long("schedspec")
            .takes_value(true)
            .validator(valid_schedspec)
            .help("Scheduler specification (F, L, S, or R<num>)"))
        .get_matches();

    let inputfile = matches.value_of("inputfile").unwrap();
    let rfile = matches.value_of("rfile").unwrap();
    let schedspec = matches.value_of("schedspec");

    let mut args = Vec::new();
    if let Some(spec) = schedspec {
        args.push("-s".to_owned());
        args.push(spec.to_owned());
    }
    args.push("--inputfile".to_owned());
    args.push(inputfile.to_owned());
    args.push("--rfile".to_owned());
    args.push(rfile.to_owned());

    // call main with the args vector
    main_with_args(args);
}

fn valid_schedspec(value: String) -> Result<(), String> {
    let re = Regex::new(r"^([FLS]|[R|P|E]\d+(:\d+)?)$").unwrap();
    if !re.is_match(&value) {
        Err(format!("Invalid scheduler specification: {}. Must be one of F, L, S, R<num>, P<num>:<num> or E<num>:<num>", value))
    } else {
        Ok(())
    }
}


fn get_process_array(filename: &str, randgen:&RandGenerator, maxprio:usize) -> Vec<Process> {
    let mut file = File::open(filename).expect(&format!("Failed to open file: {}", filename));
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut lines = contents.lines().enumerate();
    lines
        .map(|(id, line)| {
            let mut parts = line.split_whitespace();
            let at = parts.next().unwrap().parse().unwrap();
            let tc = parts.next().unwrap().parse().unwrap();
            let cb = parts.next().unwrap().parse().unwrap();
            let io = parts.next().unwrap().parse().unwrap();
            Process::new(id, at, tc, cb, io, randgen, maxprio)
        })
        .collect()
}

struct Event {
    time: usize,
    process: &Process,
    change_to: ProcState,
}

enum ProcState {
    CREATED,
    READY,
    RUNNG,
    BLOCK,
    DONE
}

enum Transition {
    CREATED_TO_READY,
    READY_TO_RUNNG,
    RUNNG_TO_BLOCK,
    RUNNG_TO_READY,
    BLOCK_TO_READY,
    DONE,
}

struct DES {
    process_arr: Vec<Process>,
    event_queue: Vec<Event>,
    scheduler: Box<dyn Scheduler>,
}

impl DES {
    fn new(process_arr: Vec<Process>, scheduler: Box<dyn Scheduler>) -> Self {
        DES {
            process_arr,
            event_queue: Vec<Event>::new(),
            scheduler,
        }
    }

    fn run(&mut self) {
        while let Some(event) = self.event_queue.pop() {
            let process = event.process;
            match event.transition {
                Transition::CREATED_TO_READY => {
                    process.state = ProcState::READY;
                    self.scheduler.add_process(process);
                    call_scheduler = true;
                },
                Transition::READY_TO_RUNNG => {
                    process.state = ProcState::RUNNG;
                    process.state_start = event.clock
                    self.cur_running_process = Some(&event.process);

                    let cpuburst = rand_generator.next(process.cb).min(process.remaining_time);
                    process.current_cpu_burst = cpuburst;

                    self.scheduler.add_process(process);

                },
                Transition::RUNNG_TO_BLOCK => {
                    process.state = ProcState::BLOCK;
                    process.state_start = event.clock
                    self.cur_running_process = None;

                    let ioburst = rand_generator.next(process.io).min(process.remaining_time);
                    process.current_io_burst = ioburst;

                    self.scheduler.add_process(process);

                },
                Transition::RUNNG_TO_READY => {
                    process.state = ProcState::READY;
                    process.state_start = event.clock
                    self.cur_running_process = None;

                    self.scheduler.add_process(process);

                },
                Transition::BLOCK_TO_READY => {
                    process.state = ProcState::READY;
                    process.state_start = event.clock
                    self.cur_running_process = None;

                    self.scheduler.add_process(process);

                },
                Transition::DONE => {
                    process.state = ProcState::DONE;
                    process.state_start = event.clock
                    self.cur_running_process = None;

                    self.scheduler.add_process(process);

                },
            }

            if call_scheduler {
                let next_process = self.scheduler.get_next_process();
                if let Some(next_process) = next_process {
                    let next_event = Event {
                        time: event.clock,
                        process: next_process,
                        transition: Transition::READY_TO_RUNNG,
                    };
                    self.event_queue.push(next_event);
                }
            }
        }
    }
}

fn main_with_args(args: Vec<String>) {
    // main code here
    // match args.get(2).map(|s| &**s) {
        // Some("F") => scheduler = Box::new(FCFS::new()),
        // Some("L") => scheduler = Box::new(LCFS::new()),
        // Some("S") => scheduler = Box::new(SRTF::new()),
        // Some(s) if s.starts_with("R") => scheduler = Box::new(RR::new(s[1..].parse::<usize>().unwrap())),
        // Some(s) if s.starts_with("P") => {
        //     let parts = s[1..].split(":").collect::<Vec<&str>>();
        //     let quantum = parts[0].parse::<usize>().unwrap();
        //     let maxprio = parts.get(1).map(|&s| s.parse::<usize>().unwrap_or(4)).unwrap_or(4);
        //     scheduler = Box::new(PRIO::new(quantum, maxprio));
        // },
        // _ => panic!("Invalid scheduler specification"),
    // }

    let maxprio = 4;
    let rand_generator = RandGenerator::new(&args[3]);
    let process_arr = get_process_array(args[1].as_str(), &rand_generator, maxprio);
    let scheduler = Scheduler::new();

    let des = DES::new(process_arr, scheduler);

    
    
    // simulation(&des, &rand_generator, &process_arr, &*scheduler);
    // print_summary(&*scheduler, &process_arr);
    
    print!("");
}

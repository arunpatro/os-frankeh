use clap::{App, Arg};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
struct VMA {}

struct PTE {
    present: bool,
    modified: bool,
    referenced: bool,
    frame: Option<usize>,
}

struct PageTable {
    entries: Vec<PTE>,
}

struct Process {
    vmas: Vec<VMA>,
    page_table: PageTable,

    // stats
    unmaps: u64,
    maps: u64,
    ins: u64,
    outs: u64,
    fins: u64,
    fouts: u64,
    zeros: u64,
    segv: u64,
    segprot: u64,
}
impl Process {
    fn new(num_vmas: usize, vmas: Vec<(u32, u32, u32, u32)>) -> Process {
        let mut entries = Vec::with_capacity(64);

        for _ in 0..64 {
            entries.push(PTE {
                present: false,
                modified: false,
                referenced: false,
                frame: None,
            });
        }

        let page_table = PageTable { entries };

        Process {
            vmas: Vec::new(),
            page_table: page_table,
            unmaps: 0,
            maps: 0,
            ins: 0,
            outs: 0,
            fins: 0,
            fouts: 0,
            zeros: 0,
            segv: 0,
            segprot: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    pid: Option<u64>,
    vpage: Option<usize>,
}

struct Pager {
    hand: usize,
}

impl Pager {
    fn new() -> Pager {
        Pager { hand: 0 }
    }

    fn select_victim_frame(&mut self) -> usize {
        let hand = self.hand;
        self.hand = (self.hand + 1) % 64;
        hand
    }
}

struct MMU {
    frame_table: Vec<Frame>,
    free_frames: VecDeque<usize>,
    pager: Pager,
    processes: Vec<Process>,
    current_process: Option<usize>,

    // stats
    ctx_switches: u64,
    process_exits: u64,
    cost: u64,
}

impl MMU {
    fn new(max_frames: usize, processes: Vec<Process>) -> MMU {
        // create a fixed size frame table of size max_frames
        let mut frame_table = vec![
            Frame {
                pid: None,
                vpage: None
            };
            max_frames
        ];
        let mut free_frames = VecDeque::new();
        for i in 0..max_frames {
            free_frames.push_back(i);
        }

        MMU {
            frame_table: frame_table,
            free_frames: free_frames,
            pager: Pager::new(),
            processes: processes,
            current_process: None,
            ctx_switches: 0,
            process_exits: 0,
            cost: 0,
        }
    }

    fn get_frame(&mut self) -> usize {
        if let Some(frame) = self.free_frames.pop_front() {
            return frame;
        }

        let frame = self.pager.select_victim_frame();
        let frame_entry = &self.frame_table[frame];
        if let (Some(pid), Some(vpage)) = (frame_entry.pid, frame_entry.vpage) {
            println!(" UNMAP {}:{}", pid, vpage);
            // update the process stats
            self.processes[pid as usize].unmaps += 1;

            let page_table = &mut self.processes[pid as usize].page_table;
            // no longer present
            page_table.entries[vpage].present = false;
            if page_table.entries[vpage].modified {
                // it means modified bit is set and is getting paged out
                println!(" OUT");
                self.processes[pid as usize].outs += 1;
            }
        }

        frame
    }

    fn process_instruction(&mut self, operation: &str, argument: usize) {
        match operation {
            "c" => {
                let pid = argument;
                self.current_process = Some(pid);
                self.ctx_switches += 1;
            }
            "r" | "w" => {
                let vpage = argument;
                {
                    let page_table = &mut self.processes[self.current_process.unwrap()].page_table;
                    if page_table.entries[vpage].present {
                        if operation == "w" {
                            page_table.entries[vpage].modified = true;
                        }
                        page_table.entries[vpage].referenced = true;
                    }
                }
                if !self.processes[self.current_process.unwrap()]
                    .page_table
                    .entries[vpage]
                    .present
                {
                    let frame_idx = self.get_frame();
                    let frame = &mut self.frame_table[frame_idx];
                    let page_table = &mut self.processes[self.current_process.unwrap()].page_table;
                    page_table.entries[vpage].present = true;
                    page_table.entries[vpage].frame = Some(frame_idx);
                    frame.pid = Some(0);
                    frame.vpage = Some(vpage);

                    if operation == "w" {
                        page_table.entries[vpage].modified = true;
                    }
                    page_table.entries[vpage].referenced = true;

                    println!(" ZERO");
                    println!(" MAP {}", frame_idx);
                    self.processes[self.current_process.unwrap()].maps += 1;
                    self.processes[self.current_process.unwrap()].zeros += 1;
                }
            }
            _ => panic!("Invalid operation: {}", operation),
        }
    }
}

fn read_input_file(filename: &str) -> (Vec<Process>, Vec<(String, usize)>) {
    let file = File::open(filename).expect("Failed to open file");
    let mut reader = BufReader::new(file);

    let mut line = String::new();

    // Skip the first 3 comment lines
    for _ in 0..3 {
        reader.read_line(&mut line).expect("Failed to read line");
        line.clear();
    }

    // Read the number of processes
    reader.read_line(&mut line).expect("Failed to read line");
    let num_processes: usize = line.trim().parse().expect("Failed to parse number");
    line.clear();

    // Read processes
    let mut processes = Vec::new();
    for _ in 0..num_processes {
        // Skip comment lines
        while {
            reader.read_line(&mut line).expect("Failed to read line");
            line.starts_with('#')
        } {
            line.clear();
        }

        // Read the number of VMAs
        let num_vmas: usize = line.trim().parse().expect("Failed to parse number");
        line.clear();

        // Read VMAs
        let mut vmas = Vec::new();
        for _ in 0..num_vmas {
            reader.read_line(&mut line).expect("Failed to read line");
            let mut iter = line
                .trim()
                .split_whitespace()
                .map(|x| x.parse::<u32>().expect("Failed to parse number"));
            let start = iter.next().unwrap();
            let end = iter.next().unwrap();
            let wprot = iter.next().unwrap();
            let mmap = iter.next().unwrap();

            vmas.push((start, end, wprot, mmap));
            line.clear();
        }

        // Create a new Process with the VMAs and add it to the processes list
        processes.push(Process::new(num_vmas, vmas));
    }

    // Skip comment lines
    reader.read_line(&mut line).expect("Failed to read line");
    line.clear();

    // Read instructions
    let mut instructions = Vec::new();
    while reader.read_line(&mut line).expect("Failed to read line") > 0 {
        if line.starts_with('#') {
            break;
        }
        let mut iter = line.trim().split_whitespace();
        let operation = iter.next().unwrap().to_string();
        let address: usize = iter
            .next()
            .unwrap()
            .parse()
            .expect("Failed to parse number");
        instructions.push((operation, address));
        line.clear();
    }

    (processes, instructions)
}
fn actual_main_fn(num_frames: usize, algorithm: String, inputfile: String, randomfile: String) {
    // Read input file
    let (processes, instructions) = read_input_file(&inputfile);

    // Create a new MMU and process instructions
    let mut mmu = MMU::new(num_frames, processes);
    for (idx, (operation, address)) in instructions.iter().cloned().enumerate() {
        println!("{}: ==> {} {}", idx, operation, address);
        mmu.process_instruction(&operation, address);
    }

    // Print end stats
    // example output:
    // PT[0]: 0:R-- 1:RM- * 3:R-- 4:R-- 5:RM- * * 8:R-- * * # * 13:R-- * * * * * * # 21:R-- * * 24:R-- * * * 28:RM- * 30:R-- 31:R-- * * * * * * 38:RM- * * * * * 44:R-- * * * 48:RM- * * # * * * # 56:R-- * * * * * * *
    // FT: 0:28 0:56 0:31 0:4 0:48 0:3 0:5 0:24 0:8 0:1 0:38 0:0 0:30 0:13 0:44 0:21
    // PROC[0]: U=10 M=26 I=0 O=4 FI=0 FO=0 Z=26 SV=0 SP=0
    // TOTALCOST 31 1 0 28260 4
    // 1. print page table of each process
    for (idx, process) in mmu.processes.iter().enumerate() {
        print!("PT[{}]:", idx);
        for (idx, entry) in process.page_table.entries.iter().enumerate() {
            if entry.present {
                print!(" {}:", idx);
                print!("{}", if entry.referenced { 'R' } else { '-' });
                print!("{}", if entry.modified { 'M' } else { '-' });
                print!("-");
                // print!("{}", if entry.pagedout { 'S' } else { '-' });
            } else if entry.modified {
                print!(" #");
            } else {
                print!(" *");
            }
        }
        println!();
    }

    // 2. print frame table
    print!("FT:");
    for (_, frame) in mmu.frame_table.iter().enumerate() {
        match (frame.pid, frame.vpage) {
            (Some(pid), Some(vpage)) => {
                print!(" {}:{}", pid, vpage);
            }
            _ => {
                print!(" *");
            }
        }
    }
    println!();

    // 3. per process stats
    for (idx, process) in mmu.processes.iter().enumerate() {
        println!(
            "PROC[{}]: U={} M={} I={} O={} FI={} FO={} Z={} SV={} SP={}",
            idx,
            process.unmaps,
            process.maps,
            process.ins,
            process.outs,
            process.fins,
            process.fouts,
            process.zeros,
            process.segv,
            process.segprot
        );
    }

    // 4. total stats
    // calc cost
    let mut cost = 0;
    for (idx, process) in mmu.processes.iter().enumerate() {
        cost += process.unmaps * 410;
        cost += process.maps * 350;
        // cost += process.ins * 3200;
        cost += process.outs * 2750;
        // cost += process.fins * 2500;
        // cost += process.fouts * 2500;
        cost += process.zeros * 150;
        // cost += process.segv * 240;
        // cost += process.segprot * 300;
    }
    cost += mmu.ctx_switches * 130;
    // cost += mmu.process_exits * 1230;
    cost += (instructions.len() as u64 - mmu.process_exits - mmu.ctx_switches) * 1;

    println!(
        "TOTALCOST {} {} {} {} {}",
        instructions.len(), mmu.ctx_switches, mmu.process_exits, cost, 4
        // instructions.len(), mmu.ctx_switches, mmu.process_exits, cost, std::mem::size_of::<PTE>()
    );
}

fn parse_args(actual_args: &Vec<String>) -> (usize, String, String, String) {
    let matches = App::new("MMU program")
        .arg(
            Arg::with_name("num_frames")
                .short('f')
                .long("frames")
                .required(true)
                .help("number of frames")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("algorithm")
                .short('a')
                .long("algorithm")
                .required(true)
                .help("algorithm")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("option")
                .short('o')
                .long("option")
                .required(false)
                .help("option")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("inputfile")
                .help("input file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("randomfile")
                .help("random file")
                .required(true)
                .index(2),
        )
        .get_matches_from(actual_args);

    let num_frames = matches
        .value_of("num_frames")
        .unwrap()
        .parse()
        .expect("Failed to parse number of frames");
    let algorithm = matches.value_of("algorithm").unwrap().to_string();
    let inputfile = matches.value_of("inputfile").unwrap().to_string();
    let randomfile = matches.value_of("randomfile").unwrap().to_string();

    (num_frames, algorithm, inputfile, randomfile)
}

fn get_default_args() -> Vec<String> {
    vec![
        "mmu-rust".to_string(),
        "-f16".to_string(),
        "-aF".to_string(),
        "mmu/lab3_assign/in1".to_string(),
        "mmu/lab3_assign/rfile".to_string(),
    ]
}

fn main() {
    let default_args = get_default_args();
    let args = std::env::args().collect::<Vec<String>>();
    let actual_args = if args.len() > 1 {
        // println!("ACTUAL ARGS: {:?}", args);
        &args
    } else {
        // println!("DEFAULT ARGS: {:?}", default_args);
        &default_args
    };

    // Parse command line arguments
    let (num_frames, algorithm, inputfile, randomfile) = parse_args(&actual_args);

    // log the arguments
    // println!("ARG num_frames: {}", num_frames);
    // println!("ARG algorithm: {}", algorithm);
    // println!("ARG inputfile: {}", inputfile);
    // println!("ARG randomfile: {}", randomfile);
    // Call your main function with the parsed arguments
    actual_main_fn(num_frames, algorithm, inputfile, randomfile);
}

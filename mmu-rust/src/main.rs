use clap::{App, Arg};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::rc::Rc;

#[derive(Debug)]
struct VMA {
    start: usize,
    end: usize,
    write_protected: bool,
    file_mapped: bool,
}
impl VMA {
    fn new(start: usize, end: usize, write_protected: bool, file_mapped: bool) -> VMA {
        VMA {
            start: start,
            end: end,
            write_protected: write_protected,
            file_mapped: file_mapped,
        }
    }
}

struct PTE {
    frame: Option<usize>,
    present: bool,
    referenced: bool,
    modified: bool,
    write_protected: bool,
    paged_out: bool,
    file_mapped: bool,
    is_valid_vma: bool,
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
    fn new(vmas: Vec<VMA>) -> Process {
        let mut entries = Vec::with_capacity(64);

        // initialize all the page table entries to not present, and set write_protected if the vma is write protected
        for vpage in 0..64 {
            let vma = vmas
                .iter()
                .find(|vma| vma.start <= vpage && vpage <= vma.end);
            let write_protected = vma.map_or(false, |vma| vma.write_protected);
            let file_mapped = vma.map_or(false, |vma| vma.file_mapped);
            entries.push(PTE {
                frame: None,
                present: false,
                referenced: false,
                modified: false,
                write_protected: write_protected,
                paged_out: false,
                file_mapped: file_mapped,
                is_valid_vma: vma.is_some(),
            });
        }

        let page_table = PageTable { entries };

        Process {
            vmas: vmas,
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

trait Pager {
    fn select_victim_frame(&mut self) -> usize;
}

struct FIFO {
    hand: usize,
    num_frames: usize,
}
impl FIFO {
    fn new(num_frames: usize) -> FIFO {
        FIFO {
            hand: 0,
            num_frames: num_frames,
        }
    }
}

impl Pager for FIFO {
    fn select_victim_frame(&mut self) -> usize {
        let frame = self.hand;
        self.hand = (self.hand + 1) % self.num_frames;
        frame
    }
}

struct Random {
    hand: usize,
    num_frames: usize,
    random_numbers: Vec<usize>,
}

impl Random {
    fn new(num_frames: usize, random_numbers: Vec<usize>) -> Random {
        Random {
            hand: 0,
            num_frames: num_frames,
            random_numbers: random_numbers,
        }
    }
}

impl Pager for Random {
    fn select_victim_frame(&mut self) -> usize {
        let frame = self.random_numbers[self.hand] % self.num_frames;
        self.hand = (self.hand + 1) % self.random_numbers.len();
        frame
    }
}

struct MMU {
    frame_table: Rc<RefCell<Vec<Frame>>>,
    free_frames: VecDeque<usize>,
    pager: Box<dyn Pager>,
    processes: Vec<Process>,
    current_process: Option<usize>,

    // stats
    ctx_switches: u64,
    process_exits: u64,
    cost: u64,
}

impl MMU {
    fn new(max_frames: usize, pager: Box<dyn Pager>, processes: Vec<Process>) -> MMU {
        // create a fixed size frame table of size max_frames
        let frame_table = Rc::new(RefCell::new(vec![
            Frame {
                pid: None,
                vpage: None,
            };
            max_frames
        ]));

        let mut free_frames = VecDeque::new();
        for i in 0..max_frames {
            free_frames.push_back(i);
        }

        MMU {
            frame_table: frame_table,
            free_frames: free_frames,
            pager: pager,
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

        let frame_idx = self.pager.select_victim_frame();
        let frame = self.frame_table.borrow()[frame_idx];
        if let (Some(pid), Some(vpage)) = (frame.pid, frame.vpage) {
            println!(" UNMAP {}:{}", pid, vpage);
            // update the process stats
            self.processes[pid as usize].unmaps += 1;

            let page = &mut self.processes[pid as usize].page_table.entries[vpage];
            // no longer present
            page.present = false;
            if page.modified {
                page.modified = false;
                // check if it is file mapped for OUT or FOUT
                if page.file_mapped {
                    println!(" FOUT");
                    self.processes[pid as usize].fouts += 1;
                } else {
                    // it means modified bit is set and is getting paged out
                    page.paged_out = true;
                    println!(" OUT");
                    self.processes[pid as usize].outs += 1;
                }
            }
        }

        frame_idx
    }

    fn page_fault_handler(&mut self, vpage: usize) {}

    fn process_instruction(&mut self, operation: &str, argument: usize) {
        match operation {
            "c" => {
                let pid = argument;
                self.current_process = Some(pid);
                self.ctx_switches += 1;
            }
            "r" | "w" => {
                let vpage = argument;
                if !self.processes[self.current_process.unwrap()]
                    .page_table
                    .entries[vpage]
                    .present
                {
                    // TODO page fault so call the page fault handler
                    // self.page_fault_handler(vpage);

                    // 1. check if page can be accessed i.e. it is in the vma, if not then segv and move on
                    {
                        let proc = &mut self.processes[self.current_process.unwrap()];
                        let vma = proc
                            .vmas
                            .iter()
                            .find(|vma| vma.start <= vpage && vpage <= vma.end);
                        if vma.is_none() {
                            println!(" SEGV");
                            proc.segv += 1;
                            return;
                        }
                    }

                    // 3. if valid then assign a frame
                    let frame_idx = self.get_frame();
                    let frame = &mut self.frame_table.borrow_mut()[frame_idx];
                    let proc = &mut self.processes[self.current_process.unwrap()];
                    let page = &mut proc.page_table.entries[vpage];
                    page.present = true;
                    page.frame = Some(frame_idx);
                    frame.pid = Some(self.current_process.unwrap() as u64);
                    frame.vpage = Some(vpage);

                    // 4. populate it -
                    //    if paged out then we need bring it "IN" from swap space
                    //    or "FIN" if it is memory mapped ?? what is this?
                    //    else zero it

                    if page.file_mapped {
                        println!(" FIN");
                        proc.fins += 1;
                    } else if page.paged_out {
                        println!(" IN");
                        proc.ins += 1;
                    } else {
                        println!(" ZERO");
                        proc.zeros += 1;
                    }
                    println!(" MAP {}", frame_idx);
                    proc.maps += 1;

                    page.referenced = true;
                }

                // finally update the reference and modified bits
                if let Some(current_process) = self.current_process {
                    let proc = &mut self.processes[current_process];
                    let page = &mut proc.page_table.entries[vpage];
                    page.referenced = true;

                    if operation == "w" {
                        if page.write_protected {
                            println!(" SEGPROT");
                            proc.segprot += 1;
                            return;
                        } else {
                            page.modified = true;
                        }
                    }
                }
            }
            "e" => {
                let pid = argument;
                println!("EXIT current process {}", pid);
                self.process_exits += 1;
                let proc = &mut self.processes[pid];
                for vpage in 0..64 {
                    let page = &mut proc.page_table.entries[vpage];
                    page.paged_out = false; // reset paged out bit
                    if page.present {
                        let frame_idx = page.frame.unwrap();
                        let frame = &mut self.frame_table.borrow_mut()[frame_idx];
                        frame.pid = None;
                        frame.vpage = None;
                        self.free_frames.push_back(frame_idx);
                        page.present = false;
                        page.frame = None;
                        println!(" UNMAP {}:{}", pid, vpage);
                        proc.unmaps += 1;

                        // all the below code is not required i think
                        if page.modified {
                            if page.file_mapped {
                                println!(" FOUT");
                                proc.fouts += 1;
                            } else {
                                // println!(" OUT");
                                // proc.outs += 1;
                            }
                        }
                    }
                }
            }
            _ => panic!("Invalid operation: {}", operation),
        }
    }
}
fn read_random_file(filename: &str) -> Vec<usize> {
    // The format is: the 1st line is the number of random numbers, and the rest are the numbers
    let file = File::open(filename).expect("Failed to open file");
    let reader = BufReader::new(file);

    let mut lines = reader.lines();
    let count: usize = lines
        .next()
        .expect("Failed to read the first line")
        .expect("Failed to read the first line")
        .trim()
        .parse()
        .expect("Failed to parse the number of random numbers");

    let mut random_numbers = Vec::with_capacity(count);
    for line in lines {
        let number: usize = line
            .expect("Failed to read a line")
            .trim()
            .parse()
            .expect("Failed to parse a random number");
        random_numbers.push(number);
    }

    random_numbers
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
                .map(|x| x.parse::<usize>().expect("Failed to parse number"));
            let start = iter.next().unwrap();
            let end = iter.next().unwrap();
            let wprot = iter.next().unwrap() == 1;
            let mmap = iter.next().unwrap() == 1;

            vmas.push(VMA::new(start, end, wprot, mmap));
            line.clear();
        }

        // Create a new Process with the VMAs and add it to the processes list
        processes.push(Process::new(vmas));
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
fn actual_main_fn(num_frames: usize, algorithm: &str, inputfile: &str, randomfile: &str) {
    // Read input file
    let (processes, instructions) = read_input_file(&inputfile);

    // Create a new MMU and a pager based on the algorithm and process instructions
    let pager = match algorithm {
        "f" => Box::new(FIFO::new(num_frames)) as Box<dyn Pager>,
        "r" => {
            let random_numbers = read_random_file(&randomfile);
            // println!("Random numbers: {:?}", random_numbers);
            Box::new(Random::new(num_frames, random_numbers))
        }
        // "c" => Pager::Clock,
        // "e" => Pager::NRU,
        // "a" => Pager::Aging,
        _ => panic!("Invalid algorithm: {}", algorithm),
    };

    let mut mmu = MMU::new(num_frames, pager, processes);
    for (idx, (operation, address)) in instructions.iter().cloned().enumerate() {
        println!("{}: ==> {} {}", idx, operation, address);
        mmu.process_instruction(&operation, address);
    }

    // Print end stats
    // 1. print page table of each process
    for (idx, process) in mmu.processes.iter().enumerate() {
        print!("PT[{}]:", idx);
        for (idx, entry) in process.page_table.entries.iter().enumerate() {
            if entry.paged_out && !entry.present && !entry.file_mapped {
                print!(" #");
            } else if !entry.is_valid_vma || !entry.present {
                print!(" *");
            } else {
                print!(" {}:", idx);
                print!("{}", if entry.referenced { 'R' } else { '-' });
                print!("{}", if entry.modified { 'M' } else { '-' });
                print!("{}", if entry.paged_out { 'S' } else { '-' });
            }
        }
        println!();
    }

    // 2. print frame table
    print!("FT:");
    let frame_table = mmu.frame_table.as_ref().borrow();
    for frame in frame_table.iter() {
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
    for process in mmu.processes.iter() {
        cost += process.unmaps * 410;
        cost += process.maps * 350;
        cost += process.ins * 3200;
        cost += process.outs * 2750;
        cost += process.fins * 2350;
        cost += process.fouts * 2800;
        cost += process.zeros * 150;
        cost += process.segv * 440;
        cost += process.segprot * 410;
    }
    cost += mmu.ctx_switches * 130;
    cost += mmu.process_exits * 1230;
    cost += (instructions.len() as u64 - mmu.process_exits - mmu.ctx_switches) * 1;

    println!(
        "TOTALCOST {} {} {} {} {}",
        instructions.len(),
        mmu.ctx_switches,
        mmu.process_exits,
        cost,
        4 // instructions.len(), mmu.ctx_switches, mmu.process_exits, cost, std::mem::size_of::<PTE>()
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
        "-ar".to_string(),
        "../mmu/lab3_assign/in1".to_string(),
        "../mmu/lab3_assign/rfile".to_string(),
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
    actual_main_fn(num_frames, &algorithm, &inputfile, &randomfile);
}

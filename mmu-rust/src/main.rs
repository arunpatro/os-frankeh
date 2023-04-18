use clap::{App, Arg};
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
    present: bool,
    modified: bool,
    referenced: bool,
    write_protected: bool,
    paged_out: bool,
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
    fn new(vmas: Vec<VMA>) -> Process {
        let mut entries = Vec::with_capacity(64);

        // initialize all the page table entries to not present, and set write_protected if the vma is write protected
        for vpage in 0..64 {
            let vma = vmas
                .iter()
                .find(|vma| vma.start <= vpage && vpage <= vma.end);
            let write_protected = vma.map_or(false, |vma| vma.write_protected);
            entries.push(PTE {
                present: false,
                modified: false,
                referenced: false,
                write_protected: write_protected,
                paged_out: false,
                frame: None,
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

struct Pager {
    hand: usize,
    num_frames: usize,
}

impl Pager {
    fn new(num_frames: usize) -> Pager {
        Pager {
            hand: 0,
            num_frames: num_frames,
        }
    }

    fn select_victim_frame(&mut self) -> usize {
        let hand = self.hand;
        self.hand = (self.hand + 1) % self.num_frames;
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
            pager: Pager::new(max_frames),
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

            let page_file_mapped = self.processes[pid as usize]
                .vmas
                .iter()
                .find(|vma| vma.start <= vpage && vpage <= vma.end)
                .unwrap()
                .file_mapped;

            let page = &mut self.processes[pid as usize].page_table.entries[vpage];
            // no longer present
            page.present = false;
            if page.modified {
                page.modified = false;
                // check if it is file mapped for OUT or FOUT
                if page_file_mapped {
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
                if !self.processes[self.current_process.unwrap()]
                    .page_table
                    .entries[vpage]
                    .present
                {
                    // TODO page fault so call the page fault handler

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
                    let frame = &mut self.frame_table[frame_idx];
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

                    // check if page is file mapped from the vma
                    let page_file_mapped = proc
                        .vmas
                        .iter()
                        .find(|vma| vma.start <= vpage && vpage <= vma.end)
                        .unwrap()
                        .file_mapped;

                    if page_file_mapped {
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

                    // 2. check if page is writeable if not then segprot and move on
                    if operation == "w" && page.write_protected {
                        println!(" SEGPROT");
                        proc.segprot += 1;
                        return;
                    }

                    if operation == "w" {
                        page.modified = true;
                    }
                    page.referenced = true;
                }

                // finally update the reference and modified bits
                {
                    let page_table = &mut self.processes[self.current_process.unwrap()].page_table;
                    if page_table.entries[vpage].present {
                        if operation == "w" {
                            page_table.entries[vpage].modified = true;
                        }
                        page_table.entries[vpage].referenced = true;
                    }
                }
            }
            "e" => {
                let pid = argument;
                self.process_exits += 1;
                let proc = &mut self.processes[pid];
                for vpage in 0..64 {
                    if proc.page_table.entries[vpage].present {
                        let frame_idx = proc.page_table.entries[vpage].frame.unwrap();
                        let frame = &mut self.frame_table[frame_idx];
                        frame.pid = None;
                        frame.vpage = None;
                        self.free_frames.push_back(frame_idx);
                        proc.page_table.entries[vpage].present = false;
                        proc.page_table.entries[vpage].frame = None;
                        println!(" UNMAP {}:{}", pid, vpage);
                        proc.unmaps += 1;
                        if proc.page_table.entries[vpage].modified {
                            // check if it is file mapped
                            let page_file_mapped = proc
                                .vmas
                                .iter()
                                .find(|vma| vma.start <= vpage && vpage <= vma.end)
                                .unwrap()
                                .file_mapped;
                            if page_file_mapped {
                                println!(" FOUT");
                                proc.fouts += 1;
                            } else {
                                println!(" OUT");
                                proc.outs += 1;
                            }
                        }
                    }
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
    // 1. print page table of each process
    for (idx, process) in mmu.processes.iter().enumerate() {
        print!("PT[{}]:", idx);
        for (idx, entry) in process.page_table.entries.iter().enumerate() {
            if entry.present {
                print!(" {}:", idx);
                print!("{}", if entry.referenced { 'R' } else { '-' });
                print!("{}", if entry.modified { 'M' } else { '-' });
                print!("{}", if entry.paged_out { 'S' } else { '-' });
            } else if entry.paged_out {
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
        "-aF".to_string(),
        "../mmu/lab3_assign/in5".to_string(),
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
    actual_main_fn(num_frames, algorithm, inputfile, randomfile);
}

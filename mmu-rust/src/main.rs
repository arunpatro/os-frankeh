use clap::{App, Arg};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
struct VMA {}

struct PTE {
    present: bool,
    modified: bool,
    referenced: bool,
    paged_out: bool,
    frame: Option<u64>,
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
        let mut page_table = PageTable {
            entries: Vec::new(),
        };
        for _ in 0..64 {
            page_table.entries.push(PTE {
                present: false,
                modified: false,
                referenced: false,
                paged_out: false,
                frame: None,
            });
        }

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
    pid: u64,
    vpage: u64,
}

struct Pager {
    hand: u64,
}

impl Pager {
    fn new() -> Pager {
        Pager { hand: 0 }
    }

    fn select_victim_frame(&mut self) -> u64 {
        let hand = self.hand;
        self.hand = (self.hand + 1) % 64;
        hand
    }
}

// TODO: use fixed lenght arrays instead of Vec
struct MMU {
    frame_table: Vec<Frame>,
    free_frames: Vec<u64>,
    page_table: PageTable,
    pager: Pager,
}

impl MMU {
    // fn new(max_frames: usize, processes: Vec<Process>) -> MMU {
    //     // create a fixed size frame table of size max_frames
    //     // let mut frame_table = [Frame; 64];

    //     // let mut free_frames = Vec::new();
    //     // for i in 0..64 {
    //     //     // frame_table.push(Frame { pid: 0, vpage: 0 });
    //     //     free_frames.push(i);
    //     // }

    //     MMU {
    //         frame_table: frame_table,
    //         free_frames: free_frames,
    //         page_table: PageTable { entries: Vec::new() },
    //         pager: Pager::new(),
    //     }
    // }
}

fn read_input_file(filename: &str) -> (Vec<Process>, Vec<(String, u32)>) {
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
        let address: u32 = iter
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
    let (processes, instructions) = read_input_file(inputfile);

    // Create a new MMU and process instructions
    // let mut mmu = MMU::new(16, processes);
    for (idx, (operation, address)) in instructions.iter().enumerate() {
        println!("{}: ==> {} {}", idx, operation, address);
        // mmu.process_instruction(operation, address);
    }

    // Print end stats
    // ...
}

fn main() {
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
            Arg::with_name("options")
                .short('o')
                .long("options")
                .help("options")
                .default_value("OPFS")
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
        .get_matches();

    let num_frames: usize = matches
        .value_of("num_frames")
        .unwrap()
        .parse()
        .expect("Failed to parse number of frames");
    let algorithm = matches.value_of("algorithm").unwrap();
    let options = matches.value_of("options").unwrap();
    let inputfile = matches.value_of("inputfile").unwrap();
    let randomfile = matches.value_of("randomfile").unwrap();

    // Call your main function with the parsed arguments
    actual_main_fn(num_frames, algorithm, inputfile, randomfile);
}

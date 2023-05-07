use clap::{App, Arg};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
mod utils;
use utils::{read_input_file, read_random_file};

// Define a struct to hold the flags
#[derive(Debug, Default)]
struct Flags {
    O_option: bool,
    P_option: bool,
    F_option: bool,
    S_option: bool,
    x_option: bool,
    y_option: bool,
    f_option: bool,
    a_option: bool,
}

thread_local! {
    static TFLAGS: RefCell<Flags> = RefCell::new(Flags::default());
}

fn print_frame_table(frame_table: Rc<RefCell<Vec<Option<Frame>>>>) {
    print!("FT:");
    let frame_table = frame_table.as_ref().borrow();
    for frame in frame_table.iter() {
        if let Some(frame) = frame {
            print!(" {}:{}", frame.pid, frame.vpage);
        } else {
            print!(" *");
        }
    }
    println!();
}

fn print_page_table(processes: Rc<RefCell<Vec<Process>>>, pid: usize) {
    print!("PT[{}]:", pid);
    let page_table = &processes.borrow()[pid].page_table;
    for (idx, entry) in page_table.entries.iter().enumerate() {
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

macro_rules! O_trace {
    ($idx:expr, $operation:expr, $address:expr) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.O_option {
                println!("{}: ==> {} {}", $idx, $operation, $address);
            }
        });
    };
}

// prints the frame table
macro_rules! F_trace {
    ($frame_table:expr) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.F_option {
                print_frame_table($frame_table);
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
                print_frame_table($frame_table);
            }
        });
    };
}

// prints the page table of current process at the end of each instruction
macro_rules! x_trace {
    ($page_table:expr, $pid:expr) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.x_option {
                print_page_table($page_table, $pid);
            }
        });
    };
}

#[macro_export]
macro_rules! a_trace {
    ($($arg:tt)*) => {
        TFLAGS.with(|tflags| {
            let tflags = tflags.borrow();
            if tflags.a_option {
                println!($($arg)*);
            }
        });
    };
}

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
            start,
            end,
            write_protected,
            file_mapped,
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

pub struct Process {
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
                write_protected,
                paged_out: false,
                file_mapped,
                is_valid_vma: vma.is_some(),
            });
        }

        let page_table = PageTable { entries };

        Process {
            vmas,
            page_table,
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

#[derive(Debug, Clone)]
struct Frame {
    pid: u64,
    vpage: usize,
    age: u32,
}

trait Pager {
    fn update_age(&mut self, _instr_idx: usize) -> u32 {
        0
    }
    fn select_victim_frame(&mut self, instr_idx: usize) -> usize;
}

struct FIFO {
    hand: usize,
    num_frames: usize,
}
impl FIFO {
    fn new(frame_table: Rc<RefCell<Vec<Option<Frame>>>>) -> FIFO {
        let num_frames = frame_table.borrow().len();
        FIFO {
            hand: 0,
            num_frames,
        }
    }
}

impl Pager for FIFO {
    fn select_victim_frame(&mut self, _instr_idx: usize) -> usize {
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
    fn new(frame_table: Rc<RefCell<Vec<Option<Frame>>>>, random_numbers: Vec<usize>) -> Random {
        let num_frames = frame_table.borrow().len();
        Random {
            hand: 0,
            num_frames,
            random_numbers,
        }
    }
}

impl Pager for Random {
    fn select_victim_frame(&mut self, _instr_idx: usize) -> usize {
        let frame = self.random_numbers[self.hand] % self.num_frames;
        self.hand = (self.hand + 1) % self.random_numbers.len();
        frame
    }
}

struct Clock {
    frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
    processes: Rc<RefCell<Vec<Process>>>,
    hand: usize,
    num_frames: usize,
}

impl Clock {
    fn new(
        frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
        processes: Rc<RefCell<Vec<Process>>>,
    ) -> Clock {
        let num_frames = frame_table.borrow().len();
        Clock {
            frame_table,
            processes,
            hand: 0,
            num_frames,
        }
    }
}

impl Pager for Clock {
    fn select_victim_frame(&mut self, _instr_idx: usize) -> usize {
        let mut frame_idx = self.hand;
        let old_hand = self.hand;
        loop {
            // get frame and then destructure and match frame to pid and vpage
            let frame = &self.frame_table.borrow()[frame_idx % self.num_frames];
            let (pid, vpage) = match frame {
                Some(frame) => (frame.pid, frame.vpage),
                None => panic!("frame table entry is empty"),
            };

            let page = &mut self.processes.borrow_mut()[pid as usize].page_table.entries[vpage];
            if page.referenced {
                page.referenced = false;
                frame_idx += 1;
            } else {
                a_trace!(
                    "ASELECT {} {}",
                    old_hand % self.num_frames,
                    frame_idx - old_hand + 1
                ); // this doesn't match with the tomasort code
                self.hand = frame_idx % self.num_frames + 1;
                return frame_idx % self.num_frames;
            }
        }
    }
}

struct NRU {
    frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
    processes: Rc<RefCell<Vec<Process>>>,
    hand: usize,
    num_frames: usize,
    instr_ckpt: usize,
}

impl NRU {
    fn new(
        frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
        processes: Rc<RefCell<Vec<Process>>>,
    ) -> NRU {
        let num_frames = frame_table.borrow().len();
        NRU {
            frame_table,
            processes,
            hand: 0,
            num_frames,
            instr_ckpt: 0,
        }
    }
}

impl Pager for NRU {
    fn select_victim_frame(&mut self, instr_idx: usize) -> usize {
        let mut classes = vec![None; 4];
        let mut frame_idx = self.hand;
        let old_hand = self.hand;
        let mut reset = 0;
        let mut class: usize;
        if instr_idx - self.instr_ckpt + 1 >= 50 {
            reset = 1;
            self.instr_ckpt = instr_idx + 1; // +1 because instr_idx starts at 0, just to offset
        }
        loop {
            // get frame and then destructure and match frame to pid and vpage
            let frame = &self.frame_table.borrow()[frame_idx % self.num_frames];
            let (pid, vpage) = match frame {
                Some(frame) => (frame.pid, frame.vpage),
                None => panic!("frame table entry is empty"),
            };

            let page = &mut self.processes.borrow_mut()[pid as usize].page_table.entries[vpage];
            class = page.referenced as usize * 2 + page.modified as usize;

            // if class is empty, set it to the current frame
            if classes[class].is_none() {
                classes[class] = Some(frame_idx % self.num_frames); // or just frame_idx
            }

            if reset == 1 {
                // reset the r bit of all the pages in the frame table
                // if reset is 1, you have to go through the whole frame table

                page.referenced = false;
            } else if classes[0].is_some() {
                // check if class 0 is present and if so, break
                break;
            }

            // one cycle through the frame table and no empty classes
            if frame_idx - old_hand == self.num_frames - 1 {
                break;
            }

            frame_idx += 1;
        }

        // choose the lowest class frame idx and set the hand to the next frame
        for i in 0..4 {
            if let Some(frame_identified) = classes[i] {
                // println!("{}", frame_idx);
                a_trace!(
                    "ASELECT: hand={:2} {} | {} {:2} {:2}",
                    old_hand,
                    reset,
                    i,
                    frame_identified,
                    frame_idx as i32 - old_hand as i32 + 1
                );
                self.hand = (frame_identified + 1) % self.num_frames;
                return frame_identified;
            }
        }

        frame_idx % self.num_frames
    }
}

struct Aging {
    frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
    processes: Rc<RefCell<Vec<Process>>>,
    hand: usize,
    num_frames: usize,
}

impl Aging {
    fn new(
        frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
        processes: Rc<RefCell<Vec<Process>>>,
    ) -> Aging {
        let num_frames = frame_table.borrow().len();
        Aging {
            frame_table,
            processes,
            hand: 0,
            num_frames,
        }
    }
}

impl Pager for Aging {
    fn select_victim_frame(&mut self, _: usize) -> usize {
        let mut frame_idx = self.hand;
        let old_hand = self.hand;
        let mut lowest_age = None;
        let mut lowest_age_frame = 0;

        // let mut frame_string = String::new(); // Initialize with empty string

        loop {
            let (pid, vpage, age);
            {
                let mut frame_table = self.frame_table.borrow_mut();
                let frame = frame_table[frame_idx % self.num_frames].as_mut().unwrap();
                pid = frame.pid;
                vpage = frame.vpage;
                frame.age >>= 1;
                // Set MSB to 1 if referenced
                let page = &mut self.processes.borrow_mut()[pid as usize].page_table.entries[vpage];
                if page.referenced {
                    frame.age |= 0x80000000;
                }
                age = frame.age;

                // add the age to the frame string by appending
                // frame_string.push_str(&format!("{}:{:x} ", frame_idx % self.num_frames, frame.age));
            }

            if let Some(current_lowest_age) = lowest_age {
                if age < current_lowest_age {
                    lowest_age = Some(age);
                    lowest_age_frame = frame_idx % self.num_frames;
                }
            } else {
                lowest_age = Some(age);
                lowest_age_frame = frame_idx % self.num_frames;
            }
            self.processes.borrow_mut()[pid as usize].page_table.entries[vpage].referenced = false;

            // One cycle through the frame table
            if frame_idx - old_hand == self.num_frames - 1 {
                break;
            }

            frame_idx += 1;
        }

        // a_trace!(
        //     "ASELECT {}-{} | {}| {}",
        //     old_hand,
        //     frame_idx % self.num_frames,
        //     frame_string,
        //     lowest_age_frame
        // );

        self.hand = (lowest_age_frame + 1) % self.num_frames;
        lowest_age_frame
    }
}

struct WorkingSet {
    frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
    processes: Rc<RefCell<Vec<Process>>>,
    hand: usize,
    num_frames: usize,
    tau: usize,
}

impl WorkingSet {
    fn new(
        frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
        processes: Rc<RefCell<Vec<Process>>>,
    ) -> WorkingSet {
        let num_frames = frame_table.borrow().len();
        WorkingSet {
            frame_table,
            processes,
            hand: 0,
            num_frames,
            tau: 50,
        }
    }
}

impl Pager for WorkingSet {
    fn update_age(&mut self, instr_idx: usize) -> u32 {
        instr_idx as u32
    }
    fn select_victim_frame(&mut self, instr_idx: usize) -> usize {
        let mut frame_idx = self.hand;
        let old_hand = self.hand;
        let mut earliest_use_time = instr_idx as u32;
        let mut earliest_use_frame = 0;
        let mut frame_string = String::new(); // Initialize with empty string

        loop {
            let mut frame_table = self.frame_table.borrow_mut();
            let frame = frame_table[frame_idx % self.num_frames].as_mut().unwrap();
            let pid = frame.pid;
            let vpage = frame.vpage;
            let page = &mut self.processes.borrow_mut()[pid as usize].page_table.entries[vpage];

            // update earliest use time and frame
            frame_string.push_str(&format!(
                " {}({} {}:{} {})",
                frame_idx % self.num_frames,
                page.referenced as u8,
                pid,
                vpage,
                frame.age
            ));

            if page.referenced {
                page.referenced = false;
                frame.age = instr_idx as u32; // reset age as last time checked
            } else if instr_idx - frame.age as usize >= self.tau {
                earliest_use_frame = frame_idx % self.num_frames;
                frame_string.push_str(&format!(" STOP({})", frame_idx - old_hand + 1));
                break;
            }

            if frame.age < earliest_use_time {
                earliest_use_time = frame.age;
                earliest_use_frame = frame_idx % self.num_frames;
            }
            // One cycle through the frame table
            if frame_idx - old_hand == self.num_frames - 1 {
                break;
            }

            frame_idx += 1;
        }

        a_trace!(
            "ASELECT {}-{} |{} | {}",
            old_hand,
            (old_hand as i32 - 1) as usize % self.num_frames,
            frame_string,
            earliest_use_frame
        );

        // return the frame with the earliest use time
        self.hand = (earliest_use_frame + 1) % self.num_frames;
        earliest_use_frame
    }
}
struct MMU {
    frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
    free_frames: Rc<RefCell<VecDeque<usize>>>,
    pager: Box<dyn Pager>,
    processes: Rc<RefCell<Vec<Process>>>,
    current_process: Option<usize>,

    // stats
    ctx_switches: u64,
    process_exits: u64,
}

impl MMU {
    fn new(
        frame_table: Rc<RefCell<Vec<Option<Frame>>>>,
        free_frames: Rc<RefCell<VecDeque<usize>>>,
        pager: Box<dyn Pager>,
        processes: Rc<RefCell<Vec<Process>>>,
    ) -> MMU {
        MMU {
            frame_table,
            free_frames,
            pager,
            processes,
            current_process: None,
            ctx_switches: 0,
            process_exits: 0,
        }
    }

    fn get_frame(&mut self, instr_idx: usize) -> usize {
        if let Some(frame) = self.free_frames.borrow_mut().pop_front() {
            return frame;
        }

        let frame_idx = self.pager.select_victim_frame(instr_idx);
        let frame_table = self.frame_table.borrow();
        let frame = &frame_table[frame_idx].as_ref().unwrap();
        // destructure frame to pid and vpage
        let (pid, vpage) = (frame.pid, frame.vpage);

        let proc = &mut self.processes.borrow_mut()[pid as usize];
        let page = &mut proc.page_table.entries[vpage];
        println!(" UNMAP {}:{}", pid, vpage);
        // update the process stats
        proc.unmaps += 1;

        page.present = false;
        if page.modified {
            page.modified = false;
            // check if it is file mapped for OUT or FOUT
            if page.file_mapped {
                println!(" FOUT");
                proc.fouts += 1;
            } else {
                // it means modified bit is set and is getting paged out
                page.paged_out = true;
                println!(" OUT");
                proc.outs += 1;
            }
        }

        frame_idx
    }

    // return Err("segv") if segv, else return Ok(())
    fn page_fault_handler(&mut self, vpage: usize, idx: usize) -> Result<(), &'static str> {
        // 1. check if page can be accessed i.e. it is in the vma, if not then segv and move on
        {
            let proc = &mut self.processes.borrow_mut()[self.current_process.unwrap()];
            let vma = proc
                .vmas
                .iter()
                .find(|vma| vma.start <= vpage && vpage <= vma.end);
            if vma.is_none() {
                println!(" SEGV");
                proc.segv += 1;
                return Err("segv");
            }
        }

        // 3. if valid then assign a frame
        let frame_idx = self.get_frame(idx);
        let proc = &mut self.processes.borrow_mut()[self.current_process.unwrap()];
        let page = &mut proc.page_table.entries[vpage];
        page.present = true;
        page.frame = Some(frame_idx);

        // TODO use the update_age method of the pagers to do this
        let mut frame_table = self.frame_table.borrow_mut();
        frame_table[frame_idx] = Some(Frame {
            pid: self.current_process.unwrap() as u64,
            vpage,
            age: self.pager.update_age(idx), // age: 0,
        });

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

        Ok(())
    }

    fn process_instruction(&mut self, idx: usize, operation: &str, argument: usize) {
        match operation {
            "c" => {
                let pid = argument;
                self.current_process = Some(pid);
                self.ctx_switches += 1;
                return; // to avoid trace
            }
            "e" => {
                let pid = argument;
                println!("EXIT current process {}", pid);
                self.process_exits += 1;
                let proc = &mut self.processes.borrow_mut()[pid];
                for vpage in 0..64 {
                    let page = &mut proc.page_table.entries[vpage];
                    page.paged_out = false; // reset paged out bit
                    if page.present {
                        let frame_idx = page.frame.unwrap();
                        let mut frame_table = self.frame_table.borrow_mut();
                        frame_table[frame_idx] = None;
                        self.free_frames.borrow_mut().push_back(frame_idx);
                        page.present = false;
                        page.frame = None;
                        println!(" UNMAP {}:{}", pid, vpage);
                        proc.unmaps += 1;

                        if page.modified && page.file_mapped {
                            println!(" FOUT");
                            proc.fouts += 1;
                        }
                    }
                }
                return; // to avoid trace
            }
            "r" | "w" => {
                let vpage = argument;
                if !self.processes.borrow_mut()[self.current_process.unwrap()]
                    .page_table
                    .entries[vpage]
                    .present
                {
                    if self.page_fault_handler(vpage, idx).is_err() {
                        return;
                    }
                }

                // now the page is definitely present
                // check write protection
                // simulate instruction execution by hardware by updating the R/M PTE bits
                // finally update the reference and modified bits
                if let Some(current_process) = self.current_process {
                    let proc = &mut self.processes.borrow_mut()[current_process];
                    let page = &mut proc.page_table.entries[vpage];
                    page.referenced = true;

                    if operation == "w" {
                        if page.write_protected {
                            println!(" SEGPROT");
                            proc.segprot += 1;
                        } else {
                            page.modified = true;
                        }
                    }
                }
            }
            _ => panic!("Invalid operation: {}", operation),
        }
        x_trace!(self.processes.clone(), self.current_process.unwrap());
        f_trace!(self.frame_table.clone());
    }
}

fn actual_main_fn(num_frames: usize, algorithm: &str, inputfile: &str, randomfile: &str) {
    // Read input file
    let (processes, instructions) = read_input_file(inputfile);

    let frame_table: Rc<RefCell<Vec<Option<Frame>>>> =
        Rc::new(RefCell::new(vec![None; num_frames]));

    // Create a list of free frames as Rc<RefCell<VecDeque<usize>>>
    let free_frames = Rc::new(RefCell::new(VecDeque::new()));
    for i in 0..num_frames {
        free_frames.borrow_mut().push_back(i);
    }

    let processes = Rc::new(RefCell::new(processes));

    // Create a new MMU and a pager based on the algorithm and process instructions
    let pager = match algorithm {
        "f" => Box::new(FIFO::new(frame_table.clone())) as Box<dyn Pager>,
        "r" => {
            let random_numbers = read_random_file(randomfile);
            // println!("Random numbers: {:?}", random_numbers);
            Box::new(Random::new(frame_table.clone(), random_numbers))
        }
        "c" => Box::new(Clock::new(frame_table.clone(), processes.clone())) as Box<dyn Pager>,

        "e" => Box::new(NRU::new(frame_table.clone(), processes.clone())) as Box<dyn Pager>,
        "a" => Box::new(Aging::new(frame_table.clone(), processes.clone())) as Box<dyn Pager>,
        "w" => Box::new(WorkingSet::new(frame_table.clone(), processes.clone())) as Box<dyn Pager>,
        _ => panic!("Invalid algorithm: {}", algorithm),
    };

    // let mut mmu = MMU::new(frame_table, free_frames, pager, processes);
    let mut mmu = MMU::new(frame_table.clone(), free_frames, pager, processes.clone());
    for (idx, (operation, address)) in instructions.iter().cloned().enumerate() {
        O_trace!(idx, operation, address);
        mmu.process_instruction(idx, &operation, address);
    }

    // Print end stats
    // 1. print page table of each process
    let P_option = { TFLAGS.with(|flags| flags.borrow().P_option) };
    if P_option {
        for idx in 0..processes.borrow().len() {
            print_page_table(processes.clone(), idx);
        }
    }

    // 2. print frame table
    F_trace!(frame_table);

    // 3. per process stats
    // need to write the S_trace! macro for this
    let S_option = { TFLAGS.with(|flags| flags.borrow().S_option) };
    if !S_option {
        return;
    }

    for (idx, process) in processes.borrow().iter().enumerate() {
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
    for process in processes.borrow().iter() {
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
    cost += instructions.len() as u64 - mmu.process_exits - mmu.ctx_switches;

    println!(
        "TOTALCOST {} {} {} {} {}",
        instructions.len(),
        mmu.ctx_switches,
        mmu.process_exits,
        cost,
        4 // std::mem::size_of::<PTE>()
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

    if let Some(option_str) = matches.value_of("option") {
        TFLAGS.with(|tflags| {
            let mut tflags = tflags.borrow_mut();
            for option in option_str.chars() {
                match option {
                    'O' => tflags.O_option = true,
                    'P' => tflags.P_option = true,
                    'F' => tflags.F_option = true,
                    'S' => tflags.S_option = true,
                    'x' => tflags.x_option = true,
                    'y' => tflags.y_option = true,
                    'f' => tflags.f_option = true,
                    'a' => tflags.a_option = true,
                    _ => (),
                }
            }
        });
    }

    (num_frames, algorithm, inputfile, randomfile)
}

fn get_default_args() -> Vec<String> {
    vec![
        "mmu-rust".to_string(),
        "-f32".to_string(),
        "-aw".to_string(),
        "-oOPFSafx".to_string(),
        "../mmu/lab3_assign/in11".to_string(),
        "../mmu/lab3_assign/rfile".to_string(),
    ]
}

fn main() {
    let default_args = get_default_args();
    let args = std::env::args().collect::<Vec<String>>();
    let actual_args = if args.len() > 1 { &args } else { &default_args };

    // Parse command line arguments
    let (num_frames, algorithm, inputfile, randomfile) = parse_args(actual_args);

    // Call your main function with the parsed arguments
    actual_main_fn(num_frames, &algorithm, &inputfile, &randomfile);
}

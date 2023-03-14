mod process;
use std::env;
use std::path::PathBuf;
use regex::Regex;

fn main() {
    let args: Vec<String> = env::args().collect();

    let matches = clap::App::new("Scheduler algorithms for OS")
        .version("1.0")
        .author("Your Name")
        .arg(clap::Arg::with_name("inputfile")
            .long("inputfile")
            .takes_value(true)
            .default_value("lab2_assign/input1")
            .help("Process array input file"))
        .arg(clap::Arg::with_name("rfile")
            .long("rfile")
            .takes_value(true)
            .default_value("lab2_assign/rfile")
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
    let re = Regex::new(r"^([FLS]|[R|P]\d+(:\d+)?)$").unwrap();
    if !re.is_match(&value) {
        Err(format!("Invalid scheduler specification: {}. Must be one of F, L, S, R<num>, P<num>, or P<num>:<num>.", value))
    } else {
        Ok(())
    }
}

fn main_with_args(args: Vec<String>) {
    // main code here
    let scheduler: Box<dyn Scheduler>;
    match args.get(2).map(|s| &**s) {
        Some("F") => scheduler = Box::new(FCFS::new()),
        Some("L") => scheduler = Box::new(LCFS::new()),
        Some("S") => scheduler = Box::new(SRTF::new()),
        Some(s) if s.starts_with("R") => scheduler = Box::new(RR::new(s[1..].parse::<usize>().unwrap())),
        Some(s) if s.starts_with("P") => {
            let parts = s[1..].split(":").collect::<Vec<&str>>();
            let quantum = parts[0].parse::<usize>().unwrap();
            let maxprio = parts.get(1).map(|&s| s.parse::<usize>().unwrap_or(4)).unwrap_or(4);
            scheduler = Box::new(PRIO::new(quantum, maxprio));
        },
        _ => panic!("Invalid scheduler specification"),
    }

    let rand_generator = RandGenerator::new(args[3].as_str());
    let process_arr = get_process_array(args[1].as_str(), &rand_generator, scheduler.maxprio());

    // let des = DES::new(process_arr);

    simulation(&des, &rand_generator, &process_arr, &*scheduler);
    print_summary(&*scheduler, &process_arr);

}

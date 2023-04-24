use crate::{Process, VMA};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_input_file(filename: &str) -> (Vec<Process>, Vec<(String, usize)>) {
    let file = File::open(filename).expect("Failed to open file");
    let mut reader = BufReader::new(file);

    let mut line = String::new();

    // Skip comment lines
    loop {
        reader.read_line(&mut line).expect("Failed to read line");
        if !line.starts_with('#') {
            break;
        }
        line.clear();
    }
    let num_processes: usize = line.trim().parse().expect("Failed to parse number");
    line.clear();

    // Read processes
    let mut processes = Vec::new();
    for _ in 0..num_processes {
        // Skip comment lines
        loop {
            reader.read_line(&mut line).expect("Failed to read line");
            if !line.starts_with('#') {
                break;
            }
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
        let mut iter = line.split_whitespace();
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

pub fn read_random_file(filename: &str) -> Vec<usize> {
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

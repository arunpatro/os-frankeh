use crate::io_operation;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};

// The input file is structured as follows: Lines starting with ‘#’ are comment lines and should be ignored.
// Any other line describes an IO operation where the 1st integer is the time step at which the IO operation
// is issued and the 2nd integer is the track that is accesses. Since IO operation latencies are largely
// dictated by seek delay (i.e. moving the head to the correct track), we ignore rotational and transfer
// delays for simplicity. The inputs are well formed.
pub fn read_input_file(filename: &str) -> Vec<io_operation> {
    let file = File::open(filename).expect("Failed to open file");
    let mut reader = BufReader::new(file);
    let mut line = String::new();

    let mut io_operations = Vec::new();
    while let Ok(len) = reader.read_line(&mut line) {
        if len == 0 {
            break;
        }
        if line.starts_with('#') {
            line.clear();
            continue;
        }
        let mut iter = line.split_whitespace();
        let time: usize = iter
            .next()
            .expect("Failed to read time")
            .parse()
            .expect("Failed to parse time");
        let track: usize = iter
            .next()
            .expect("Failed to read track")
            .parse()
            .expect("Failed to parse track");
        io_operations.push(io_operation {
            time,
            track,
            start_time: None,
            completed_time: None,
        });
        line.clear();
    }

    io_operations
}

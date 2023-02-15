use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

// create a file in string
const INPUT_FILE: &str = "1 xy 2
2 z xy
5 R 1004  I 5678  E 2000  R 8002  E 7001
0
1 z
6 R 8001  E 1000  E 1000  E 3000  R 1002  A 1010
0
1 z
2 R 5001  E 4000
1 z 2
2 xy z
3 A 8000  E 1001  E 2000
";

// struct Parser {
//     file
//     lineNum: usize,
//     lineOff: usize,
//     finalOffset: usize,
// }

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() {
    println!("Hello, world!");

    

    if let Ok(lines) = read_lines("input-1") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(ip) = line {
                
                




                println!("{}", ip);
            }
        }
    }


}

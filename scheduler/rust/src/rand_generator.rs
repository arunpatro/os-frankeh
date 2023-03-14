use std::fs::File;
use std::io::Read;

pub struct RandGenerator {
    pub filename: String,
    queue: Vec<u32>,
}

impl RandGenerator {
    pub fn new(filename: &str) -> Self {
        let mut rand_generator = RandGenerator {
            filename: filename.to_owned(),
            queue: Vec::new(),
        };
        rand_generator.read_file();
        rand_generator
    }

    pub fn get_rand(&mut self) -> u32 {
        if self.queue.is_empty() {
            self.read_file();
        }
        self.queue.pop().unwrap()
    }

    fn read_file(&mut self) {
        let mut file =
            File::open(&self.filename).expect(&format!("Failed to open file: {}", &self.filename));
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let mut lines = contents.lines();
        let count = lines.next().unwrap().parse().unwrap();

        self.queue = lines
            .take(count)
            .flat_map(|line| line.split_whitespace())
            .map(|s| s.parse().unwrap())
            .collect();
    }
}

use std::path::PathBuf;

use glob::glob;

pub struct CPUState {
    pub path: Vec<PathBuf>,
}

impl CPUState {
    pub fn new() -> Self {
        let mut res = Self { path: vec![] };
        res.read();

        res
    }

    fn read(&mut self) {
        for entry in glob("/sys/devices/system/cpu/cpufreq/policy*").unwrap() {
            match entry {
                Ok(path) => self.path.push(path),
                Err(e) => println!("Error: {:?}", e),
            }
        }
    }
}

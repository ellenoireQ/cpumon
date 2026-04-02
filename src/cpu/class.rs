use std::{fs, path::PathBuf};

use glob::glob;

pub struct Cpu {
    pub id: u32,
    pub path: PathBuf,
    pub scaling_gov: String,
}

pub struct CPUState {
    pub cpu: Vec<Cpu>,
    pub path: Vec<PathBuf>,
}

impl CPUState {
    pub fn new() -> Self {
        let mut res = Self {
            cpu: vec![],
            path: vec![],
        };
        res.read();
        res.read_all_cpus();
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

    fn read_all_cpus(&mut self) {
        for path in &self.path {
            let path_gov = format!("{}/scaling_governor", path.display());
            let cpu_name = path.file_name().unwrap().to_string_lossy();

            let id = cpu_name
                .trim_start_matches("policy")
                .parse::<usize>()
                .unwrap();

            let scaling_gov =
                fs::read_to_string(path_gov).expect("Should have been able to read the file");

            let cpu = Cpu {
                id: id as u32,
                path: path.to_path_buf(),
                scaling_gov: scaling_gov,
            };

            self.cpu.push(cpu);
        }
    }
}

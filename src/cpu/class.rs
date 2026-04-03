use std::{fs, io::Error, path::PathBuf};

use glob::glob;

pub enum Val {
    String(String),
    Int(u32),
}

pub struct Cpu {
    pub id: u32,
    pub path: PathBuf,
    pub scaling_gov: String,
    pub scaling_available_governors: Vec<String>,
    pub scaling_cur_freq: String,
    pub scaling_driver: String,
    pub scaling_governor: String,
    pub scaling_max_freq: String,
    pub scaling_min_freq: String,
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
            let path_available_governors =
                format!("{}/scaling_available_governors", path.display());
            let path_cur_freq = format!("{}/scaling_cur_freq", path.display());
            let path_driver = format!("{}/scaling_driver", path.display());
            let path_governor = format!("{}/scaling_governor", path.display());
            let path_max_freq = format!("{}/scaling_max_freq", path.display());
            let path_min_freq = format!("{}/scaling_min_freq", path.display());
            let cpu_name = path.file_name().unwrap().to_string_lossy();

            let id = cpu_name
                .trim_start_matches("policy")
                .parse::<usize>()
                .unwrap();

            let scaling_available_governors: Vec<String> =
                fs::read_to_string(path_available_governors)
                    .expect("Should have been able to read the file")
                    .trim()
                    .split(' ')
                    .map(|s| s.to_string())
                    .collect();

            let scaling_cur_freq = fs::read_to_string(path_cur_freq)
                .expect("Should have been able to read the file")
                .trim()
                .to_string();
            let scaling_driver = fs::read_to_string(path_driver)
                .expect("Should have been able to read the file")
                .trim()
                .to_string();
            let scaling_governor = fs::read_to_string(path_governor)
                .expect("Should have been able to read the file")
                .trim()
                .to_string();
            let scaling_max_freq = fs::read_to_string(path_max_freq)
                .expect("Should have been able to read the file")
                .trim()
                .to_string();
            let scaling_min_freq = fs::read_to_string(path_min_freq)
                .expect("Should have been able to read the file")
                .trim()
                .to_string();

            let cpu = Cpu {
                id: id as u32,
                path: path.to_path_buf(),
                scaling_gov: scaling_governor.clone(),
                scaling_available_governors,
                scaling_cur_freq,
                scaling_driver,
                scaling_governor,
                scaling_max_freq,
                scaling_min_freq,
            };

            self.cpu.push(cpu);
        }
    }

    pub fn write(&self, path: PathBuf, expect: Val) -> Result<(), Error> {
        match expect {
            Val::String(s) => fs::write(path, s),
            Val::Int(i) => fs::write(path, i.to_string()),
        }
    }
}

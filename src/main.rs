mod cli;
mod cpu;
use clap::Parser;

use crate::{cli::version::version, cpu::class::CPUState};

#[derive(Parser)]
#[command(name = "cpumon", about = "A tool for manages cpu usage in Linux")]
struct Cli {
    #[arg(short, long)]
    read: bool,

    /// Show version
    #[arg(short, long)]
    version: bool,
}

fn main() {
    let args = Cli::parse();

    if args.read {
        let cpu = CPUState::new();
        cpu.cpu.iter().for_each(|f| {
            println!(
                "ID: {}\npath: {}\nscale_gov: {}\nscaling_available_governors: {}\nscaling_cur_freq: {}\nscaling_driver: {}\nscaling_governor: {}\nscaling_max_freq: {}\nscaling_min_freq: {}",
                f.id,
                f.path.display(),
                f.scaling_gov,
                f.scaling_available_governors,
                f.scaling_cur_freq,
                f.scaling_driver,
                f.scaling_governor,
                f.scaling_max_freq,
                f.scaling_min_freq
            )
        });
    }
    if args.version {
        println!("{}", version());
    }
}

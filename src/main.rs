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
        cpu.path.iter().for_each(|f| println!("{:?}", f));
    }
    if args.version {
        println!("{}", version());
    }
}

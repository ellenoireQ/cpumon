mod cli;
mod cpu;
mod tui;
use clap::Parser;

use crate::{cli::version::version, cpu::class::CPUState};

#[derive(Parser)]
#[command(name = "cpumon", about = "A tool for manages cpu usage in Linux")]
struct Cli {
    #[arg(short, long)]
    read: bool,

    /// interactive Terminal UI
    #[arg(short = 't', long)]
    tui: bool,

    /// Show version
    #[arg(short, long)]
    version: bool,
}

fn main() {
    let args = Cli::parse();

    if args.version {
        println!("{}", version());
        return;
    }

    if args.read {
        let cpu = CPUState::new();
        cpu.cpu.iter().for_each(|f| {
            println!(
                "ID: {}\npath: {}\nscale_gov: {}\nscaling_available_governors: {:?}\nscaling_cur_freq: {}\nscaling_driver: {}\nscaling_governor: {}\nscaling_max_freq: {}\nscaling_min_freq: {}",
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
        return;
    }

    if args.tui || !args.read {
        if let Err(e) = tui::run() {
            eprintln!("failed to start TUI: {e}");
            std::process::exit(1);
        }
    }
}

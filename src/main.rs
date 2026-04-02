mod cli;
use clap::Parser;

use crate::cli::version::version;

#[derive(Parser)]
#[command(name = "cpumon", about = "A tool for manages cpu usage in Linux")]
struct Cli {
    /// Show version
    #[arg(short, long)]
    version: bool,
}

fn main() {
    let args = Cli::parse();

    if args.version {
        println!("{}", version());
    }
}

use oberon_compiler::{cli::Cli, driver};
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    if let Err(report) = driver::run(cli) {
        eprintln!("{report}");
        std::process::exit(1);
    }
}
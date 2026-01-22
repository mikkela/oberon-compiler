use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "oberon-compiler", version, about = "An Oberon compiler (scaffolding stage).")]
pub struct Cli {
    /// Input source file (.oberon, .Mod, etc.)
    pub input: PathBuf,

    /// Output file (optional; defaults to <input>.bin)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Print extra info (useful while scaffolding)
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

use std::{env, fs};
use std::path::{Path, PathBuf};
use crate::error::CompilerError;
use std::error::Error;

mod frontend;
mod backend;
mod ir;
mod error;

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");

        let mut current = err.source();
        while let Some(source) = current {
            eprintln!("  caused by: {source}");
            current = source.source();
        }

        std::process::exit(1);
    }
}

fn run() -> Result<(), CompilerError> {
    let input = env::args()
        .nth(1)
        .expect("please provide an input filename");

    let output = env::args()
        .nth(2)
        .expect("please provide an output filename");

    let input_path = PathBuf::from(input);
    let output_path = PathBuf::from(output);

    let source = read_source_file(&input_path)?;
    write_output_file(&output_path, &source)?;

    println!("Copied {} to {}", input_path.display(), output_path.display());

    Ok(())
}

fn read_source_file(path: &Path) -> Result<String, CompilerError> {
    let bytes = fs::read(path).map_err(|source| CompilerError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    String::from_utf8(bytes).map_err(|source| CompilerError::Utf8 {
        path: path.to_path_buf(),
        source,
    })
}

fn write_output_file(path: &Path, text: &str) -> Result<(), CompilerError> {
    fs::write(path, text).map_err(|source| CompilerError::Io {
        path: path.to_path_buf(),
        source,
    })
}
use std::path::{Path, PathBuf};

use crate::{
    cli::Cli,
    diagnostics::{Diagnostic, Severity},
    error::{CompilerError, Report},
    source::SourceFile,
};

pub fn run(cli: Cli) -> Result<(), Report> {
    // 1) Resolve output path
    let output = cli.output.unwrap_or_else(|| default_output_path(&cli.input));

    // 2) Load input file (bytes + line index)
    let source = SourceFile::load(&cli.input).map_err(Report::from)?;

    if cli.verbose {
        eprintln!("Loaded: {} ({} bytes)", source.path.display(), source.text.len());
        eprintln!("Will write output to: {}", output.display());
    }

    // 3) “Pretend compiler”: do nothing useful yet — but demonstrate structured errors.
    // You can remove this once the real pipeline exists.
    if source.text.trim().is_empty() {
        let diag = Diagnostic {
            severity: Severity::Error,
            message: "Input file is empty.".to_string(),
            // Whole-file span (0..0) is fine for empty content
            span: Some(source.span_whole()),
            note: Some("Provide an Oberon module and try again.".to_string()),
        };
        return Err(Report::new(source, vec![diag]));
    }

    // 4) Placeholder output write (creates the file but doesn’t emit code yet)
    std::fs::write(&output, b"")  // empty output for now
        .map_err(|e| Report::from(CompilerError::Io { path: output, source: e }))?;

    Ok(())
}

fn default_output_path(input: &Path) -> PathBuf {
    let mut p = input.to_path_buf();
    // Replace extension, or add .bin if none
    p.set_extension("bin");
    p
}
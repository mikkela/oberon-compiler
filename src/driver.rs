use std::path::{Path, PathBuf};

use crate::{
    cli::Cli,
    diagnostics::{Diagnostic, Severity},
    error::{CompilerError, Report},
    source::SourceFile,
    lexer::{Lexer, TokenKind},
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

    // 3) Early sanity check
    if source.text.trim().is_empty() {
        let diag = Diagnostic {
            severity: Severity::Error,
            message: "Input file is empty.".to_string(),
            span: Some(source.span_whole()),
            note: Some("Provide an Oberon module and try again.".to_string()),
        };
        return Err(Report::new(source, vec![diag]));
    }

    // 4) Lexing phase
    let mut lexer = Lexer::new(&source.text);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token().map_err(|e| {
            // Map LexError → Diagnostic → Report
            let diag = Diagnostic {
                severity: Severity::Error,
                message: e.message,
                span: Some(e.span),
                note: None,
            };
            Report::new(source.clone(), vec![diag])
        })?;

        if cli.verbose {
            eprintln!("TOKEN: {:?}", token);
        }

        if matches!(token.kind, TokenKind::Eof) {
            break;
        }

        tokens.push(token);
    }

    // 5) Placeholder output write (still no codegen)
    std::fs::write(&output, b"")
        .map_err(|e| Report::from(CompilerError::Io { path: output, source: e }))?;

    Ok(())
}

fn default_output_path(input: &Path) -> PathBuf {
    let mut p = input.to_path_buf();
    p.set_extension("bin");
    p
}

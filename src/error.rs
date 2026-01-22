use std::{fmt, io, path::PathBuf};

use thiserror::Error;

use crate::{diagnostics::{Diagnostic, Severity}, source::SourceFile, span::Span};

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("I/O error reading/writing file: {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Source file is not valid UTF-8: {path}")]
    Encoding {
        path: PathBuf,
        #[source]
        source: std::string::FromUtf8Error,
    },
}

/// A pretty-ish report that can show source context + spans.
#[derive(Debug)]
pub struct Report {
    source: Option<SourceFile>,
    diagnostics: Vec<Diagnostic>,
    fatal: Option<CompilerError>,
}

impl Report {
    pub fn new(source: SourceFile, diagnostics: Vec<Diagnostic>) -> Self {
        Self { source: Some(source), diagnostics, fatal: None }
    }

    pub fn from(err: CompilerError) -> Self {
        Self { source: None, diagnostics: vec![], fatal: Some(err) }
    }
}

impl From<CompilerError> for Report {
    fn from(value: CompilerError) -> Self {
        Report::from(value)
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(e) = &self.fatal {
            writeln!(f, "error: {e}")?;
            return Ok(());
        }

        let Some(src) = &self.source else {
            for d in &self.diagnostics {
                writeln!(f, "{}: {}", severity_label(d.severity), d.message)?;
            }
            return Ok(());
        };

        for d in &self.diagnostics {
            writeln!(f, "{}: {}", severity_label(d.severity), d.message)?;
            if let Some(span) = d.span {
                write_span_block(f, src, span)?;
            }
            if let Some(note) = &d.note {
                writeln!(f, "note: {note}")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

fn severity_label(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    }
}

fn write_span_block(f: &mut fmt::Formatter<'_>, src: &SourceFile, span: Span) -> fmt::Result {
    let (line, col) = src.line_col(span.start);
    let line_text = src.line_text(line).unwrap_or("");

    writeln!(f, "  --> {}:{}:{}", src.path.display(), line, col)?;
    writeln!(f, "   |")?;
    writeln!(f, "{:>3} | {}", line, line_text.trim_end_matches('\n'))?;
    writeln!(f, "   | {:>width$}{}", "", "^", width = col.saturating_sub(1))?;
    Ok(())
}

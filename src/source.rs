use std::{path::PathBuf, fs};

use crate::{error::CompilerError, span::Span};

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    /// Start byte offset of each line (0-based). line_starts[0] == 0
    pub line_starts: Vec<usize>,
}

impl SourceFile {
    pub fn load(path: &PathBuf) -> Result<Self, CompilerError> {
        let bytes = fs::read(path).map_err(|e| CompilerError::Io { path: path.clone(), source: e })?;

        // Scaffolding: assume UTF-8 for now; if you later want “bytes-first” semantics,
        // keep bytes and decode more carefully.
        let text = String::from_utf8(bytes)
            .map_err(|e| CompilerError::Encoding { path: path.clone(), source: e })?;

        let line_starts = compute_line_starts(&text);

        Ok(Self {
            path: path.clone(),
            text,
            line_starts,
        })
    }

    pub fn span_whole(&self) -> Span {
        Span::new(0, self.text.len())
    }

    /// Convert byte offset -> (line, column), both 1-based
    pub fn line_col(&self, byte_offset: usize) -> (usize, usize) {
        let line = match self.line_starts.binary_search(&byte_offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = self.line_starts.get(line).copied().unwrap_or(0);
        let col0 = byte_offset.saturating_sub(line_start);
        (line + 1, col0 + 1)
    }

    pub fn line_text(&self, line_1based: usize) -> Option<&str> {
        if line_1based == 0 { return None; }
        let line_idx = line_1based - 1;
        let start = *self.line_starts.get(line_idx)?;
        let end = self.line_starts.get(line_idx + 1).copied().unwrap_or(self.text.len());
        Some(self.text.get(start..end).unwrap_or(""))
    }
}

fn compute_line_starts(s: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (i, b) in s.bytes().enumerate() {
        if b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

# oberon-compiler

A# oberon-compiler

A small Rust project that sets up the **scaffolding** for an Oberon compiler.

This repository deliberately stops *before* introducing language structure
(lexer, parser, AST, etc.).  
The goal is to build the **compiler as a program** first.

---

## Features

- Command line interface (CLI)
- File loading
- Line/column and span tracking
- Diagnostics and error reporting
- A “pretend compiler” that can **fail well**

---

## Quick start

```bash
cargo run -- path/to/Module.Mod --verbose
```
---
## Project layout
```text
src/
  main.rs         # entry point: parse CLI, run driver, print report on error
  lib.rs          # module exports
  cli.rs          # clap-based CLI definition
  driver.rs       # orchestration: resolve paths, load source, write output
  source.rs       # SourceFile: load file, line index, line/col lookup
  span.rs         # Span { start, end } in byte offsets
  diagnostics.rs  # Diagnostic + Severity
  error.rs        # CompilerError + Report (formatted diagnostics)

tests/
  smoke_cli.rs        # minimal “does it run” test
  cli_behavior.rs     # CLI + IO + error behavior tests
```
## Design intent
This project separates **compiler infrastructure** from **language semantics**.
Each module exists to solve a single, early problem.

### `cli.rs` — **User contract**
Defines the public interface of the compiler.
Design goals:
* explicit arguments
* predictable behavior
* minimal surprises

The CLI is treated as a **contract** with the user.

### `source.rs` — **Reality boundary**
Handles everything related to input files:
* file IO
* UTF-8 decoding
* byte offsets
* line start indexing
* line/column lookup

All later compiler stages depend on correct source handling.

### `span.rs` — **Location primitive**
Defines the fundamental unit of location:

`[start, end)`

Spans are expressed in byte offsets and are:
* cheap to copy
* easy to compose
* independent of line/column formatting

### `diagnostics.rs` — **Diagnostic data**
Defines diagnostic data structures:
* severity (`error`, `warning`, `note`)
* message
* optional span
* optional note

No formatting logic lives here.

### `error.rs` — **Reporting and formatting**
Responsible for turning diagnostics into human-readable output:
* formatting error headers
* mapping spans to line/column
* printing source excerpts
* rendering caret indicators

Errors are **designed output**, not panics.

### `driver.rs` — **Orchestration**
Coordinates the compilation process.

At the current scaffolding stage it:
1. resolves input/output paths
2. loads the source file
3. emits early diagnostics
4. writes placeholder output

No language structure is introduced yet.

## Current behavior
At this stage, the compiler:
* reads the input file as UTF-8
* builds a line index for position tracking
* emits a structured diagnostic if the file is empty
* prints diagnostics with source context
* writes an output file (currently empty)

This behavior is intentional.

## Philosophy
> A program that knows how to fail
is already honest.

The first milestone is failing well, not generating code.

## Next steps
The next phase introduces structure:
* recognizing form
* introducing meaning
* teaching the compiler what the language looks like

Those concerns are deliberately absent from this version.
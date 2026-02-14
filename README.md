# oberon-compiler

A small Rust project that incrementally builds an **Oberon compiler** with a strong
focus on *clarity of phases* and *honest machinery*.

The project treats the compiler first as a **well-behaved program**:
one that starts cleanly, reads input correctly, reports errors precisely,
and only then begins to understand language structure.

---

## Features

- Command line interface (CLI)
- File loading
- Line/column and span tracking
- Diagnostics and error reporting
- An explicit **finite-state lexer**
- A compiler that can **fail well**

---

## Quick start

Create a minimal input file (any non-empty text works at this stage):

```bash
mkdir -p examples
printf "MODULE Hello;\nEND Hello.\n" > examples/Hello.Mod
```
Run the compiler:

```bash
cargo run -- path/to/Module.Mod --verbose
```
The program exits on its own and returns you to the shell.
Use Ctrl+C only if you interrupt it during development.
---
## Project layout
```text
src/
  main.rs         # entry point: parse CLI, run driver, print report on error
  lib.rs          # module exports
  cli.rs          # clap-based CLI definition
  driver.rs       # orchestration of the compilation process
  source.rs       # SourceFile: load file, line index, line/col lookup
  span.rs         # Span { start, end } in byte offsets
  diagnostics.rs  # Diagnostic + Severity
  error.rs        # CompilerError + Report (formatted diagnostics)
  lexer.rs        # Finite-state lexer (explicit FSM)

tests/
  smoke_cli.rs        # minimal “does it run” test
  cli_behavior.rs     # CLI + IO + error behavior tests
```
## Design intent
This project separates compiler infrastructure from language semantics.
Each module exists to solve one clear problem, at the lowest appropriate level
of abstraction.

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

### `lexer.rs` — Finite-state lexing

Responsible for recognizing tokens using an explicit finite state machine:
* transitioning between a small, fixed set of states
* reading input character by character
* emitting tokens with precise byte spans
* distinguishing identifiers, numbers, keywords, and simple symbols

The lexer has no stack, no recursion, and no notion of nesting.
Non-regular constructs (such as nested comments) are handled by later phases.

### `driver.rs` — **Orchestration**
Coordinates the compilation process.
At the current stage it:
1. resolves input/output paths
2. loads the source file
3. invokes the lexer
4. emits early diagnostics
5. writes placeholder output

Language structure beyond tokens is introduced later.

## Lexer: an explicit finite-state machine
Lexing is implemented as a **finite state automaton (FSA)**.
The lexer:
* has a finite set of states
* reads input one character at a time
* transitions based only on:
  * the current state
  * the current character 
* has **no stack**, no recursion, no nesting

If a feature cannot be expressed as local state transitions,
it does not belong in the lexer.

### Tokens covered (so far)
* Number(i64) — integer literals
* Identifier(String) — names
* KeywordIf — the keyword IF
* Less and LessEqual — < and <=
* Eof

All tokens carry a Span { start, end } in byte offsets.

### Lexer state graph (conceptual)

                     digit
                +--------------+
                |              v
            +--------+      +--------+
            | Start  | ---> | Number | --(non-digit)--> emit Number, go Start
            +--------+      +--------+
                |
                | letter / '_'
                v
            +--------+
            | Ident  | --(non-ident)--> emit Identifier/Keyword, go Start
            +--------+
                ^
                | letter / digit / '_'
                +----------------------

                '<'
                v
            +--------+
            |  Lt    | -- '='? --> emit <= else emit <, go Start
            +--------+

            whitespace:
              Start --(ws)--> Start  (skip)

### Example: lexer output

The lexer produces a linear stream of tokens, each annotated with a byte span.

For the input:

```oberon
IF a<=10
IF b<2
```
the token stream looks like this:
```text
Number(12) @ [0..2]
Keyword(IF) @ [0..2]
Identifier(a) @ [3..4]
Symbol(<=) @ [4..6]
Number(10) @ [6..8]
Keyword(IF) @ [10..12]
Identifier(b) @ [13..14]
Symbol(<) @ [14..15]
Number(2) @ [15..16]
EOF @ [16..16]
```
This example is maintained as a golden test in
```text
tests/golden/lexer_example.txt.
```
Any intentional change to lexer behavior must update the golden file.

### What the lexer deliberately does not handle
Some Oberon constructs are not regular.

In particular, Oberon comments can nest:
```text
(* outer (* inner *) still outer *)
```
Nested comments require a stack and therefore a context-free recognizer.

For that reason, comments are not handled in the lexer.

That boundary is intentional.

## Current behavior
At this stage, the compiler:
* reads the input file as UTF-8
* builds a line index for position tracking
* lexes input into tokens
* emits a structured diagnostic if the file is empty
* writes an output file (currently empty)

This behavior is intentional.

## Philosophy
> A program that knows how to fail
is already honest.

The first milestone is **failing well**, not generating code.

## Next steps
The next phase introduces structure:
* parsing (context-free recognition)
* stacks and nesting
* meaning beyond tokens

Those concerns are deliberately absent from this version.
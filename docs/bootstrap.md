# Self-Hosting and Bootstrap Plan

Self-compilation is a sequence of working compilers, not a flag that can be switched on at the end. Each stage must be able to build the next stage and must have a reproducible artifact or hash.

## Stage 0: Rust host

The Rust compiler implements lexing, parsing, semantic analysis, and code generation. The compiler library should avoid depending on the command-line layer so it can later be translated into K.

## Stage 1: Useful native compiler

Add a parser, typed abstract syntax tree, name resolution, and a small code
generator. The current implementation reaches the first part of this stage:
`k emit` produces x86-64 System V assembly without a runtime or libc. This is
intentionally suitable as a foundation for a freestanding kernel, although
boot entry points, memory access, interrupts, and a kernel ABI are not yet
implemented. The next backend work must define those interfaces rather than
silently assuming an operating-system process.

## Stage 2: K compiler written in K

Implement a K version of the lexer and parser using only the stable subset already supported by the Rust compiler. Keep the Rust compiler as the trusted reference compiler. Differential tests should compare tokens, diagnostics, and generated output for the same inputs.

The host boundary is now explicit: compiler stages accept source buffers
through `driver::compile_source`, while the command-line `compile` command
only reads UTF-8 source bytes and writes the resulting assembly artifact.

The first K-written component is [`compiler/lexer.k`](../compiler/lexer.k).
It uses caller-owned `char*` input and a `struct Token*` output record, with
no allocation or runtime dependency. The initial version emits one source
unit at a time for whitespace, identifier characters, decimal digits, and
basic punctuation. The Rust test suite compiles it through the normal
pipeline, making it a checked bootstrap artifact while the K implementation
grows toward complete token-span scanning.

## Stage 3: K builds K

Compile the K compiler source with the Rust host compiler, then use that resulting K compiler to compile itself. The two outputs must agree on a defined set of source programs. This is the first self-hosting milestone.

## Stage 4: Reduce the host dependency

Move target-specific code into a small, documented backend boundary. Keep a Rust bootstrap compiler for recovery, but make normal K development use the self-hosted compiler. Reproducible build scripts should record compiler version, target, and input hashes.

## Invariants for every stage

- The language specification and implementation tests change together.
- A bootstrap compiler never silently accepts syntax that the next compiler cannot read.
- Compiler output has a deterministic mode for comparison.
- Error messages remain source-spanned and actionable.
- The bootstrap process is documented as commands a new contributor can run.

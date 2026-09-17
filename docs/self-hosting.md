# K Self-Hosting Roadmap

Self-hosting is a sequence of independently verifiable compiler stages. The
Rust compiler remains the recovery and reference implementation until the K
compiler has produced two reproducible releases.

## Stage 1: Reference frontend

The Rust host must remain the executable specification for the supported K
grammar:

- lexer tokens and source spans
- recursive-descent parsing
- semantic diagnostics
- deterministic x86-64 assembly

Every new K bootstrap source is listed in `compiler/sources.txt` and is
compiled and checked by the Rust host.

## Stage 2: K frontend

The K implementation mirrors the reference frontend using only the stable
language subset:

1. `compiler/lexer.k` provides allocation-free tokenization.
2. `compiler/parser.k` owns lexer state and parses structs, function
   signatures, blocks, declarations, control flow, and expressions.
3. A K semantic checker adds scopes, symbol tables, type compatibility, and
   source-spanned diagnostics.

The completion gate for this stage is differential coverage: the Rust and K
frontends must accept and reject the same conformance fixtures and report the
same token boundaries.

## Stage 3: K backend

Add the remaining compiler pipeline in small deterministic boundaries:

1. typed AST to K IR lowering
2. IR validation
3. x86-64 System V code generation
4. assembly and object-file driver

The K backend must reproduce the Rust backend for the conformance fixtures.
Target-specific operations such as the Linux `print` intrinsic stay behind a
documented backend boundary.

## Stage 4: First self-hosted compiler

The Rust host compiles the complete K compiler source set into an executable.
That executable then compiles the same source set. This is the first
self-hosting milestone. The two builds must agree on:

- source manifest and compiler version
- diagnostics for invalid fixtures
- generated assembly for valid fixtures
- executable behavior of smoke tests

## Stage 5: Reproducible bootstrap

Build the compiler twice from the same tagged source release and compare
artifacts byte-for-byte, or compare a documented normalized representation
when timestamps or object metadata cannot be removed. Repeat this process for
two tagged releases.

Only after both releases pass the bootstrap and conformance suites may the
Rust implementation be removed from the normal build.

## Working commands

From the repository root:

```text
cargo test --quiet
cargo run --quiet -- check compiler\lexer.k
cargo run --quiet -- check compiler\parser.k
scripts\check-bootstrap.ps1
```

The Linux/WSL bootstrap script additionally assembles, links, and runs the
freestanding hello-world program.

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
2. `compiler/parser.k` owns lexer state, exposes source slices, and parses
   structs, function signatures, blocks, declarations, control flow, and
   expression-shaped token sequences into caller-owned AST records. Its
   expression arena provides nested unary, binary, call, index, and field
   nodes without an allocator. The statement arena stores function bodies,
   control-flow ranges, assignments, returns, and expression roots in the same
   caller-owned storage model.
3. A K semantic checker adds scopes, symbol tables, type compatibility, and
   source-spanned diagnostics.

The first semantic slice is implemented in `compiler/parser.k`: it validates
AST storage, records parameters and `let` declarations in caller-owned symbol
storage, resolves name expressions, and traverses nested statement ranges.
It now carries primitive/pointer type kinds, infers expression types, checks
returns, assignments, arithmetic, indexing, and boolean conditions, and
collects caller-owned function and struct field signatures. Calls validate
known callees, arity, and every argument type through a caller-owned linked
argument-list arena.

The first backend slice is now present as caller-owned typed IR lowering.
`IrStorage` records typed instructions, function ranges, and basic-block
ranges. Nested blocks, `if`, and `while` lower to explicit branch and jump
records. `ir_validate` now checks those ranges and control-flow references
before target-specific assembly consumes the IR; target-specific assembly
remains a separate stage.

The first K-written backend slice is in `compiler/backend.k`. It owns a
caller-provided character buffer and emits deterministic x86-64 assembly
headers, function labels, prologues, and a return-zero epilogue. Instruction
selection now dispatches unary negate, arithmetic, and return IR records;
integer constants, integer formatting, and typed local stack-slot load/store
emitters are now available. Calls, pointer memory, and control-flow label
emission remain the next backend work.

The completion gate for this stage is differential coverage: the Rust and K
frontends must accept and reject the same conformance fixtures and report the
same token boundaries.

The machine-model prerequisite is in progress: fixed-width primitive names
are recognized through the Rust frontend and typed IR layout. Exact-width
operations, casts, overflow rules, and backend lowering must be specified
before those types are used as a stable contract by the K-written compiler.

## Stage 3: K backend

Add the remaining compiler pipeline in small deterministic boundaries:

1. typed AST to K IR lowering, including basic blocks and control flow
2. IR validation
3. x86-64 System V code generation
4. assembly and object-file driver

The K backend must reproduce the Rust backend for the conformance fixtures.
Target-specific operations such as the Linux `print` intrinsic stay behind a
documented backend boundary.

The first complete-pipeline arithmetic regression is now covered by the Rust
driver: `int main() { let answer = 20 + 22; return answer; }` is compiled for
the KrumpyOS x86-64 target and checked for constant emission, local storage,
local reload, and return-value handling. This is the baseline for equivalent
K-written backend output before adding a runtime/QEMU execution harness.

Target selection is part of the compiler contract. The first self-hosted
release must reproduce the x86-64 KrumpyOS target before the AArch64 backend is
added; unsupported targets must fail explicitly rather than silently emitting
the wrong machine code.

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

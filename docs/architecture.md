# Compiler Architecture

K is organized as a sequence of explicit transformations. Each stage owns one kind of knowledge and communicates through data structures that can be tested independently.

```text
source text
    -> lexer       (tokens with byte spans)
    -> parser      (syntax tree)
    -> resolver    (names and scopes)
    -> type checker (types and diagnostics)
    -> lowerer     (typed intermediate representation)
    -> backend     (target machine code)
```

## Host boundary

The command-line binary in `src/main.rs` handles files, arguments, and human-readable output. The compiler library in `src/lib.rs` owns language behavior. Keeping those responsibilities separate is important for self-hosting: the K implementation can replace the host shell while reusing the same pipeline concepts.

## Planned modules

- `lexer`: converts source bytes into tokens and reports spans.
- `parser`: validates grammar and creates the source-spanned untyped syntax tree in `src/parser.rs`.
- `sema`: resolves declarations, scopes, types, and invalid operations.
- `ir`: lowers checked programs into typed locals, basic blocks, explicit
  constants, loads/stores, arithmetic, calls, and control-flow instructions;
  it also performs constant folding, unreachable-block pruning, and
  deterministic ordered struct layouts with explicit field offsets.
- `codegen`: emits deterministic x86-64 System V assembly for the first
  supported target. It has no runtime or libc dependency, which is the first
  step toward a freestanding kernel toolchain.
- String literals are emitted into `.rodata`; indexed pointer arithmetic uses
  the pointee size recorded by the typed IR; struct field access lowers to
  address adjustment followed by the normal load/store instructions.
- `driver`: coordinates stages and diagnostics without embedding policy in them.
- `driver::compile_source`: the filesystem-independent source-buffer API;
  filesystem reads and writes remain in the command-line binary.

## Design constraints

The core should remain deterministic, testable without a filesystem, and suitable for a future freestanding build. Allocation may be used in the Rust bootstrap implementation, but compiler stages should not depend on hidden global state or host-specific behavior.

Every stage should preserve enough source span information for diagnostics. Intermediate representations should be serializable or printable so the Rust compiler and future K compiler can be compared during bootstrap.

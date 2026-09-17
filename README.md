# K

K is a small, low-level programming language inspired by C and implemented in Rust. The project is intended to become self-compiling: eventually, the K compiler will be rebuilt in K and compiled by an earlier K compiler.

This repository is at **stage 2 bootstrap**. The Rust implementation can lex, parse, check,
and emit x86-64 assembly for a small, intentionally conservative subset of K.
The first fixed-width primitive type names (`u8`, `u16`, `u32`, `u64`, `i32`,
`i64`, and `bool`) are now recognized and carried through semantic analysis
and IR layout. Their final arithmetic and assembly-width behavior is still
being implemented.
The backend is freestanding-friendly assembly: it does not link a runtime or
depend on libc, but it currently targets the System V calling convention.

## Try it

```text
cargo test
cargo run -- lex examples/hello.k
cargo run -- parse examples/hello.k
cargo run -- check examples/hello.k
cargo run -- emit examples/hello.k
cargo run -- compile examples/hello.k target/hello.s
cargo run -- compile examples/hello.k target/hello.s x86_64-krumpyos
```

The command-line tool currently exposes one diagnostic command:

```text
k lex <file.k>
k parse <file.k>
k check <file.k>
k emit <file.k>
k compile <input.k> <output.s>
```

Target names are explicit at the compile boundary. The implemented targets
currently share the x86-64 backend:

- `x86_64-unknown-linux-gnu` (default)
- `x86_64-krumpyos`

`aarch64-krumpyos` is reserved for the upcoming ARM64 backend and fails
explicitly until that backend and its ABI are implemented.

## Design direction

- Explicit integer and pointer types; no hidden allocations.
- Predictable data layout and calling conventions.
- Fixed-width primitive type names are being introduced before `int` and
  `char` are stabilized.
- A small grammar that is easy to implement without a large runtime.
- Diagnostics with byte spans from the beginning.
- A freestanding-friendly compiler core, separated from the host command line.
- Bootstrapping in small, auditable stages rather than a one-shot rewrite.

The syntax and guarantees are provisional until marked stable. See [the language notes](docs/language.md) and [the bootstrap plan](docs/bootstrap.md).

Rust is currently a temporary bootstrap and recovery compiler. The project
will remove it only after two reproducible releases have been built by K.

## Repository map

- `src/lexer.rs`: source spans, tokens, and the first executable language rules.
- `src/parser.rs`: recursive-descent parser and source-spanned AST.
- `src/sema.rs`: name resolution and initial type checking.
- `src/codegen.rs`: initial x86-64 System V assembly backend.
- The current bootstrap subset includes mutable assignment and character literals,
  which are prerequisites for representing compiler state.
- Pointers, dereference, and indexing are available as explicit zero-cost
  operations, and code generation now passes through a typed IR boundary.
- The IR records typed locals and basic blocks with explicit constants,
  loads/stores, arithmetic, calls, branches, jumps, and returns. The native
  backend consumes this IR rather than walking the parser AST.
- IR lowering now folds constant instruction sequences and removes unreachable
  basic blocks before assembly emission.
- K now supports `void`, string literals as zero-terminated `char*` data, and
  pointee-aware pointer indexing.
- `src/driver.rs` provides a filesystem-independent source-to-assembly API;
  `k compile` is the host file-to-artifact interface.
- `src/lib.rs`: compiler-library boundary and future pipeline stages.
- `src/main.rs`: host-side command-line interface.
- `compiler/lexer.k`: first K-written compiler component, with complete
  identifier and decimal-integer spans in a caller-owned-buffer scanner.
- `compiler/hello.k`: minimal K program using the freestanding `print`
  intrinsic.
- `editors/vscode/`: TextMate syntax grammar and VS Code language metadata.
- `.gitattributes`: GitHub Linguist language classification for `.k` files.
- `docs/language.md`: current language contract and open decisions.
- `docs/bootstrap.md`: route from Rust implementation to self-hosting.
- `scripts/`: reproducible bootstrap and source-manifest checks.
- `examples/hello.k`: a tiny source fixture used by the documentation.

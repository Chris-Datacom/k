# K

K is a small, low-level programming language inspired by C and implemented in Rust. The project is intended to become self-compiling: eventually, the K compiler will be rebuilt in K and compiled by an earlier K compiler.

This repository is at **stage 0**. The Rust implementation can lex a small, intentionally conservative subset of K source and print its tokens.

## Try it

```text
cargo test
cargo run -- lex examples/hello.k
```

The command-line tool currently exposes one diagnostic command:

```text
k lex <file.k>
```

## Design direction

- Explicit integer and pointer types; no hidden allocations.
- Predictable data layout and calling conventions.
- A small grammar that is easy to implement without a large runtime.
- Diagnostics with byte spans from the beginning.
- A freestanding-friendly compiler core, separated from the host command line.
- Bootstrapping in small, auditable stages rather than a one-shot rewrite.

The syntax and guarantees are provisional until marked stable. See [the language notes](docs/language.md) and [the bootstrap plan](docs/bootstrap.md).

## Repository map

- `src/lexer.rs`: source spans, tokens, and the first executable language rules.
- `src/lib.rs`: compiler-library boundary and future pipeline stages.
- `src/main.rs`: host-side command-line interface.
- `docs/language.md`: current language contract and open decisions.
- `docs/bootstrap.md`: route from Rust implementation to self-hosting.
- `examples/hello.k`: a tiny source fixture used by the documentation.

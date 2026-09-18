# Contributing to K

K is an experimental language project. Early contributions should favor small, reviewable steps that make the language easier to understand and bootstrap.

## Before changing code

1. Read [the language notes](docs/language.md), [the architecture](docs/architecture.md),
   and [the self-hosting roadmap](docs/self-hosting.md).
2. Check whether the behavior is marked prototype, specified, or stable.
3. Update the relevant Markdown specification when a language rule changes.

## Development

Install a current stable Rust toolchain, then run:

```text
cargo test
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

The CLI prototype can inspect a source file with:

```text
cargo run -- lex examples/hello.k
```

## Pull requests

Keep each change focused. Include tests for compiler behavior and explain any change to syntax, diagnostics, memory rules, or generated output. Do not commit `target/` or editor-specific files.

A language feature is not complete until its implementation, tests, and documentation agree. When a design is still unsettled, label it as provisional instead of quietly making it a promise.

Changes affecting the KrumpyOS kernel target or planned user-space target must
state their ABI, privilege, bootstrap, and package-build impact. Keep OS policy
in the KrumpyOS repository; keep language and compiler contracts here.

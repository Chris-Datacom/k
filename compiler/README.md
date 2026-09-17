# K-written compiler

This directory contains the compiler implementation that will eventually
replace the Rust host.

## Bootstrap policy

Rust remains the trusted recovery compiler until K has produced two
reproducible self-hosted releases. It is not the long-term implementation.
The Rust compiler must continue to compile every file in this directory so a
broken K bootstrap can be recovered without circular dependencies.

## Current contents

- `lexer.k`: allocation-free source scanning into caller-owned `Token` records,
  including complete identifiers, decimal literals, keyword classification,
  whitespace skipping, and `//` comment skipping. It also exposes the
  stateful `Lexer`/`next_token` token-stream interface.
- `parser.k`: an allocation-free recursive-descent parser built on the same
  token-stream model. It recognizes structs, function signatures, blocks,
  declarations, control flow, and expression-shaped token sequences. It also
  exposes source slices and caller-owned program/function AST records, plus a
  fixed-capacity expression arena for nested expression nodes and a
  fixed-capacity statement arena for function bodies.
- `parser.k` also includes the first semantic boundary: caller-owned symbols,
  duplicate declaration checks, name resolution, expression-tree validation,
  statement traversal, inferred primitive/pointer types, return checking,
  assignment compatibility, condition validation, and caller-owned function
  and struct signature tables for calls and field access. Calls retain a
  caller-owned linked argument arena and validate every argument's type.
  The initial typed IR boundary adds caller-owned instruction and function
  ranges, lowering constants, names, unary/binary expressions, calls, lets,
  assignments, returns, and expression statements with explicit capacity
  errors. Basic-block storage now lowers nested blocks, `if`, and `while`
  into explicit branch and jump records.
- `hello.k`: executable smoke test using the freestanding Linux `print`
  intrinsic.
- `sources.txt`: deterministic list of K compiler sources checked by the
  bootstrap scripts. It includes the lexer, parser, and hello-world smoke
  test.

From the repository root, run `scripts/bootstrap.sh` on Linux or
`scripts/bootstrap.ps1` on Windows with WSL. `scripts/check-bootstrap.sh`
checks every source in the manifest and runs the Rust reference tests.
Windows users can run the equivalent `scripts/check-bootstrap.ps1`.

## Required path to self-hosting

The K compiler must grow in this order:

1. Complete the K lexer and parser against the Rust conformance fixtures.
2. Add a K semantic checker and symbol tables.
3. Add K typed IR lowering and the x86-64 backend.
4. Add a K compiler driver that can compile the compiler sources.
5. Produce two reproducible K-built releases matching the Rust reference.

See [`docs/self-hosting.md`](../docs/self-hosting.md) for the gates and
commands for each stage. Only after the final reproducibility gate may the
Rust implementation be removed.

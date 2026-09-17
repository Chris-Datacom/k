# K-written compiler

This directory contains the compiler implementation that will eventually
replace the Rust host.

## Bootstrap policy

Rust remains the trusted recovery compiler until K has produced two
reproducible self-hosted releases. It is not the long-term implementation.
The Rust compiler must continue to compile every file in this directory so a
broken K bootstrap can be recovered without circular dependencies.

## Current contents

- `lexer.k`: allocation-free source scanning into caller-owned `Token` records.
- `hello.k`: executable smoke test using the freestanding Linux `print`
  intrinsic.
- `sources.txt`: deterministic list of K compiler sources checked by the
  bootstrap scripts.

From the repository root, run `scripts/bootstrap.sh` on Linux or
`scripts/bootstrap.ps1` on Windows with WSL. `scripts/check-bootstrap.sh`
checks every source in the manifest and runs the Rust reference tests.
Windows users can run the equivalent `scripts/check-bootstrap.ps1`.

## Required path to self-hosting

The K compiler must grow in this order:

1. Complete lexer token kinds and comments.
2. K parser for the supported language subset.
3. K semantic checker and symbol tables.
4. K typed IR lowering and x86-64 backend.
5. A K compiler driver that can compile the compiler sources.
6. Two reproducible K-built releases matching the Rust reference output.

Only after step 6 may the Rust implementation be removed.

# Changelog

All notable changes to K will be recorded here. The project is pre-1.0, so syntax and compiler behavior may change between releases.

## Unreleased

- Added the Rust compiler workspace.
- Added a source-spanned lexer and token dump command.
- Added parsing, semantic checking, and an initial freestanding-friendly
  x86-64 assembly emitter.
- Added pointer types, address/dereference operations, indexing, memory
  assignment, and the first typed IR boundary before code generation.
- Added a stack-based typed IR with explicit constants, loads/stores,
  arithmetic, calls, branches, jumps, and returns; native emission now
  consumes IR instead of walking the parser AST.
- Added target-independent IR optimization for constant instruction folding
  and unreachable basic-block pruning.
- Added language, architecture, contribution, and self-hosting documentation.

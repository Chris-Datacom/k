# Changelog

All notable changes to K will be recorded here. The project is pre-1.0, so syntax and compiler behavior may change between releases.

## 0.1.0 - 2026-09-17

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
- Added `void` returns, zero-terminated string literals, static string data,
  and pointee-size-aware pointer indexing.
- Added named structs, deterministic field layouts, and field load/store
  lowering.
- Added the first K-written lexer component with caller-owned `Token` records.
- Added a Rust pipeline test that compiles `compiler/lexer.k`.
- Added language, architecture, contribution, and self-hosting documentation.

## Unreleased

The next milestone will expand the K-written lexer from single source-unit
classification to keyword and comment recognition, then compare its output
against the Rust reference lexer.

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
- Added the freestanding Linux `print(char*)` intrinsic and
  `compiler/hello.k` smoke test.
- Added a Linux `_start` wrapper so generated `main` programs exit cleanly.
- Added GitHub Linguist metadata and a VS Code/TextMate syntax grammar for
  `.k` files.
- Added language, architecture, contribution, and self-hosting documentation.

## Unreleased

- Added prototype fixed-width primitive type names: `u8`, `u16`, `u32`, `u64`,
  `i32`, `i64`, and `bool`.
- Carried fixed-width types through parsing, semantic analysis, and IR layout
  sizing, with tests for primitive struct field sizes.
- Added the first kernel-facing intrinsics, `outb(u16, u8) -> void` and
  `inb(u16) -> u8`, lowering to the x86-64 `out`/`in` port instructions.
  They are only accepted on the freestanding `x86_64-krumpyos` target and
  are rejected at code generation on hosted targets. Added
  `examples/serial_port.k` as a smoke test.
- Added `cli()`, `sti()`, `hlt()`, and `pause()` intrinsics for the
  `x86_64-krumpyos` target, lowering directly to the matching x86-64
  instructions and rejecting hosted targets at code generation.
- Added explicit `(type)expression` casts between pointers and integers of
  any width, restricted to K's scalar machine-model types (`bool`, `void`,
  and `struct` are rejected as cast sources or targets). Widening and
  pointer-reinterpretation casts are free; narrowing casts emit a single
  truncating instruction. This is the mechanism for naming fixed hardware
  addresses (MMIO, the VGA text buffer) as pointers, and
  `examples/serial_port.k` now demonstrates it.
- Added a `volatile` pointer qualifier (`volatile T*`). Loads and stores
  performed through a volatile pointer (`*ptr`, `*ptr = value`, `ptr[i]`,
  `ptr[i] = value`) are lowered to IR instructions explicitly tagged
  `volatile`, which the backend never merges, reorders, or elides, and
  which any future optimization pass is required to honor the same way.
  `volatile` and non-`volatile` pointers to the same pointee are distinct
  types; converting between them requires an explicit cast, matching K's
  exact-type-match rules. `examples/serial_port.k`'s VGA buffer now uses
  `volatile u16*`.
- The next language milestone is exact-width arithmetic, conversions,
  overflow rules, and backend load/store lowering.

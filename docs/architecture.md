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

## Target boundary

Target selection is explicit at the driver boundary. `x86_64-unknown-linux-gnu`
and `x86_64-krumpyos` currently use the implemented x86-64 backend. The
`aarch64-krumpyos` target is reserved but rejected before code generation until
its calling convention, object format, and backend are implemented.

`x86_64-krumpyos` is the kernel-facing target: it does not emit the Linux
`_start`/`exit` trampoline, and it is the only target that accepts the
`outb`/`inb` I/O-port intrinsics plus the `cli`/`sti`/`hlt`/`pause` CPU
control intrinsics. `x86_64-unknown-linux-gnu` remains a hosted target used
for the `print` intrinsic and for running compiled programs without a kernel
or emulator during development.

The first kernel image layout is defined by
[`linker/x86_64-krumpyos.ld`](../linker/x86_64-krumpyos.ld). It produces an
`elf64-x86-64` image beginning at physical address `0x00010000` and requires a
boot stub to provide `_start`. The script places `.text`, `.rodata`, `.data`,
and `.bss` in that order and exports section boundary symbols
(`__text_start`/`__text_end`, `__rodata_start`/`__rodata_end`,
`__data_start`/`__data_end`, and `__bss_start`/`__bss_end`) plus
`__kernel_start`/`__kernel_end`. The linker script does not select a boot
protocol or initialize the CPU; those responsibilities remain with the
boot-stub milestone.

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
- Fixed-width loads and stores use the IR type to select byte, word, dword,
  or qword operations; narrow values are extended into the virtual stack
  representation before arithmetic or calls.
- String literals are emitted into `.rodata`; indexed pointer arithmetic uses
  the pointee size recorded by the typed IR; struct field access lowers to
  address adjustment followed by the normal load/store instructions.
- Explicit `(type)expression` casts between pointers and integers lower to a
  single `Cast` IR instruction. Because every value already lives in a full
  64-bit stack slot with the correct sign/zero extension applied at load
  time, widening and pointer-to-pointer casts require no instructions;
  narrowing casts emit one register-width truncation (`movzx`/`mov`/`movsxd`)
  so the stored bit pattern matches the target type.
- Pointer types may carry a `volatile` qualifier, tracked as part of the
  pointer's type in the parser, sema, and IR layers (`Type::Pointer`,
  `ValueType::Pointer`, and `IrType::Pointer` each carry a `volatile: bool`).
  Loads and stores lowered through a volatile pointer are tagged
  `volatile` on the IR instruction itself (`Load`, `Store`, and the
  `Unary`/`Dereference` read), independent of any type inference elsewhere
  in the pipeline. **This is a hard contract for every future optimization
  pass**: constant folding, common subexpression elimination, dead-store
  elimination, and any instruction scheduling must treat `volatile`
  instructions as an ordering barrier and must never merge, reorder, hoist,
  sink, or eliminate them. The current backend performs none of those
  optimizations, so today the tag mainly documents an assembly comment
  (`; volatile load` / `; volatile store`); its purpose is to make later
  optimization work MMIO-safe by construction instead of by review.
- `driver`: coordinates stages and diagnostics without embedding policy in them.
- `driver::compile_source`: the filesystem-independent source-buffer API;
  filesystem reads and writes remain in the command-line binary.

## Design constraints

The core should remain deterministic, testable without a filesystem, and suitable for a future freestanding build. Allocation may be used in the Rust bootstrap implementation, but compiler stages should not depend on hidden global state or host-specific behavior. Fixed-width primitive names are now part of the prototype type pipeline; their exact instruction selection remains a backend contract to be completed.

Every stage should preserve enough source span information for diagnostics. Intermediate representations should be serializable or printable so the Rust compiler and future K compiler can be compared during bootstrap.

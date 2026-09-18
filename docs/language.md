# K Language Notes

This document is the working language contract. It is deliberately more precise than a collection of examples: compiler behavior should be derived from rules here, and changes should update this document with the implementation.

## Principles

K is intended for systems work where representation and cost matter. The language should make the machine model visible without requiring every program to be written in assembly.

1. Values have a known size and alignment.
2. Memory access is explicit.
3. Integer overflow behavior is specified, not accidental.
4. The compiler does not require a garbage collector or managed runtime.
5. The core language remains small enough to bootstrap.

## Current lexical contract

Source is UTF-8, but the initial grammar is ASCII. Whitespace is insignificant. A line comment begins with `//` and continues to the end of the line.

Identifiers begin with `a-z`, `A-Z`, or `_`, followed by those characters, digits, or `_`. Decimal integer literals contain one or more digits. Character literals use single quotes and support `\\n`, `\\r`, `\\t`, `\\\\`, and `\\'`; strings use double quotes and support the same escapes plus `\\\"`. The first reserved words are `int`, `char`, `if`, `else`, `while`, `return`, `let`, `true`, and `false`; `void` is recognized in type position, and `volatile` is recognized as a pointer qualifier in type position.

The operator and punctuation set includes `+ - * / = == != < <= > >= & | ^ ~ << >> ; , ( ) { } [ ] .`.

The freestanding Linux bootstrap provides a `print(char*)` intrinsic. It
writes the complete zero-terminated string's payload to standard output using
the x86-64 `write` system call; it does not allocate or require libc. This
intrinsic is currently target-specific and exists to make the self-hosting
bootstrap observable.

The freestanding `x86_64-krumpyos` target provides low-level kernel intrinsics:
- Port I/O: `outb(u16, u8) -> void`, `inb(u16) -> u8`
- Interrupts & CPU control: `cli() -> void`, `sti() -> void`, `hlt() -> void`, `pause() -> void`
- Control registers: `read_cr0() -> u64`, `write_cr0(u64) -> void`, `read_cr2() -> u64`, `read_cr3() -> u64`, `write_cr3(u64) -> void`, `read_cr4() -> u64`, `write_cr4(u64) -> void`
- Descriptor & TLB management: `lidt(void*) -> void`, `sidt(void*) -> void`, `invlpg(void*) -> void`
- Model-specific registers: `rdmsr(u32) -> u64`, `wrmsr(u32, u64) -> void`

They are rejected at code generation on hosted targets such as
`x86_64-unknown-linux-gnu`, where userspace lacks the privilege level to
execute privileged machine instructions.

## Parsed core syntax

```k
int main() {
    let answer = 42;
    return answer;
}
```

The parser currently accepts function definitions with `void`, `int`, `char`,
the fixed-width names `u8`, `u16`, `u32`, `u64`, `i32`, `i64`, and `bool`, or
pointer return types, optionally `volatile`-qualified. It also accepts
pointer parameters, braced blocks, `let` declarations, assignments to
variables or memory locations, `return`, `if`/`else`, `while`, expression
statements, calls, character and string literals, unary `-`, `&`, and `*`,
indexing, C-style `(type)expression` casts, and binary
arithmetic/comparison operators. String literals lower to static
zero-terminated bytes and have type `char*`. Pointer indexing scales by the
pointee size; bounds checks are intentionally absent.

A cast is written `(type)expression`, for example `(u32*)address` or
`(u64)pointer`. Casts are restricted to K's machine-model scalars: any
pointer or fixed-width/`int` integer type may be reinterpreted as another
pointer or integer type. `bool`, `void`, and `struct` values are rejected as
cast sources or targets with a diagnostic; they must go through an explicit
comparison, call, or field access instead. Widening a value that was already
loaded with the correct sign/zero extension is free; narrowing a value emits
a single truncating instruction so the stored bit pattern matches the target
width. Casts have no other runtime behavior: there is no bounds checking,
alignment checking, or provenance tracking. This is the mechanism freestanding
code uses to name fixed hardware addresses (MMIO registers, the VGA text
buffer, page tables) as pointers; see `examples/serial_port.k`.

A pointer type may be qualified `volatile`, written before the pointee type,
for example `volatile u16*` or `(volatile u32*)address`. The qualifier marks
memory reached through that pointer as having effects visible outside the
program (hardware registers, memory-mapped I/O) that the compiler must not
optimize away: every `*pointer`, `*pointer = value`, `pointer[i]`, and
`pointer[i] = value` performed through a `volatile`-qualified pointer lowers
to an IR instruction explicitly tagged `volatile`, which the backend never
merges, reorders, or elides, and which any future optimization pass must
honor the same way. `volatile` is only meaningful on pointer types; using it
on a non-pointer type, or with no following `*`, is a parse error. A
`volatile` pointer and a plain pointer to the same pointee are distinct
types under K's exact-type-match rules: converting between them (adding or
discarding the qualifier) requires an explicit cast, the same as any other
pointer reinterpretation. See `examples/serial_port.k` for a MMIO example
using a `volatile u16*` into the VGA text buffer.

The grammar remains provisional. Before syntax is stabilized, the compiler must answer: declaration forms, function types, arrays, module boundaries, and whether `let` permits inference everywhere.

## Structs

Struct declarations use explicit fields and C-like field access:

```k
struct Token {
    int kind;
    int start;
}
```

Named struct types may be used in function signatures, and fields are selected
with `value.field`. A pointer to a struct may also use the same selector; the
compiler treats it as implicit dereference for this prototype. Struct layout is
ordered and has no hidden allocation or runtime metadata. Field offsets and
complete aggregate lowering are still being completed before this syntax is
used by the self-hosted compiler. Local struct storage is supported with a
declaration such as `struct Token token;`; the compiler reserves the complete
layout size in the function frame, and the declaration is initially
uninitialized.

## Extern declarations

External functions defined outside the compilation unit (such as in assembly or foreign object files) are declared with `extern`:

```k
extern void install_idt();
extern int add(int a, int b);
```

The compiler checks call arity and argument/return types against the declared signature, and emits standard ABI calls to the external symbol.

## Primitive type status

| Type | Intent |
| --- | --- |
| `u8` | 8-bit unsigned integer and byte |
| `u16` | 16-bit unsigned integer |
| `u32` | 32-bit unsigned integer |
| `u64` | 64-bit unsigned integer |
| `i32` | 32-bit signed integer |
| `i64` | 64-bit signed integer |
| `bool` | `true` or `false` |
| `void` | no value |

The fixed-width names are accepted by the lexer, parser, semantic checker, IR,
and x86-64 backend memory operations. Loads and stores use the declared width
and extend values into the compiler's virtual 64-bit stack representation.
Exact-width arithmetic, overflow behavior, alignment, and ABI-width argument
rules remain target work and must be specified before these types are stable.

`int` and `char` remain prototype aliases while their final meaning is decided.
They must not be treated as portable fixed-width types.

## Safety boundary

K will not pretend to be memory-safe by accident. The compiler should distinguish ordinary operations from explicitly unsafe operations, document aliasing and lifetime assumptions, and make undefined behavior cases visible in the specification. This boundary is an open design task, not a promise that the prototype already enforces it.

## Stability labels

- **Prototype**: implemented enough to experiment with, likely to change.
- **Specified**: behavior is documented and covered by tests.
- **Stable**: compatibility is promised for the language version.

The lexer is prototype-level today. Every new stage should move only the relevant behavior toward specified, with executable tests beside the implementation.

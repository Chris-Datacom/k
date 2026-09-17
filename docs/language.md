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

Identifiers begin with `a-z`, `A-Z`, or `_`, followed by those characters, digits, or `_`. Decimal integer literals contain one or more digits. Character literals use single quotes and support `\\n`, `\\r`, `\\t`, `\\\\`, and `\\'`. The first reserved words are `int`, `char`, `if`, `else`, `while`, `return`, `let`, `true`, and `false`.

The initial operator and punctuation set is `+ - * / = == != < <= > >= & ; , ( ) { } [ ]`.

## Parsed core syntax

```k
int main() {
    let answer = 42;
    return answer;
}
```

The parser currently accepts function definitions with `int`, `char`, or pointer return types, pointer parameters, braced blocks, `let` declarations, assignments to variables or memory locations, `return`, `if`/`else`, `while`, expression statements, calls, character literals, unary `-`, `&`, and `*`, indexing, and binary arithmetic/comparison operators. Pointer and index operations use machine-word loads and stores in the initial backend; bounds checks are intentionally absent.

The grammar remains provisional. Before syntax is stabilized, the compiler must answer: declaration forms, function types, arrays, pointer spelling, casts, modules, and whether `let` permits inference everywhere.

## Planned primitive types

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

`int` and `char` are currently lexer keywords for exploring C-like syntax. Their final meaning must be fixed before the type checker is declared stable; machine-dependent aliases are a poor foundation for portable K programs.

## Safety boundary

K will not pretend to be memory-safe by accident. The compiler should distinguish ordinary operations from explicitly unsafe operations, document aliasing and lifetime assumptions, and make undefined behavior cases visible in the specification. This boundary is an open design task, not a promise that the prototype already enforces it.

## Stability labels

- **Prototype**: implemented enough to experiment with, likely to change.
- **Specified**: behavior is documented and covered by tests.
- **Stable**: compatibility is promised for the language version.

The lexer is prototype-level today. Every new stage should move only the relevant behavior toward specified, with executable tests beside the implementation.

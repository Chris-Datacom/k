# Self-Hosting and Bootstrap Plan

Self-compilation is a sequence of working compilers, not a flag that can be switched on at the end. Each stage must be able to build the next stage and must have a reproducible artifact or hash.

## Stage 0: Rust host (temporary)

The Rust compiler implements lexing, parsing, semantic analysis, and code generation. The compiler library should avoid depending on the command-line layer so it can later be translated into K.
Rust is retained as a recovery compiler until two reproducible releases have
been built by K itself. This prevents a broken bootstrap from making the
language unrecoverable.

## Stage 1: Useful native compiler

Add a parser, typed abstract syntax tree, name resolution, and a small code
generator. The current implementation reaches the first part of this stage:
`k emit` produces x86-64 System V assembly without a runtime or libc. This is
intentionally suitable as a foundation for a freestanding kernel, although
boot entry points, memory access, interrupts, and a kernel ABI are not yet
implemented. The next backend work must define those interfaces rather than
silently assuming an operating-system process.

## Stage 2: K compiler written in K

Implement a K version of the lexer and parser using only the stable subset already supported by the Rust compiler. Keep the Rust compiler as the trusted reference compiler. Differential tests should compare tokens, diagnostics, and generated output for the same inputs.

The host boundary is now explicit: compiler stages accept source buffers
through `driver::compile_source`, while the command-line `compile` command
only reads UTF-8 source bytes and writes the resulting assembly artifact.

The first K-written components are [`compiler/lexer.k`](../compiler/lexer.k)
and [`compiler/parser.k`](../compiler/parser.k).
It uses caller-owned `char*` input and a `struct Token*` output record, with
no allocation or runtime dependency. It emits complete spans for identifiers,
decimal integer literals, keywords, and basic punctuation while skipping
whitespace and `//` comments. The Rust test
suite compiles it through the normal pipeline, making it a checked bootstrap
artifact while the K implementation grows toward a complete token stream.

The next stage adds the parser boundary in [`compiler/parser.k`](../compiler/parser.k).
It keeps the same caller-owned `Lexer` model, adds a `Parser` state record,
and implements structural recursive descent for structs, function signatures,
blocks, declarations, control flow, and expression-shaped token sequences.
The Rust host compiles this source through the normal pipeline as another
checked bootstrap artifact. The parser also exposes `SourceSlice`, `AstProgram`, `AstFunction`, and
`AstNode` records. `parse_program_ast` writes function records into a
caller-provided buffer and reports capacity overflow explicitly; no allocator
or hidden global storage is involved. `ExpressionArena` now stores nested primary, unary, binary, call, index, and
field nodes in caller-provided memory. Each node contains its kind, operator,
source span, child indices, and auxiliary value. Capacity exhaustion and parse
errors remain explicit arena status codes.

Statement and function records now use the same ownership model. `AstStorage`
groups caller-provided function, parameter, expression, and statement buffers;
`parse_program_tree` fills those buffers and records body ranges, statement
kinds, expression roots, parameters, and `if`/`else` block ranges. The parser
does not allocate or retain source-owned state outside the supplied storage.

The first semantic boundary is also present in `compiler/parser.k`.
`SemanticChecker` owns a caller-provided symbol buffer and validates expression
child indices, name references, duplicate declarations, parameters, and
statement ranges. `semantic_check_parser` binds the parser's arenas without
copying them. It now also retains primitive/pointer type kinds on functions and
parameters, infers expression types, and checks returns, assignments, binary
operators, indexing, and boolean control-flow conditions. Calls and struct
field typing now use caller-owned function, struct, and field signature tables.
Call arity and every argument type are checked through a caller-owned linked
argument arena. Its status codes distinguish
symbol capacity, duplicate names, unresolved names, malformed AST storage,
invalid operators, type mismatches, unsupported callee forms, unknown
signatures, arity mismatches, return mismatches, assignment mismatches, and
non-boolean conditions.

The first typed IR boundary is also implemented in `compiler/parser.k`.
`IrStorage` owns fixed-capacity instruction, function, and basic-block arrays.
The lowering stage emits typed stack-oriented records for constants, names,
unary and binary expressions, calls, declarations, assignments, returns, and
expression statements. Nested blocks, `if`, and `while` emit explicit branch
and jump records. Target-specific code generation remains separate, and
unsupported statement kinds fail explicitly.

The complete staged plan, including the semantic checker, IR, backend, driver,
and reproducibility gates, is documented in
[`docs/self-hosting.md`](self-hosting.md).

The lexer now exposes a stateful token-stream boundary:

```k
struct Lexer {
    char* source;
    int length;
    int position;
}

void lexer_init(char* source, int length, struct Lexer* lexer);
int next_token(struct Lexer* lexer, struct Token* out);
```

The parser can own the `Lexer` storage and advance it without allocations;
each call writes one `Token` record and returns its kind.

`parser.k` now builds caller-owned AST, expression, statement, semantic, and
initial typed-IR records. It is checked by the Rust host and listed in
`compiler/sources.txt`, making the frontend source set reproducible.

[`compiler/hello.k`](../compiler/hello.k) is the first executable K smoke
test. It calls the freestanding Linux `print` intrinsic, which lowers to the
x86-64 `write` system call. The Rust host can compile it with:

```text
cargo run -- compile compiler/hello.k target/hello.s
```

The resulting assembly is intended to be assembled and linked on Linux with
the repository's x86-64 target assumptions. Windows can validate compilation
and inspect the generated assembly, but cannot execute this Linux syscall
output natively. Programs defining `main` receive a minimal `_start` wrapper
that calls `main` and exits with its integer return value.

The repository now includes repeatable bootstrap setup:

```text
scripts/bootstrap.sh       # Linux/WSL: test, compile, assemble, link, run
scripts/bootstrap.ps1      # Windows: test, compile, and use WSL when present
scripts/check-bootstrap.sh # check every source listed in compiler/sources.txt
scripts/check-bootstrap.ps1 # Windows equivalent of the manifest check
```

The source manifest is deliberately explicit so future K-built stages can
reproduce the exact compiler input set.

## Stage 3: K builds K

Compile the K compiler source with the Rust host compiler, then use that resulting K compiler to compile itself. The two outputs must agree on a defined set of source programs. This is the first self-hosting milestone.

## Stage 4: Reproducible K bootstrap

Build the complete compiler with the K compiler, then build it again with the
resulting compiler. Compare source manifests, generated assembly, and
executable behavior. Repeat this process for two tagged releases. Both
releases must pass the same language and compiler conformance suite.

## Stage 5: Remove the Rust implementation

After two reproducible K-built releases, move target-specific code into a
small documented backend boundary and remove the Rust implementation from the
normal workspace. Preserve a separately archived recovery compiler until the
K toolchain has an independent release process.

## Invariants for every stage

- The language specification and implementation tests change together.
- A bootstrap compiler never silently accepts syntax that the next compiler cannot read.
- Compiler output has a deterministic mode for comparison.
- Error messages remain source-spanned and actionable.
- The bootstrap process is documented as commands a new contributor can run.

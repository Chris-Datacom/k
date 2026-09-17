#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

cargo test
cargo run -- compile compiler/hello.k target/hello.s
cargo run -- compile compiler/lexer.k target/lexer.s
cat compiler/lexer.k compiler/lexer_conformance_harness.k > target/lexer_conformance.k
cargo run -- compile target/lexer_conformance.k target/lexer_conformance.s
as --64 -o target/hello.o target/hello.s
ld -o target/hello target/hello.o
./target/hello
as --64 -o target/lexer_conformance.o target/lexer_conformance.s
ld -o target/lexer_conformance target/lexer_conformance.o
./target/lexer_conformance

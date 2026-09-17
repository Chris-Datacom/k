#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

cargo test
cargo run -- compile compiler/hello.k target/hello.s
as --64 -o target/hello.o target/hello.s
ld -o target/hello target/hello.o
./target/hello

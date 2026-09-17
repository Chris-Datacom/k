#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

while IFS= read -r source; do
    [ -z "$source" ] && continue
    cargo run --quiet -- check "$source"
done < compiler/sources.txt

cargo test --quiet

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

cargo test
cargo run --quiet -- check compiler\lexer.k
cargo run --quiet -- check compiler\hello.k

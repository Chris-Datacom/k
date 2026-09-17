$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
$root = (Get-Location).Path

cargo test
cargo run -- compile compiler\hello.k target\hello.s
cargo run -- compile compiler\lexer.k target\lexer.s
$lexerSource = (Get-Content -Raw compiler\lexer.k) + "`n" + (Get-Content -Raw compiler\lexer_conformance_harness.k)
[IO.File]::WriteAllText((Join-Path $root "target\lexer_conformance.k"), $lexerSource, [Text.UTF8Encoding]::new($false))
cargo run -- compile target\lexer_conformance.k target\lexer_conformance.s

if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
    Write-Host "K assembly generated at target\hello.s"
    Write-Host "K lexer conformance assembly generated at target\lexer_conformance.s"
    Write-Host "Install WSL to assemble and run the Linux target."
    exit 0
}

$drive = $root.Substring(0, 1).ToLower()
$rest = $root.Substring(2).Replace("\", "/")
$wslRoot = "/mnt/$drive$rest"
wsl.exe sh -lc "set -eu; cd '$wslRoot'; as --64 -o target/hello.o target/hello.s; ld -o target/hello target/hello.o; ./target/hello"
wsl.exe sh -lc "set -eu; cd '$wslRoot'; as --64 -o target/lexer_conformance.o target/lexer_conformance.s; ld -o target/lexer_conformance target/lexer_conformance.o; ./target/lexer_conformance"

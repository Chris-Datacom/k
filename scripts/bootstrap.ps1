$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
$root = (Get-Location).Path

cargo test
cargo run -- compile compiler\hello.k target\hello.s

if (-not (Get-Command wsl.exe -ErrorAction SilentlyContinue)) {
    Write-Host "K assembly generated at target\hello.s"
    Write-Host "Install WSL to assemble and run the Linux target."
    exit 0
}

$drive = $root.Substring(0, 1).ToLower()
$rest = $root.Substring(2).Replace("\", "/")
$wslRoot = "/mnt/$drive$rest"
wsl sh -lc "set -eu; cd '$wslRoot'; as --64 -o target/hello.o target/hello.s; ld -o target/hello target/hello.o; ./target/hello"

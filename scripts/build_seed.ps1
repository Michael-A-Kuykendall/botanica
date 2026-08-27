# Build cultivated seed (gate2 pilot → duckdb + parquet + MANIFEST)
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)
cargo run --bin build_seed -- .
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Write-Host "Seed build finished. See data/manifests/"

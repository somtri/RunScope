$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

Write-Host "Fetching Rust dependencies..."
cargo fetch --manifest-path "$Root\backend\Cargo.toml"

Write-Host "Installing frontend dependencies..."
Push-Location "$Root\frontend"
try {
    npm install
}
finally {
    Pop-Location
}

Write-Host "run_scope dependencies are ready."

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

Write-Host "Starting run_scope backend at http://localhost:8080"
$Backend = Start-Process cargo -ArgumentList "run" -WorkingDirectory "$Root\backend" -PassThru

Write-Host "Starting run_scope frontend at http://localhost:5173"
$Frontend = Start-Process npm -ArgumentList "run", "dev" -WorkingDirectory "$Root\frontend" -PassThru

try {
    Wait-Process -Id $Backend.Id, $Frontend.Id
}
finally {
    Stop-Process -Id $Backend.Id, $Frontend.Id -ErrorAction SilentlyContinue
}

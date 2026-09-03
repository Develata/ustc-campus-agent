param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot

# Keep this script ASCII-only for Windows PowerShell 5.1 compatibility.
& docker compose down
if ($LASTEXITCODE -ne 0) { throw "docker compose down failed." }
Write-Host "MVP stopped. Test state is preserved in the Docker volume."

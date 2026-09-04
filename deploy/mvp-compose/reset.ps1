param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot

# Keep this script ASCII-only for Windows PowerShell 5.1 compatibility.
$answer = Read-Host "This deletes all local test state for this MVP. Type RESET to continue"
if ($answer -cne "RESET") {
  Write-Host "Cancelled."
  exit 0
}

& docker compose down --volumes
if ($LASTEXITCODE -ne 0) { throw "docker compose down --volumes failed." }
$adminHashPath = Join-Path $PSScriptRoot "secrets\admin-password.phc"
if (Test-Path -LiteralPath $adminHashPath) {
  Remove-Item -LiteralPath $adminHashPath -Force
}
Write-Host "MVP state and local deployment access password have been reset."

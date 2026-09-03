param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot
& docker compose down
if ($LASTEXITCODE -ne 0) { throw "docker compose down 失败。" }
Write-Host "MVP 已停止；测试状态仍保留在 Docker volume 中。"

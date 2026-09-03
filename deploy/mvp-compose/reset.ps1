param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot
$answer = Read-Host "这会删除本 MVP 的全部本地测试状态。输入 RESET 继续"
if ($answer -cne "RESET") {
  Write-Host "已取消。"
  exit 0
}
& docker compose down --volumes
if ($LASTEXITCODE -ne 0) { throw "docker compose down --volumes 失败。" }
Write-Host "MVP 状态已重置。"

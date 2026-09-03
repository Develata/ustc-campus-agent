param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot

& docker info *> $null
if ($LASTEXITCODE -ne 0) {
  throw "Docker Desktop 未启动，或当前用户无法连接 Docker Engine。"
}

& docker compose up --build -d
if ($LASTEXITCODE -ne 0) {
  throw "docker compose up 失败。"
}

$port = if ($env:UCA_MVP_PORT) { $env:UCA_MVP_PORT } else { "8787" }
$url = "http://127.0.0.1:$port"
$deadline = (Get-Date).AddMinutes(5)
while ((Get-Date) -lt $deadline) {
  try {
    $health = Invoke-RestMethod -Uri "$url/healthz" -TimeoutSec 3
    if (($health.schema -eq "ustc-agentd-health/v1") -and ($health.status -eq "ok")) {
      Write-Host "MVP 已就绪：$url" -ForegroundColor Green
      Start-Process $url
      exit 0
    }
  } catch {
    Start-Sleep -Seconds 2
  }
}

& docker compose ps
& docker compose logs --no-color --tail 120
throw "MVP 在 5 分钟内没有通过健康检查。"

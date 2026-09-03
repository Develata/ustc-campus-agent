param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot

# Keep this script ASCII-only for Windows PowerShell 5.1 compatibility.
& docker info *> $null
if ($LASTEXITCODE -ne 0) {
  throw "Docker Desktop is not running, or the current user cannot connect to Docker Engine."
}

& docker compose up --build -d
if ($LASTEXITCODE -ne 0) {
  throw "docker compose up failed."
}

$port = if ($env:UCA_MVP_PORT) { $env:UCA_MVP_PORT } else { "8787" }
$url = "http://127.0.0.1:$port"
$deadline = (Get-Date).AddMinutes(5)
while ((Get-Date) -lt $deadline) {
  try {
    $health = Invoke-RestMethod -Uri "$url/healthz" -TimeoutSec 3
    if (($health.schema -eq "ustc-agentd-health/v1") -and ($health.status -eq "ok")) {
      Write-Host "MVP is ready: $url" -ForegroundColor Green
      Start-Process -FilePath $url
      exit 0
    }
  } catch {
    Start-Sleep -Seconds 2
  }
}

& docker compose ps
& docker compose logs --no-color --tail 120
throw "MVP did not pass its health check within 5 minutes."

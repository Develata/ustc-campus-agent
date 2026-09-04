param()
$ErrorActionPreference = "Stop"
Set-Location -LiteralPath $PSScriptRoot

# Keep this script ASCII-only for Windows PowerShell 5.1 compatibility.
function Normalize-MvpSetting([string]$Value) {
  $normalized = $Value.Trim()
  if ($normalized.Length -ge 2) {
    $first = $normalized[0]
    $last = $normalized[$normalized.Length - 1]
    if (($first -eq '"' -and $last -eq '"') -or ($first -eq "'" -and $last -eq "'")) {
      $normalized = $normalized.Substring(1, $normalized.Length - 2)
    }
  }
  return $normalized
}

function Normalize-MvpDotEnvSetting([string]$Value) {
  $normalized = $Value.Trim()
  $match = [regex]::Match($normalized, '^"([^"]*)"\s*(?:#.*)?$')
  if ($match.Success) {
    $normalized = $match.Groups[1].Value
  } else {
    $match = [regex]::Match($normalized, "^'([^']*)'\s*(?:#.*)?$")
    if ($match.Success) {
      $normalized = $match.Groups[1].Value
    } else {
      $match = [regex]::Match($normalized, '^(.*\S)\s+#.*$')
      if ($match.Success) {
        $normalized = Normalize-MvpSetting $match.Groups[1].Value
      } else {
        $normalized = Normalize-MvpSetting $normalized
      }
    }
  }
  if ($normalized.Contains('$')) {
    throw "Compose interpolation is not supported for security-critical .env values; use a literal value."
  }
  return $normalized
}

function Assert-MvpDotEnvContract {
  $dotenv = Join-Path $PSScriptRoot ".env"
  $item = Get-Item -LiteralPath $dotenv -Force -ErrorAction SilentlyContinue
  if ($null -eq $item) {
    return
  }
  if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
    throw ".env must be a regular non-symlink file."
  }
  if (-not (Test-Path -LiteralPath $dotenv -PathType Leaf)) {
    throw ".env must be a readable regular non-symlink file."
  }
  $counts = @{
    UCA_AGENT_PROVIDER = 0
    UCA_AGENT_API_KEY_SOURCE = 0
    UCA_ADMIN_PASSWORD_HASH_SOURCE = 0
  }
  foreach ($line in [System.IO.File]::ReadAllLines($dotenv)) {
    $trimmed = $line.TrimStart()
    $match = [regex]::Match(
      $trimmed,
      '^(?:export\s+)?(UCA_AGENT_PROVIDER|UCA_AGENT_API_KEY_SOURCE|UCA_ADMIN_PASSWORD_HASH_SOURCE)(?:\s*=|\s*$)'
    )
    if (-not $match.Success) {
      continue
    }
    $name = $match.Groups[1].Value
    if (-not $line.StartsWith("$name=", [StringComparison]::Ordinal)) {
      throw "$name must use an exact column-zero KEY=value assignment in .env."
    }
    $counts[$name] += 1
    if ($counts[$name] -ne 1) {
      throw "Duplicate $name definitions are forbidden in .env."
    }
    $null = Normalize-MvpDotEnvSetting $line.Substring($name.Length + 1)
  }
}

function Get-MvpSetting([string]$Name, [string]$DefaultValue) {
  $processValue = [Environment]::GetEnvironmentVariable($Name, "Process")
  if ($null -ne $processValue) {
    $normalized = Normalize-MvpSetting $processValue
    if ([String]::IsNullOrWhiteSpace($normalized)) {
      throw "$Name must not be empty."
    }
    return $normalized
  }
  $dotenv = Join-Path $PSScriptRoot ".env"
  if (Test-Path -LiteralPath $dotenv -PathType Leaf) {
    foreach ($line in [System.IO.File]::ReadAllLines($dotenv)) {
      $prefix = "$Name="
      if ($line.StartsWith($prefix, [StringComparison]::Ordinal)) {
        $normalized = Normalize-MvpDotEnvSetting $line.Substring($prefix.Length)
        if ([String]::IsNullOrWhiteSpace($normalized)) {
          throw "$Name must not be empty."
        }
        return $normalized
      }
    }
  }
  return $DefaultValue
}

function Show-ComposeDiagnostics {
  Write-Host "--- docker compose ps -a ---" -ForegroundColor Yellow
  & docker compose ps -a
  Write-Host "--- docker compose logs (last 120 lines) ---" -ForegroundColor Yellow
  & docker compose logs --no-color --tail 120 mvp
}

Assert-MvpDotEnvContract

$provider = Get-MvpSetting "UCA_AGENT_PROVIDER" "mock"
$adminUsername = Get-MvpSetting "UCA_ADMIN_USERNAME" "admin"
if ($provider -eq "openai-compatible") {
  $keySource = Get-MvpSetting "UCA_AGENT_API_KEY_SOURCE" ""
  if ([String]::IsNullOrEmpty($keySource)) {
    throw "UCA_AGENT_API_KEY_SOURCE is required in openai-compatible mode."
  }
  if ([System.IO.Path]::IsPathRooted($keySource)) {
    $keyPath = [System.IO.Path]::GetFullPath($keySource)
  } else {
    $keyPath = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot $keySource))
  }
  if (-not (Test-Path -LiteralPath $keyPath -PathType Leaf)) {
    throw "Provider key source must be a readable regular file."
  }
  $item = Get-Item -LiteralPath $keyPath -Force
  if (($item.Attributes -band [System.IO.FileAttributes]::ReparsePoint) -ne 0) {
    throw "Provider key source must not be a symlink or reparse point."
  }
  $placeholder = Join-Path $PSScriptRoot "mock-provider-key.txt"
  $keyValue = [System.IO.File]::ReadAllText($keyPath).Trim()
  $placeholderValue = [System.IO.File]::ReadAllText($placeholder).Trim()
  if ([String]::Equals($keyValue, $placeholderValue, [StringComparison]::Ordinal)) {
    throw "The bundled mock provider placeholder is forbidden in openai-compatible mode."
  }
}

& docker info *> $null
if ($LASTEXITCODE -ne 0) {
  throw "Docker Desktop is not running, or the current user cannot connect to Docker Engine."
}

$adminHashSource = Get-MvpSetting "UCA_ADMIN_PASSWORD_HASH_SOURCE" ".\secrets\admin-password.phc"
if ([IO.Path]::IsPathRooted($adminHashSource)) {
  $adminHashPath = [IO.Path]::GetFullPath($adminHashSource)
} else {
  $adminHashPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot $adminHashSource))
}
$defaultAdminHashPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot "secrets\admin-password.phc"))
if (-not (Test-Path -LiteralPath $adminHashPath -PathType Leaf)) {
  if (-not $adminHashPath.Equals($defaultAdminHashPath, [StringComparison]::OrdinalIgnoreCase)) {
    throw "The configured UCA_ADMIN_PASSWORD_HASH_SOURCE does not exist as a regular file."
  }
  & powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "set-admin-password.ps1") -Initial
  if ($LASTEXITCODE -ne 0) {
    throw "Local deployment access setup failed."
  }
}
$adminHashItem = Get-Item -LiteralPath $adminHashPath -Force
if (($adminHashItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {
  throw "The local administrator password verifier must not be a symlink or reparse point."
}
$adminHash = [IO.File]::ReadAllText($adminHashPath)
if ($adminHash -notmatch '^\$argon2id\$v=19\$m=19456,t=2,p=1\$[A-Za-z0-9+/]{22}\$[A-Za-z0-9+/]{43}$') {
  throw "The local administrator password verifier is invalid. Run set-admin-password.cmd."
}
$adminHash = $null

& docker compose up --build -d
if ($LASTEXITCODE -ne 0) {
  throw "docker compose up failed."
}

$port = $null
$portWatch = [Diagnostics.Stopwatch]::StartNew()
while ($portWatch.Elapsed.TotalSeconds -lt 30) {
  $publishedLines = @(& docker compose port mvp 8787 2>$null | Where-Object { -not [String]::IsNullOrWhiteSpace($_) })
  $portExit = $LASTEXITCODE
  if ($portExit -eq 0 -and $publishedLines.Count -gt 0) {
    if ($publishedLines.Count -ne 1 -or $publishedLines[0] -notmatch '^127\.0\.0\.1:([0-9]{1,5})$') {
      Show-ComposeDiagnostics
      throw "Unexpected Compose published address; expected exactly one 127.0.0.1:<port> binding."
    }
    $candidatePort = [int]$Matches[1]
    if ($candidatePort -lt 1 -or $candidatePort -gt 65535) {
      Show-ComposeDiagnostics
      throw "Invalid Compose published port."
    }
    $port = $candidatePort
    break
  }
  Start-Sleep -Seconds 1
}
if ($null -eq $port) {
  Show-ComposeDiagnostics
  throw "Timed out waiting for docker compose port mvp 8787 after 30 seconds."
}
$url = "http://127.0.0.1:$port"
$healthWatch = [Diagnostics.Stopwatch]::StartNew()
while ($healthWatch.Elapsed.TotalMinutes -lt 5) {
  try {
    $health = Invoke-RestMethod -Uri "$url/healthz" -TimeoutSec 3
    if (($health.schema -eq "ustc-agentd-health/v1") -and ($health.status -eq "ok")) {
      Write-Host "MVP is ready: $url" -ForegroundColor Green
      Write-Host "Local deployment access username: $adminUsername" -ForegroundColor Green
      try {
        Start-Process -FilePath $url
      } catch {
        Write-Warning "MVP is ready, but the browser could not be opened automatically: $url"
      }
      exit 0
    }
    Show-ComposeDiagnostics
    throw "The health endpoint returned an incompatible response."
  } catch {
    if ($_.Exception.Message -eq "The health endpoint returned an incompatible response.") {
      throw
    }
    $containerId = (& docker compose ps -q mvp 2>$null | Select-Object -First 1)
    if (-not [String]::IsNullOrWhiteSpace($containerId)) {
      $containerState = (& docker inspect --format '{{.State.Status}}' $containerId 2>$null | Select-Object -First 1)
      if ($containerState -eq "exited" -or $containerState -eq "restarting" -or $containerState -eq "dead") {
        Show-ComposeDiagnostics
        throw "The MVP container stopped before becoming healthy."
      }
    }
    Start-Sleep -Seconds 2
  }
}

Show-ComposeDiagnostics
throw "MVP did not pass its health check within 5 minutes."

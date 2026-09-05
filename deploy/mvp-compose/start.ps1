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
  }
  foreach ($line in [System.IO.File]::ReadAllLines($dotenv)) {
    $trimmed = $line.TrimStart()
    $match = [regex]::Match(
      $trimmed,
      '^(?:export\s+)?(UCA_AGENT_PROVIDER|UCA_AGENT_API_KEY_SOURCE)(?:\s*=|\s*$)'
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

Assert-MvpDotEnvContract

$provider = Get-MvpSetting "UCA_AGENT_PROVIDER" "mock"
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

& docker compose up --build -d
if ($LASTEXITCODE -ne 0) {
  throw "docker compose up failed."
}

# Drain native stdout before selecting a line. In Windows PowerShell 5.1,
# Select-Object -First can stop the native pipeline and leave exit code -1.
$publishedLines = @(& docker compose port mvp 8787)
$portExitCode = $LASTEXITCODE
if ($portExitCode -ne 0) {
  throw "docker compose port failed."
}
$published = $publishedLines | Select-Object -First 1
if ($published -notmatch '^127\.0\.0\.1:([0-9]{1,5})$') {
  throw "Unexpected Compose published address."
}
$port = [int]$Matches[1]
if ($port -lt 1 -or $port -gt 65535) {
  throw "Invalid Compose published port."
}
$url = "http://127.0.0.1:$port"
$deadline = (Get-Date).AddMinutes(5)
while ((Get-Date) -lt $deadline) {
  try {
    $health = Invoke-RestMethod -Uri "$url/healthz" -TimeoutSec 3
    if (($health.schema -eq "ustc-agentd-health/v1") -and ($health.status -eq "ok")) {
      Write-Host "MVP is ready: $url" -ForegroundColor Green
      try {
        Start-Process -FilePath $url
      } catch {
        Write-Warning "MVP is ready, but the browser could not be opened automatically: $url"
      }
      exit 0
    }
  } catch {
    Start-Sleep -Seconds 2
  }
}

& docker compose ps
& docker compose logs --no-color --tail 120
throw "MVP did not pass its health check within 5 minutes."

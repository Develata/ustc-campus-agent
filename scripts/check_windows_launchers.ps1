$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($PSVersionTable.PSEdition -ne "Desktop" -or $PSVersionTable.PSVersion.Major -ne 5) {
  throw "Windows PowerShell 5.1 required."
}

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$launcherRoot = Join-Path $repoRoot "deploy\mvp-compose"
foreach ($launcher in Get-ChildItem -LiteralPath $launcherRoot -File | Where-Object { $_.Extension -in ".ps1", ".cmd" }) {
  $bytes = [System.IO.File]::ReadAllBytes($launcher.FullName)
  if ($bytes.Length -eq 0 -or $bytes[$bytes.Length - 1] -ne 10) {
    throw "$($launcher.Name): missing LF"
  }
  if ($bytes -contains 0) {
    throw "$($launcher.Name): contains NUL"
  }
  foreach ($byte in $bytes) {
    if ($byte -gt 127) {
      throw "$($launcher.Name): non-ASCII byte"
    }
  }
}
foreach ($script in Get-ChildItem -LiteralPath $launcherRoot -Filter "*.ps1" -File) {
  $tokens = $null
  $errors = $null
  [void][System.Management.Automation.Language.Parser]::ParseFile($script.FullName, [ref]$tokens, [ref]$errors)
  if ($errors.Count -ne 0) {
    throw "$($script.Name): parser errors: $errors"
  }
}
$startTokens = $null
$startErrors = $null
$startAst = [System.Management.Automation.Language.Parser]::ParseFile(
  (Join-Path $launcherRoot "start.ps1"),
  [ref]$startTokens,
  [ref]$startErrors
)
$startProcessCalls = @($startAst.FindAll({
  param($node)
  ($node -is [System.Management.Automation.Language.CommandAst]) -and
    ($node.GetCommandName() -eq "Start-Process")
}, $true))
if ($startProcessCalls.Count -ne 1) {
  throw "start.ps1 must contain exactly one browser launch."
}
$tryDepth = 0
$ancestor = $startProcessCalls[0].Parent
while ($null -ne $ancestor) {
  if ($ancestor -is [System.Management.Automation.Language.TryStatementAst]) {
    $tryDepth += 1
  }
  $ancestor = $ancestor.Parent
}
if ($tryDepth -lt 2) {
  throw "start.ps1 browser launch must be isolated from the health-check retry."
}

$shim = Join-Path $env:RUNNER_TEMP "uca-fake-docker"
[void](New-Item -ItemType Directory -Force -Path $shim)
[System.IO.File]::WriteAllText((Join-Path $shim "docker.cmd"), "@exit /b 23`r`n", [System.Text.Encoding]::ASCII)
$oldPath = $env:PATH
$oldLocation = (Get-Location).Path
$oldProvider = [Environment]::GetEnvironmentVariable("UCA_AGENT_PROVIDER", "Process")
$oldKeySource = [Environment]::GetEnvironmentVariable("UCA_AGENT_API_KEY_SOURCE", "Process")
$oldAdminHashSource = [Environment]::GetEnvironmentVariable("UCA_ADMIN_PASSWORD_HASH_SOURCE", "Process")
$dotenvLauncherRoot = Join-Path $env:RUNNER_TEMP "uca-commented-dotenv-launchers"
try {
  $env:PATH = "$shim;$oldPath"
  if (Test-Path -LiteralPath $dotenvLauncherRoot) {
    Remove-Item -LiteralPath $dotenvLauncherRoot -Recurse -Force
  }
  Copy-Item -LiteralPath $launcherRoot -Destination $dotenvLauncherRoot -Recurse
  [System.IO.File]::WriteAllLines(
    (Join-Path $dotenvLauncherRoot ".env"),
    @(
      'UCA_AGENT_PROVIDER="openai-compatible" # local provider picked by $MODE',
      'UCA_AGENT_API_KEY_SOURCE="./mock-provider-key.txt" # owner-only source'
    ),
    [System.Text.Encoding]::ASCII
  )
  [Environment]::SetEnvironmentVariable("UCA_AGENT_PROVIDER", $null, "Process")
  [Environment]::SetEnvironmentVariable("UCA_AGENT_API_KEY_SOURCE", $null, "Process")
  [Environment]::SetEnvironmentVariable("UCA_ADMIN_PASSWORD_HASH_SOURCE", $null, "Process")
  $commentedProviderRejected = $false
  try {
    & (Join-Path $dotenvLauncherRoot "start.ps1")
  } catch {
    if ($_.Exception.Message -ne "The bundled mock provider placeholder is forbidden in openai-compatible mode.") {
      throw
    }
    $commentedProviderRejected = $true
  }
  if (-not $commentedProviderRejected) {
    throw "start.ps1 did not parse Compose-style .env comments before provider preflight."
  }

  # Preserve the generated NBSP fixture while keeping this script itself ASCII-only.
  $unsafeDotEnvEncoding = New-Object System.Text.UTF8Encoding($false)
  $unsafeDotEnvCases = @(
    @{
      Name = "provider-interpolation"
      Lines = @('UCA_AGENT_PROVIDER=${MODE:-openai-compatible}')
      Expected = "Compose interpolation is not supported for security-critical .env values; use a literal value."
    },
    @{
      Name = "key-source-interpolation"
      Lines = @('UCA_AGENT_PROVIDER=mock', 'UCA_AGENT_API_KEY_SOURCE=${KEY_SOURCE:-./secrets/key.txt}')
      Expected = "Compose interpolation is not supported for security-critical .env values; use a literal value."
    },
    @{
      Name = "admin-hash-source-interpolation"
      Lines = @('UCA_AGENT_PROVIDER=mock', 'UCA_ADMIN_PASSWORD_HASH_SOURCE=${ADMIN_HASH_SOURCE:-./secrets/admin-password.phc}')
      Expected = "Compose interpolation is not supported for security-critical .env values; use a literal value."
    },
    @{
      Name = "duplicate-provider"
      Lines = @('UCA_AGENT_PROVIDER=mock', 'UCA_AGENT_PROVIDER=openai-compatible')
      Expected = "Duplicate UCA_AGENT_PROVIDER definitions are forbidden in .env."
    },
    @{
      Name = "duplicate-key-source"
      Lines = @('UCA_AGENT_PROVIDER=mock', 'UCA_AGENT_API_KEY_SOURCE=./first.txt', 'UCA_AGENT_API_KEY_SOURCE=./second.txt')
      Expected = "Duplicate UCA_AGENT_API_KEY_SOURCE definitions are forbidden in .env."
    },
    @{
      Name = "duplicate-admin-hash-source"
      Lines = @('UCA_AGENT_PROVIDER=mock', 'UCA_ADMIN_PASSWORD_HASH_SOURCE=./first.phc', 'UCA_ADMIN_PASSWORD_HASH_SOURCE=./second.phc')
      Expected = "Duplicate UCA_ADMIN_PASSWORD_HASH_SOURCE definitions are forbidden in .env."
    },
    @{
      Name = "leading-provider"
      Lines = @(' UCA_AGENT_PROVIDER=openai-compatible')
      Expected = "UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env."
    },
    @{
      Name = "bare-provider"
      Lines = @('UCA_AGENT_PROVIDER')
      Expected = "UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env."
    },
    @{
      Name = "export-provider"
      Lines = @('export UCA_AGENT_PROVIDER=openai-compatible')
      Expected = "UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env."
    },
    @{
      Name = "spaced-equals-provider"
      Lines = @('UCA_AGENT_PROVIDER =openai-compatible')
      Expected = "UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env."
    },
    @{
      Name = "unicode-leading-provider"
      Lines = @(("{0}UCA_AGENT_PROVIDER=openai-compatible" -f [char]0x00A0))
      Expected = "UCA_AGENT_PROVIDER must use an exact column-zero KEY=value assignment in .env."
    }
  )
  foreach ($unsafeCase in $unsafeDotEnvCases) {
    [System.IO.File]::WriteAllLines(
      (Join-Path $dotenvLauncherRoot ".env"),
      $unsafeCase.Lines,
      $unsafeDotEnvEncoding
    )
    $rejected = $false
    try {
      & (Join-Path $dotenvLauncherRoot "start.ps1")
    } catch {
      if ($_.Exception.Message -ne $unsafeCase.Expected) {
        throw
      }
      $rejected = $true
    }
    if (-not $rejected) {
      throw "start.ps1 accepted unsafe .env case $($unsafeCase.Name)."
    }
  }

  $symlinkTarget = Join-Path $dotenvLauncherRoot "symlink-target.env"
  $dotenvPath = Join-Path $dotenvLauncherRoot ".env"
  [System.IO.File]::WriteAllText($symlinkTarget, "UCA_AGENT_PROVIDER=openai-compatible`n", [System.Text.Encoding]::ASCII)
  Remove-Item -LiteralPath $dotenvPath -Force
  New-Item -ItemType SymbolicLink -Path $dotenvPath -Target $symlinkTarget | Out-Null
  $symlinkRejected = $false
  try {
    & (Join-Path $dotenvLauncherRoot "start.ps1")
  } catch {
    if ($_.Exception.Message -ne ".env must be a regular non-symlink file.") {
      throw
    }
    $symlinkRejected = $true
  }
  if (-not $symlinkRejected) {
    throw "start.ps1 accepted a symlinked .env."
  }
  Remove-Item -LiteralPath $dotenvPath -Force

  $normalizedPlaceholder = Join-Path $env:RUNNER_TEMP "uca-normalized-placeholder.txt"
  $unicodePadding = [char]0x00A0
  [System.IO.File]::WriteAllText(
    $normalizedPlaceholder,
    ($unicodePadding + "unused-placeholder-for-deterministic-mock-mode" + $unicodePadding + "`r`n"),
    (New-Object System.Text.UTF8Encoding($false))
  )
  $env:UCA_AGENT_PROVIDER = '"openai-compatible"'
  $env:UCA_AGENT_API_KEY_SOURCE = "'$normalizedPlaceholder'"
  $placeholderRejected = $false
  try {
    & (Join-Path $launcherRoot "start.ps1")
  } catch {
    if ($_.Exception.Message -ne "The bundled mock provider placeholder is forbidden in openai-compatible mode.") {
      throw
    }
    $placeholderRejected = $true
  }
  if (-not $placeholderRejected) {
    throw "start.ps1 accepted the bundled mock provider placeholder."
  }

  $realKey = Join-Path $env:RUNNER_TEMP "uca-provider-test-key.txt"
  [System.IO.File]::WriteAllText($realKey, "non-secret-launcher-test-value`n", [System.Text.Encoding]::ASCII)
  $env:UCA_AGENT_API_KEY_SOURCE = $realKey
  $failedClosed = $false
  try {
    & (Join-Path $launcherRoot "start.ps1")
  } catch {
    if ($_.Exception.Message -ne "Docker Desktop is not running, or the current user cannot connect to Docker Engine.") {
      throw
    }
    $failedClosed = $true
  }
  if (-not $failedClosed) {
    throw "start.ps1 accepted a failing Docker command."
  }

  foreach ($wrapperName in @("start.cmd", "stop.cmd", "reset.cmd", "set-admin-password.cmd")) {
    $wrapper = Join-Path $launcherRoot $wrapperName
    if ($wrapperName -eq "reset.cmd") {
      & $env:ComSpec /d /c "echo RESET| call `"$wrapper`""
    } else {
      & $env:ComSpec /d /c "call `"$wrapper`" <NUL"
    }
    if ($LASTEXITCODE -eq 0) {
      throw "$wrapperName discarded the PowerShell failure status."
    }
  }
} finally {
  $env:PATH = $oldPath
  $env:UCA_AGENT_PROVIDER = $oldProvider
  $env:UCA_AGENT_API_KEY_SOURCE = $oldKeySource
  $env:UCA_ADMIN_PASSWORD_HASH_SOURCE = $oldAdminHashSource
  Set-Location -LiteralPath $oldLocation
  if (Test-Path -LiteralPath $dotenvLauncherRoot) {
    Remove-Item -LiteralPath $dotenvLauncherRoot -Recurse -Force
  }
}

Write-Output "WINDOWS_POWERSHELL_51_LAUNCHERS=PASS"

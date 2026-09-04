param([switch]$Initial)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
Set-Location -LiteralPath $PSScriptRoot

# Keep this script ASCII-only for Windows PowerShell 5.1 compatibility.
function Convert-SecureStringToBase64([Security.SecureString]$SecureValue) {
  $pointer = [IntPtr]::Zero
  $plain = $null
  $bytes = $null
  try {
    $pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($SecureValue)
    $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer)
    $bytes = [Text.Encoding]::UTF8.GetBytes($plain)
    if ($bytes.Length -lt 12 -or $bytes.Length -gt 1024) {
      throw "Password must contain between 12 and 1024 UTF-8 bytes."
    }
    if ($bytes -contains 0) {
      throw "Password must not contain NUL."
    }
    return [Convert]::ToBase64String($bytes)
  } finally {
    if ($null -ne $bytes) {
      [Array]::Clear($bytes, 0, $bytes.Length)
    }
    $plain = $null
    if ($pointer -ne [IntPtr]::Zero) {
      [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer)
    }
  }
}

function Protect-AdminPath([string]$Path, [bool]$Directory) {
  $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
  $sid = $identity.User
  if ($Directory) {
    $security = New-Object Security.AccessControl.DirectorySecurity
    $inheritance = [Security.AccessControl.InheritanceFlags]"ContainerInherit, ObjectInherit"
    $rule = New-Object Security.AccessControl.FileSystemAccessRule(
      $sid,
      [Security.AccessControl.FileSystemRights]::FullControl,
      $inheritance,
      [Security.AccessControl.PropagationFlags]::None,
      [Security.AccessControl.AccessControlType]::Allow
    )
  } else {
    $security = New-Object Security.AccessControl.FileSecurity
    $rule = New-Object Security.AccessControl.FileSystemAccessRule(
      $sid,
      [Security.AccessControl.FileSystemRights]::FullControl,
      [Security.AccessControl.AccessControlType]::Allow
    )
  }
  $security.SetOwner($sid)
  $security.SetAccessRuleProtection($true, $false)
  $security.AddAccessRule($rule)
  (Get-Item -LiteralPath $Path -Force).SetAccessControl($security)
}

& docker info *> $null
if ($LASTEXITCODE -ne 0) {
  throw "Docker Desktop is not running, or the current user cannot connect to Docker Engine."
}

$secretsDirectory = Join-Path $PSScriptRoot "secrets"
$verifierPath = Join-Path $secretsDirectory "admin-password.phc"
$existing = Test-Path -LiteralPath $verifierPath
if ($Initial -and $existing) {
  throw "The local administrator password verifier already exists."
}
if ((Test-Path -LiteralPath $secretsDirectory) -and
    ((Get-Item -LiteralPath $secretsDirectory -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
  throw "The secrets directory must not be a symlink or reparse point."
}
if ($existing) {
  $existingItem = Get-Item -LiteralPath $verifierPath -Force
  if ($existingItem.PSIsContainer -or
      (($existingItem.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0)) {
    throw "The local administrator password verifier must be a regular non-reparse file."
  }
  $answer = Read-Host "Type ROTATE to replace the local administrator password"
  if ($answer -cne "ROTATE") {
    throw "Password rotation cancelled."
  }
}

$password = Read-Host "New local administrator password" -AsSecureString
$confirmation = Read-Host "Confirm local administrator password" -AsSecureString
$passwordBase64 = Convert-SecureStringToBase64 $password
$confirmationBase64 = Convert-SecureStringToBase64 $confirmation
$password.Dispose()
$confirmation.Dispose()
if (-not [String]::Equals($passwordBase64, $confirmationBase64, [StringComparison]::Ordinal)) {
  $passwordBase64 = $null
  $confirmationBase64 = $null
  throw "Passwords do not match."
}

& docker compose build mvp
if ($LASTEXITCODE -ne 0) {
  throw "docker compose build failed before password hashing."
}
$imageIds = @(& docker compose images -q mvp 2>$null | Where-Object { -not [String]::IsNullOrWhiteSpace($_) })
if ($LASTEXITCODE -ne 0 -or $imageIds.Count -ne 1) {
  throw "Could not resolve exactly one built MVP image."
}
$imageId = [string]$imageIds[0]

$hashOutput = @($passwordBase64 | & docker run --rm -i --pull never --read-only --cap-drop ALL --security-opt no-new-privileges --user 65532:65532 --entrypoint /app/ustc-agentctl $imageId admin hash-password)
$hashExit = $LASTEXITCODE
$passwordBase64 = $null
$confirmationBase64 = $null
if ($hashExit -ne 0) {
  throw "The local password hashing command failed."
}
$hashLines = @($hashOutput | Where-Object { -not [String]::IsNullOrWhiteSpace($_) })
if ($hashLines.Count -ne 1 -or $hashLines[0] -notmatch '^\$argon2id\$v=19\$m=19456,t=2,p=1\$[A-Za-z0-9+/]{22}\$[A-Za-z0-9+/]{43}$') {
  throw "The local password hashing command returned an invalid verifier."
}
$verifier = [string]$hashLines[0]

if (-not (Test-Path -LiteralPath $secretsDirectory)) {
  [IO.Directory]::CreateDirectory($secretsDirectory) | Out-Null
}
Protect-AdminPath $secretsDirectory $true
$tempPath = Join-Path $secretsDirectory (".admin-password.{0}.tmp" -f [Guid]::NewGuid().ToString("N"))
try {
  [IO.File]::WriteAllText($tempPath, $verifier, (New-Object Text.UTF8Encoding($false)))
  Protect-AdminPath $tempPath $false
  if ($existing) {
    [IO.File]::Replace($tempPath, $verifierPath, $null)
  } else {
    [IO.File]::Move($tempPath, $verifierPath)
  }
  Protect-AdminPath $verifierPath $false
} finally {
  if (Test-Path -LiteralPath $tempPath) {
    Remove-Item -LiteralPath $tempPath -Force
  }
  $verifier = $null
  $hashOutput = $null
}

Write-Host "Local deployment access password verifier updated." -ForegroundColor Green
if (-not $Initial) {
  & docker compose up -d --force-recreate --no-deps mvp
  if ($LASTEXITCODE -ne 0) {
    throw "Password was updated, but the MVP container could not be recreated. Run start.cmd for diagnostics."
  }
  Write-Host "Existing browser sessions were invalidated. Run start.cmd to verify readiness." -ForegroundColor Yellow
}

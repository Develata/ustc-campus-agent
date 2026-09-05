# Run by check_windows_launchers.ps1 in Windows PowerShell 5.1.
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if ($PSVersionTable.PSEdition -ne "Desktop" -or $PSVersionTable.PSVersion.Major -ne 5) {
  throw "Windows PowerShell 5.1 required."
}
$root = Join-Path $env:RUNNER_TEMP ("uca-native-port-" + [guid]::NewGuid().ToString("N"))
[void](New-Item -ItemType Directory -Path $root)
$oldPath = $env:PATH
$oldMode = $env:UCA_TEST_PORT_MODE
$oldDone = $env:UCA_TEST_PORT_DONE
try {
  # An actual native process, not a PowerShell function that sets LASTEXITCODE.
  Add-Type -OutputAssembly (Join-Path $root "docker.exe") -OutputType ConsoleApplication -TypeDefinition @'
using System;
using System.IO;
using System.Threading;
public static class NativePortFixture {
  public static int Main(string[] args) {
    if (String.Join(" ", args) != "compose port mvp 8787") return 91;
    string mode = Environment.GetEnvironmentVariable("UCA_TEST_PORT_MODE");
    if (mode == "empty") return 0;
    if (mode == "wildcard") Console.WriteLine("0.0.0.0:8877");
    else if (mode == "range") Console.WriteLine("127.0.0.1:65536");
    else if (mode == "malformed") Console.WriteLine("not-an-address");
    else Console.WriteLine("127.0.0.1:8877");
    Console.Out.Flush();
    Thread.Sleep(200);
    if (mode == "multiple") Console.WriteLine("127.0.0.1:8878");
    File.WriteAllText(Environment.GetEnvironmentVariable("UCA_TEST_PORT_DONE"), "completed");
    return mode == "failure" ? 23 : 0;
  }
}
'@
  $env:PATH = "$root;$oldPath"
  $env:UCA_TEST_PORT_DONE = Join-Path $root "completed.txt"
  $source = [IO.File]::ReadAllText((Join-Path $PSScriptRoot "..\deploy\mvp-compose\start.ps1"))
  $begin = $source.IndexOf('$publishedLines =', [StringComparison]::Ordinal)
  $end = $source.IndexOf('$url =', [StringComparison]::Ordinal)
  if ($begin -lt 0 -or $end -le $begin) { throw "Port capture block missing." }
  $block = [scriptblock]::Create($source.Substring($begin, $end - $begin) + "`nWrite-Output `$port")
  $cases = @(
    @{ Mode = "single"; Error = $null },
    @{ Mode = "multiple"; Error = $null },
    @{ Mode = "failure"; Error = "docker compose port failed." },
    @{ Mode = "empty"; Error = "Unexpected Compose published address." },
    @{ Mode = "wildcard"; Error = "Unexpected Compose published address." },
    @{ Mode = "range"; Error = "Invalid Compose published port." },
    @{ Mode = "malformed"; Error = "Unexpected Compose published address." }
  )
  foreach ($case in $cases) {
    Remove-Item -LiteralPath $env:UCA_TEST_PORT_DONE -Force -ErrorAction SilentlyContinue
    $env:UCA_TEST_PORT_MODE = $case.Mode
    $caught = $null
    $actual = $null
    try { $actual = & $block } catch { $caught = $_.Exception.Message }
    if ($caught -ne $case.Error) { throw "Native port $($case.Mode): expected '$($case.Error)', got '$caught'." }
    if ($null -eq $case.Error -and $actual -ne 8877) { throw "Native port $($case.Mode): wrong port." }
    if ($case.Mode -ne "empty" -and -not (Test-Path -LiteralPath $env:UCA_TEST_PORT_DONE)) {
      throw "Native port $($case.Mode): producer did not finish."
    }
    Write-Output "NATIVE_COMPOSE_PORT_$($case.Mode)=PASS"
  }
} finally {
  $env:PATH = $oldPath
  $env:UCA_TEST_PORT_MODE = $oldMode
  $env:UCA_TEST_PORT_DONE = $oldDone
  Remove-Item -LiteralPath $root -Recurse -Force
}

[CmdletBinding()]
param(
  [string]$ExePath = "",
  [string]$DataDir = "",
  [int]$StartupSeconds = 5
)

$ErrorActionPreference = "Stop"
$dataDirEnvVar = "SELECTOR_DESKTOP_DATA_DIR"

$scriptRoot = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($scriptRoot)) {
  $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if ([string]::IsNullOrWhiteSpace($ExePath)) {
  $ExePath = Join-Path $scriptRoot "..\src-tauri\target\release\selector-desktop.exe"
}

$smokeRoot = Join-Path $scriptRoot "..\..\.codex\release-smoke"
if ([string]::IsNullOrWhiteSpace($DataDir)) {
  $DataDir = Join-Path $smokeRoot "data"
}

$resolvedExe = Resolve-Path -LiteralPath $ExePath -ErrorAction SilentlyContinue
if (-not $resolvedExe) {
  Write-Error "Release executable was not found: $ExePath"
  exit 1
}

$smokeRootPath = [System.IO.Path]::GetFullPath($smokeRoot)
$dataDirPath = [System.IO.Path]::GetFullPath($DataDir)
if (-not $dataDirPath.StartsWith($smokeRootPath, [System.StringComparison]::OrdinalIgnoreCase)) {
  Write-Error "Refusing to use data directory outside release smoke root: $dataDirPath"
  exit 1
}

if (Test-Path -LiteralPath $dataDirPath) {
  Remove-Item -LiteralPath $dataDirPath -Recurse -Force
}
New-Item -ItemType Directory -Path $dataDirPath -Force | Out-Null

$previousDataDir = [Environment]::GetEnvironmentVariable($dataDirEnvVar, "Process")
$process = $null
try {
  [Environment]::SetEnvironmentVariable($dataDirEnvVar, $dataDirPath, "Process")
  $process = Start-Process `
    -FilePath $resolvedExe.Path `
    -WorkingDirectory (Split-Path -Parent $resolvedExe.Path) `
    -WindowStyle Hidden `
    -PassThru
  Start-Sleep -Seconds $StartupSeconds

  if ($process.HasExited) {
    Write-Error "Release executable exited during startup smoke test with code $($process.ExitCode)."
    exit 1
  }

  $databasePath = Join-Path $dataDirPath "selector.db"
  if (-not (Test-Path -LiteralPath $databasePath)) {
    Write-Error "Smoke test database was not created: $databasePath"
    exit 1
  }

  Write-Host "Release smoke test passed."
  Write-Host ("ProcessId {0}; database {1}" -f $process.Id, $databasePath)
} finally {
  if ($process -and -not $process.HasExited) {
    Stop-Process -Id $process.Id -Force
    $process.WaitForExit()
  }
  if ($null -eq $previousDataDir) {
    [Environment]::SetEnvironmentVariable($dataDirEnvVar, $null, "Process")
  } else {
    [Environment]::SetEnvironmentVariable($dataDirEnvVar, $previousDataDir, "Process")
  }
  if (Test-Path -LiteralPath $dataDirPath) {
    Remove-Item -LiteralPath $dataDirPath -Recurse -Force
  }
}

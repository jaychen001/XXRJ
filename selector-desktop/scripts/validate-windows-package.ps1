[CmdletBinding()]
param(
  [string]$BundleDir = ""
)

$ErrorActionPreference = "Stop"

$scriptRoot = $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($scriptRoot)) {
  $scriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
}
if ([string]::IsNullOrWhiteSpace($BundleDir)) {
  $BundleDir = Join-Path $scriptRoot "..\src-tauri\target\release\bundle"
}

$resolvedBundle = Resolve-Path -LiteralPath $BundleDir -ErrorAction SilentlyContinue
if (-not $resolvedBundle) {
  Write-Error "Windows bundle directory was not created: $BundleDir"
  exit 1
}

$artifacts = Get-ChildItem -LiteralPath $resolvedBundle.Path -Recurse -File -Include *.exe, *.msi |
  Sort-Object FullName
if (-not $artifacts) {
  Write-Error "No Windows installer artifacts were found under $($resolvedBundle.Path)."
  exit 1
}

Write-Host "Windows package artifacts:"
foreach ($artifact in $artifacts) {
  $hash = Get-FileHash -LiteralPath $artifact.FullName -Algorithm SHA256
  Write-Host ("{0} | {1} bytes | SHA256 {2}" -f $artifact.FullName, $artifact.Length, $hash.Hash)
}

$blockedFilePatterns = @(
  '(^|\\)\.env($|\.)',
  '\.db$',
  '\.sqlite$',
  'selector\.db$'
)
$files = Get-ChildItem -LiteralPath $resolvedBundle.Path -Recurse -File
$blockedFiles = foreach ($file in $files) {
  foreach ($pattern in $blockedFilePatterns) {
    if ($file.FullName -match $pattern) {
      $file.FullName
      break
    }
  }
}
if ($blockedFiles) {
  Write-Error ("Sensitive runtime files were found in the package: {0}" -f ($blockedFiles -join "; "))
  exit 1
}

$blockedMarkers = @(
  "D:\codex",
  "C:\Users",
  "Users/",
  "SCKJ",
  "sk-ant-",
  "sk-proj-",
  "ANTHROPIC_API_KEY",
  "OPENAI_API_KEY"
)
$markerHits = New-Object System.Collections.Generic.List[string]
foreach ($file in $files) {
  if ($file.Length -gt 80MB) {
    continue
  }
  $bytes = [System.IO.File]::ReadAllBytes($file.FullName)
  $texts = @(
    [System.Text.Encoding]::UTF8.GetString($bytes),
    [System.Text.Encoding]::Unicode.GetString($bytes)
  )
  foreach ($marker in $blockedMarkers) {
    foreach ($text in $texts) {
      if ($text.Contains($marker)) {
        $markerHits.Add("{0} -> {1}" -f $file.FullName, $marker)
        break
      }
    }
  }
}
if ($markerHits.Count -gt 0) {
  Write-Error ("Sensitive markers were found in the package: {0}" -f ($markerHits -join "; "))
  exit 1
}

Write-Host "Package validation passed."

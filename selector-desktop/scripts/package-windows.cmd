@echo off
setlocal

set "SCRIPT_ROOT=%~dp0"
set "BUNDLE_DIR=%SCRIPT_ROOT%..\src-tauri\target\release\bundle"

if exist "%BUNDLE_DIR%" (
  powershell -NoProfile -ExecutionPolicy Bypass -Command "$bundle=(Resolve-Path -LiteralPath $env:BUNDLE_DIR).Path; $releaseRoot=(Resolve-Path -LiteralPath (Join-Path $env:SCRIPT_ROOT '..\src-tauri\target\release')).Path; if (-not $bundle.StartsWith($releaseRoot, [System.StringComparison]::OrdinalIgnoreCase)) { throw \"Refusing to remove unexpected bundle path: $bundle\" }; Remove-Item -LiteralPath $bundle -Recurse -Force"
  if errorlevel 1 exit /b %errorlevel%
)

call "%SCRIPT_ROOT%tauri-build.cmd"
if errorlevel 1 exit /b %errorlevel%

powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_ROOT%validate-windows-package.ps1" -BundleDir "%BUNDLE_DIR%"
if errorlevel 1 exit /b %errorlevel%

exit /b 0

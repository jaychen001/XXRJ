@echo off
setlocal

call "%~dp0tauri-build.cmd"
if errorlevel 1 exit /b %errorlevel%

set "BUNDLE_DIR=%~dp0..\src-tauri\target\release\bundle"
if not exist "%BUNDLE_DIR%" (
  echo Windows bundle directory was not created: %BUNDLE_DIR%
  exit /b 1
)

set FOUND=0
echo Windows package artifacts:
for /r "%BUNDLE_DIR%" %%F in (*.exe *.msi) do (
  set FOUND=1
  echo %%F
)

if "%FOUND%"=="0" (
  echo No Windows installer artifacts were found under %BUNDLE_DIR%.
  exit /b 1
)

exit /b 0

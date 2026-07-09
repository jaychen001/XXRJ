@echo off
setlocal EnableExtensions
pushd "%~dp0.." || exit /b 1
set "EXIT_CODE=0"

set "VCVARS="
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if exist "%VSWHERE%" (
  for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -find "VC\Auxiliary\Build\vcvars64.bat"`) do (
    if not defined VCVARS set "VCVARS=%%I"
  )
)

if not defined VCVARS (
  for %%E in (BuildTools Community Professional Enterprise) do (
    if not defined VCVARS if exist "%ProgramFiles%\Microsoft Visual Studio\2022\%%E\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%ProgramFiles%\Microsoft Visual Studio\2022\%%E\VC\Auxiliary\Build\vcvars64.bat"
    if not defined VCVARS if exist "%ProgramFiles(x86)%\Microsoft Visual Studio\2022\%%E\VC\Auxiliary\Build\vcvars64.bat" set "VCVARS=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\%%E\VC\Auxiliary\Build\vcvars64.bat"
  )
)

if not defined VCVARS (
  echo Visual Studio C++ build environment vcvars64.bat not found.
  echo Install Visual Studio 2022 Build Tools, Community, Professional, or Enterprise with Desktop development with C++.
  set "EXIT_CODE=1"
  goto :end
)
call "%VCVARS%" >nul
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul || (
  echo Cargo was not found after loading the Rust toolchain path.
  set "EXIT_CODE=1"
  goto :end
)
where pnpm.cmd >nul 2>nul || (
  echo pnpm.cmd was not found.
  set "EXIT_CODE=1"
  goto :end
)
call pnpm.cmd exec tauri build
set "EXIT_CODE=%errorlevel%"

:end
popd
exit /b %EXIT_CODE%

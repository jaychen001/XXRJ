@echo off
setlocal
set VCVARS=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
if not exist "%VCVARS%" (
  echo Visual Studio Build Tools vcvars64.bat not found.
  exit /b 1
)
call "%VCVARS%" >nul
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul || (
  echo Cargo was not found after loading the Rust toolchain path.
  exit /b 1
)
call pnpm.cmd exec tauri build

@echo off
REM Build script for compiling tweers-core to WASM (Windows)

setlocal enabledelayedexpansion

echo === TweeRS WASM Build Script ===
echo.

REM Check if wasm-pack is installed
where wasm-pack >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo Error: wasm-pack is not installed
    echo Install it from: https://rustwasm.github.io/wasm-pack/installer/
    exit /b 1
)

REM Default values
set TARGET=web
set PROFILE=release
set OUT_DIR=target\wasm

REM Parse arguments
:parse_args
if "%~1"=="" goto end_parse
if "%~1"=="--target" (
    set TARGET=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--dev" (
    set PROFILE=dev
    shift
    goto parse_args
)
if "%~1"=="--out-dir" (
    set OUT_DIR=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--help" (
    echo Usage: build-wasm.bat [OPTIONS]
    echo.
    echo Options:
    echo   --target TARGET    Target platform: web, nodejs, bundler (default: web)
    echo   --dev              Build in dev mode (default: release)
    echo   --out-dir DIR      Output directory (default: pkg)
    echo   --help             Show this help message
    exit /b 0
)
echo Unknown option: %~1
exit /b 1

:end_parse

echo Building with:
echo   Target: %TARGET%
echo   Profile: %PROFILE%
echo   Output: %OUT_DIR%
echo.

REM Navigate to core crate
cd crates\core

REM Build command - wasm-pack outputs to pkg\ by default
set BUILD_CMD=wasm-pack build --target %TARGET% --features wasm

if "%PROFILE%"=="dev" (
    set BUILD_CMD=%BUILD_CMD% --dev
)
REM Note: wasm-pack builds in release mode by default

echo Running: %BUILD_CMD%
echo.

REM Execute build
%BUILD_CMD%

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Build failed!
    exit /b 1
)

REM Move output to target directory
echo.
echo Moving output to ..\..\%OUT_DIR%
if not exist ..\..\target mkdir ..\..\target
if not exist ..\..\%OUT_DIR% mkdir ..\..\%OUT_DIR%
del /q ..\..\%OUT_DIR%\* 2>nul
xcopy /E /I /Y pkg ..\..\%OUT_DIR%
rmdir /s /q pkg

echo.
echo Build completed successfully!
echo Output directory: %OUT_DIR%
echo.
echo Files generated:
dir /b ..\..\%OUT_DIR%\*.js ..\..\%OUT_DIR%\*.wasm ..\..\%OUT_DIR%\*.ts 2>nul

cd ..\..
endlocal

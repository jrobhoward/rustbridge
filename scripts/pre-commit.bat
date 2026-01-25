@echo off
setlocal enabledelayedexpansion

REM Pre-commit validation script for rustbridge (Windows)
REM
REM Usage:
REM   scripts\pre-commit.bat           Run all checks
REM   scripts\pre-commit.bat --fast    Skip clippy and integration tests

set "FAST_MODE=0"
set "EXIT_CODE=0"

REM Parse arguments
:parse_args
if "%~1"=="" goto :start
if "%~1"=="--fast" (
    set "FAST_MODE=1"
    shift
    goto :parse_args
)
if "%~1"=="--help" (
    echo Usage: %~nx0 [--fast] [--help]
    echo.
    echo Options:
    echo   --fast    Skip clippy and integration tests
    echo   --help    Show this help message
    exit /b 0
)
shift
goto :parse_args

:start
echo.
echo ===================================================
echo rustbridge Pre-Commit Validation (Windows)
echo ===================================================
echo.

REM ============================================================================
REM 1. Check Dependencies
REM ============================================================================
echo ===================================================
echo Checking Dependencies
echo ===================================================

where cargo >nul 2>&1
if errorlevel 1 (
    echo [ERROR] cargo not found. Please install Rust.
    exit /b 1
)
echo [OK] cargo found

where cargo-deny >nul 2>&1
if errorlevel 1 (
    echo [WARN] cargo-deny not found. Installing...
    cargo install cargo-deny
    if errorlevel 1 (
        echo [ERROR] Failed to install cargo-deny
        exit /b 1
    )
)
echo [OK] cargo-deny found

echo.

REM ============================================================================
REM 2. Code Formatting
REM ============================================================================
echo ===================================================
echo Checking Code Formatting (cargo fmt)
echo ===================================================

cargo fmt --all -- --check
if errorlevel 1 (
    echo.
    echo [ERROR] Code formatting check failed!
    echo [INFO] Run 'cargo fmt --all' to fix formatting issues.
    exit /b 1
)
echo [OK] All Rust code is properly formatted
echo.

REM ============================================================================
REM 3. Security and License Checks
REM ============================================================================
echo ===================================================
echo Running Security and License Checks (cargo deny)
echo ===================================================

if not exist "deny.toml" (
    echo [WARN] deny.toml not found. Skipping cargo deny checks.
) else (
    cargo deny check
    if errorlevel 1 (
        echo.
        echo [ERROR] cargo deny checks failed!
        exit /b 1
    )
    echo [OK] All cargo deny checks passed
)
echo.

REM ============================================================================
REM 4. Rust Unit Tests
REM ============================================================================
echo ===================================================
echo Running Rust Unit Tests
echo ===================================================

cargo test --workspace --lib
if errorlevel 1 (
    echo.
    echo [ERROR] Rust unit tests failed!
    exit /b 1
)
echo [OK] All Rust unit tests passed
echo.

REM ============================================================================
REM 5. Rust Integration Tests (skip in fast mode)
REM ============================================================================
if "%FAST_MODE%"=="1" (
    echo [INFO] Skipping Rust integration tests (--fast mode)
    echo.
) else (
    echo ===================================================
    echo Running Rust Integration Tests
    echo ===================================================

    cargo test --workspace --test *
    if errorlevel 1 (
        echo.
        echo [ERROR] Rust integration tests failed!
        exit /b 1
    )
    echo [OK] All Rust integration tests passed
    echo.
)

REM ============================================================================
REM 6. Build Example Plugins
REM ============================================================================
echo ===================================================
echo Building Example Plugins
echo ===================================================

cargo build -p hello-plugin --release
if errorlevel 1 (
    echo.
    echo [ERROR] Failed to build hello-plugin
    exit /b 1
)
echo [OK] hello-plugin built successfully
echo.

REM ============================================================================
REM 7. Java/Kotlin Tests
REM ============================================================================
if exist "rustbridge-java\gradlew.bat" (
    echo ===================================================
    echo Running Java/Kotlin Tests
    echo ===================================================

    pushd rustbridge-java
    call .\gradlew.bat test
    if errorlevel 1 (
        popd
        echo.
        echo [ERROR] Java/Kotlin tests failed!
        exit /b 1
    )
    popd
    echo [OK] All Java/Kotlin tests passed
    echo.
) else (
    echo [INFO] Skipping Java/Kotlin tests (gradlew.bat not found)
    echo.
)

REM ============================================================================
REM 8. C# Tests
REM ============================================================================
where dotnet >nul 2>&1
if errorlevel 1 (
    echo [INFO] Skipping C# tests (dotnet not found)
    echo.
) else (
    if exist "rustbridge-csharp\RustBridge.sln" (
        echo ===================================================
        echo Running C# Tests
        echo ===================================================

        pushd rustbridge-csharp
        dotnet build
        if errorlevel 1 (
            popd
            echo.
            echo [ERROR] C# build failed!
            exit /b 1
        )
        echo [OK] C# build succeeded

        dotnet test
        if errorlevel 1 (
            popd
            echo.
            echo [ERROR] C# tests failed!
            exit /b 1
        )
        popd
        echo [OK] All C# tests passed
        echo.
    ) else (
        echo [INFO] Skipping C# tests (RustBridge.sln not found)
        echo.
    )
)

REM ============================================================================
REM 9. Clippy (skip in fast mode)
REM ============================================================================
if "%FAST_MODE%"=="1" (
    echo [INFO] Skipping clippy checks (--fast mode)
    echo.
) else (
    echo ===================================================
    echo Running Clippy (Lints)
    echo ===================================================

    cargo clippy --workspace --examples --tests -- -D warnings
    if errorlevel 1 (
        echo.
        echo [ERROR] Clippy checks failed!
        exit /b 1
    )
    echo [OK] All clippy checks passed
    echo.
)

REM ============================================================================
REM Summary
REM ============================================================================
echo ===================================================
echo Pre-Commit Validation Complete
echo ===================================================
echo.
echo [OK] All checks passed!
echo.
echo Your code is ready to commit.
echo.

exit /b 0

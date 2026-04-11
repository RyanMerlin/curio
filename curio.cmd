@echo off
setlocal
set MANIFEST=%~dp0curio-rs\Cargo.toml
cargo run --manifest-path "%MANIFEST%" -- %*
exit /b %ERRORLEVEL%

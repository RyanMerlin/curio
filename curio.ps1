$ErrorActionPreference = "Stop"

$manifest = Join-Path $PSScriptRoot "curio-rs\Cargo.toml"
& cargo run --manifest-path $manifest -- @Args
exit $LASTEXITCODE

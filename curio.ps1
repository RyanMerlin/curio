$ErrorActionPreference = "Stop"

$env:CURIO_REPO_ROOT = $PSScriptRoot
$manifest = Join-Path $PSScriptRoot "curio-rs\Cargo.toml"
& cargo run --manifest-path $manifest -- @Args
exit $LASTEXITCODE

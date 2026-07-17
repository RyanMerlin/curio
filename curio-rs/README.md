# Curio

Curio is a Git-native knowledge-base CLI. It keeps a structured Markdown wiki
as the source of truth, moves content through intake, staged, review, and
published lanes, and can sync the published tree to Confluence.

The Rust binary provides deterministic filesystem, Git, taxonomy, quality, and
Confluence operations. Agent workflows make editorial decisions outside the
binary and hand Curio explicit manifests or route files to apply.

## Install

### Versioned release binary

The preferred installation path is a versioned binary from the [v1.0.1 GitHub
Release](https://github.com/RyanMerlin/curio/releases/tag/v1.0.1). If the v1.0.1
release assets are not present yet, wait for that release to be published before
using this path. The release includes the platform archive and `SHA256SUMS`.

When available, the v1.0.1 release assets are named:

- Linux x86_64: `curio-x86_64-unknown-linux-gnu.tar.gz`
- macOS ARM64: `curio-aarch64-apple-darwin.tar.gz`
- Windows x86_64: `curio-x86_64-pc-windows-msvc.zip`

For Linux x86_64, download and verify the archive before installing it:

```sh
VERSION=v1.0.1
ASSET=curio-x86_64-unknown-linux-gnu.tar.gz
BASE_URL="https://github.com/RyanMerlin/curio/releases/download/${VERSION}"
curl -fLO "${BASE_URL}/${ASSET}"
curl -fLO "${BASE_URL}/SHA256SUMS"
grep "  ${ASSET}$" SHA256SUMS | sha256sum --check -
tar -xzf "${ASSET}"
mkdir -p ~/.local/bin
install -m 0755 curio ~/.local/bin/curio
~/.local/bin/curio --version
```

For macOS ARM64, use `curio-aarch64-apple-darwin.tar.gz` as `ASSET` and use
`shasum -a 256 -c -` instead of `sha256sum --check -`. For Windows x86_64,
use PowerShell to download, verify, and extract the archive:

```powershell
$Version = "v1.0.1"
$Asset = "curio-x86_64-pc-windows-msvc.zip"
$BaseUrl = "https://github.com/RyanMerlin/curio/releases/download/$Version"
Invoke-WebRequest "$BaseUrl/$Asset" -OutFile $Asset
Invoke-WebRequest "$BaseUrl/SHA256SUMS" -OutFile SHA256SUMS
$Expected = (Select-String -Path SHA256SUMS -Pattern "  $Asset$").Line.Split()[0]
$Actual = (Get-FileHash $Asset -Algorithm SHA256).Hash.ToLowerInvariant()
if ($Actual -ne $Expected) { throw "SHA-256 checksum mismatch" }
Expand-Archive $Asset -DestinationPath . -Force
.\curio.exe --version
```

The `curio-km` package is not yet published to crates.io, so
`cargo install curio-km` is not a currently supported installation command.

### Build from source

To build the latest source instead:

```sh
git clone https://github.com/RyanMerlin/curio.git
cd curio/curio-rs
cargo build --release --bin curio
./target/release/curio --version
```

## Quick Start

Create a knowledge-base store, then use `--kb-dir` to target it explicitly:

```sh
curio init-kb --path ./my-kb --name demo
curio --kb-dir ./my-kb status
curio --kb-dir ./my-kb intake --file ./notes.md
```

Curio's intake and routing workflow is two-phase. First generate a manifest for
an agent to inspect and route; then apply the resulting decisions:

```sh
curio --kb-dir ./my-kb process --prepare > /tmp/curio-routing.json
# Have an agent turn the manifest into /tmp/curio-routes.json.
curio --kb-dir ./my-kb process --route-file /tmp/curio-routes.json
curio --kb-dir ./my-kb review --lane staged
curio --kb-dir ./my-kb publish <slug>
curio --kb-dir ./my-kb sync --dry-run
```

Pages with ambiguous taxonomy or incomplete evidence should remain in
`review/`. Use `resolve <slug>` after review to move an item to `staged/`, then
publish it only when it is ready.

## Workspaces

For multiple knowledge bases, register directories in the local workspace
registry and select one by name:

```sh
curio workspace add --name acme --path /path/to/acme-kb
curio --workspace acme status
curio --workspace acme sync
```

Each KB has its own taxonomy, Markdown content, Git history, and Confluence
configuration. Keep credentials in the KB's environment/configuration rather
than in source files.

## Useful Commands

- `curio init` creates or repairs a `wiki/` scaffold in an existing KB.
- `curio status` reports intake, staged, review, and published counts.
- `curio doctor` scans the published tree for structural and quality issues.
- `curio review` lists items awaiting review or resolution.
- `curio search` searches the local wiki registry and page content.
- `curio tree` synchronizes published directories with the taxonomy.
- `curio reindex` rebuilds co-located Markdown indexes.
- `curio lint` scans for contradictions, stale claims, and orphaned links.
- `curio feedback` applies Confluence review signals to the local wiki.
- `curio heal --prepare` emits a confidence-gated repair manifest.
- `curio agent ...` manages provider and harness readiness commands.

Most commands accept `--json` for machine-readable output and `--dry-run`
where a preview is supported. Run `curio --help` or `curio <command> --help`
for the complete command and option reference.

## Documentation

- [Human CLI reference](https://github.com/RyanMerlin/curio/blob/main/curio-rs/docs/cli_for_humans.md)
- [Agent CLI reference](https://github.com/RyanMerlin/curio/blob/main/curio-rs/docs/cli_for_agents.md)
- [Curio repository and architecture](https://github.com/RyanMerlin/curio)
- [Issue tracker](https://github.com/RyanMerlin/curio/issues)
- [Apache License 2.0](https://github.com/RyanMerlin/curio/blob/main/LICENSE)

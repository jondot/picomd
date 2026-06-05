# Releasing picomd

picomd ships prebuilt binaries through three channels:

- GitHub Releases: a signed `tar.gz` per target (the curl installer pulls these).
- npm: the `picomd` package plus per-platform binary packages.
- crates.io: `cargo install picomd`.

## Cutting a release

```sh
./scripts/release.sh 0.2.0
```

This requires a clean tree on `main`. It bumps `version` in `Cargo.toml`,
regenerates `Cargo.lock`, commits `release v0.2.0`, tags `v0.2.0`, and pushes.
Pushing the `v*` tag triggers `.github/workflows/release.yml`.

## What the Release workflow does

1. **build** (matrix, one runner per target):

   | Target | Runner | Notes |
   |--------|--------|-------|
   | `x86_64-apple-darwin` | `macos-latest` | cross-compiled on Apple silicon (macOS frameworks are universal, so arm64 → x86_64 links cleanly — avoids the scarce/slow `macos-13` Intel runners) |
   | `aarch64-apple-darwin` | `macos-latest` | native Apple silicon |
   | `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | installs webkit2gtk/gtk |

   Each produces `picomd-<target>.tar.gz`. These are the desktop targets; Windows
   and arm64-Linux are intentionally excluded (see below).

2. **release** — signs every `tar.gz` with cosign (keyless / sigstore, needs
   `id-token: write`) producing a `.bundle`, then creates the GitHub Release with
   auto-generated notes and uploads the archives + bundles.

3. **publish-npm** — extracts each binary into its `npm/picomd-<platform>/`
   package, sets all versions to the tag, pins the main package's
   `optionalDependencies`, and publishes the platform packages then the main
   `picomd` package.

4. **publish-crates** — runs `cargo publish` (so `cargo install picomd` works).
   The build verifies the bin, so the job installs webkit2gtk/gtk first.

## Required repository secrets & variables

Both publish jobs are **opt-in**, each gated by a repository variable so a release
can build, sign, and publish to GitHub Releases without any token:

| Channel | Secret | Enable variable |
|---------|--------|-----------------|
| npm | `NPM_TOKEN` (automation token with publish rights to `picomd` + the three `picomd-<platform>` packages) | `PUBLISH_NPM=true` |
| crates.io | `CARGO_REGISTRY_TOKEN` | `PUBLISH_CRATES=true` |

`GITHUB_TOKEN` is provided automatically; the workflow requests `contents:write`
and `id-token:write` (the latter for cosign keyless signing).

`cargo publish` only succeeds for a version not already on crates.io, so the
crates job publishes exactly once per new tag.

## Not included (and why)

- **Windows.** Out of scope for now. picomd's *code* supports Windows (WebView2),
  but a Windows release artifact needs its own packaging (`.zip` + `picomd.exe`, a
  `win32-x64` npm package, and a PowerShell installer). Add it deliberately later.
- **arm64 Linux.** Excluded — desktop release targets are macOS (Intel + Apple
  silicon) and Linux x86_64.
- **Apple notarization.** Artifacts are cosign-signed (generic), not Apple
  notarized; notarization needs developer-ID certificates and is out of scope.

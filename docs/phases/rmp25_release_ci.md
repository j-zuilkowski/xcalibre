# RMP-25 — Release Infrastructure & CI/CD

> Prerequisite: rmp22b complete (all feature phases done).
> Status: ✅ complete (2026-05-12)
> This phase adds session instructions, CI, and the cross-platform release pipeline.

## Status

| Task | Title | Status |
|------|-------|--------|
| R25-T01 | Add CLAUDE.md — Claude Code session instructions | ✅ |
| R25-T02 | Add AGENTS.md — Codex session instructions | ✅ |
| R25-T03 | Add CI workflow — Rust + frontend checks on push/PR | ✅ |
| R25-T04 | Add release workflow — cross-platform installer build on tag push | ✅ |
| R25-T05 | Align all 8 crate versions to 1.0.0 | ✅ |
| R25-T06 | Remove chmlib — rewrite CHM extractors to pure Rust | ✅ |
| R25-T07 | Fix tauri.conf.json — remove placeholder updater pubkey | ✅ |
| R25-T08 | Fix release.yml — add `permissions: contents: write` | ✅ |
| R25-T09 | Fix Cargo.lock — commit regenerated lock without chmlib | ✅ |
| R25-T10 | Fix bundle targets — `"all"` in tauri.conf.json, `--bundles deb` for Linux | ✅ |
| R25-T11 | Fix cache key — remove `matrix.args` to avoid commas | ✅ |
| R25-T12 | Fix v1.0.0 tag — force-move to HEAD after all fixes landed | ✅ |

---

## R25-T01 — CLAUDE.md

**File:** `CLAUDE.md` (new, xcalibre root)

Session instructions for Claude Code following the same pattern as `xcalibre-server` and Merlin.
Includes: stack overview, key paths, non-negotiable constraints, git commit protocol, 8-file
versioning table, git state checks, build commands, and guides for adding formats/providers/plugins.

Contains `**Current version: 1.0.0** (tag \`v1.0.0\`)` callout.

---

## R25-T02 — AGENTS.md

**File:** `AGENTS.md` (rewritten from stale early-phase content)

Trimmed Codex session instructions. Same versioning table and callout as CLAUDE.md. Includes
Codex execution instructions (`DO NOT output code as text. Use your tools…`).

---

## R25-T03 — CI workflow

**File:** `.github/workflows/ci.yml` (new)

Runs on every push to `main` and every PR:
- `rust` job (ubuntu-22.04): `cargo check`, `cargo clippy -D warnings`, `cargo test --workspace`
- `frontend` job (ubuntu-22.04): `npm install`, `tsc --noEmit`, `npm test --watchAll=false`

---

## R25-T04 — Release workflow

**File:** `.github/workflows/release.yml` (new)

Builds installers for all four targets on tag push (`v*`) or `workflow_dispatch`:

| Platform | Runner | Args | Artifact |
|----------|--------|------|----------|
| macOS arm64 | macos-latest | `--target aarch64-apple-darwin` | `.dmg` |
| macOS x86_64 | macos-latest | `--target x86_64-apple-darwin` | `.dmg` |
| Windows | windows-latest | — | `.msi`, `.exe` (NSIS) |
| Linux | ubuntu-22.04 | `--bundles deb` | `.deb` |

Uses `tauri-apps/tauri-action@v0` with `permissions: contents: write` and
`includeUpdaterJson: false`. No signing keys required.

---

## R25-T05 — Version alignment

**Files:** `xcalibre-ai/Cargo.toml`, `xcalibre-epub/Cargo.toml`

Both had drifted to `0.1.0` while the rest of the workspace was `1.0.0`. Bumped both to `1.0.0`.

All 8 version sources now carry `1.0.0`:

| File | Field |
|------|-------|
| `src-tauri/Cargo.toml` | `version = "1.0.0"` |
| `src-tauri/tauri.conf.json` | `"version": "1.0.0"` |
| `ui/package.json` | `"version": "1.0.0"` |
| `processing/Cargo.toml` | `version = "1.0.0"` |
| `xcalibre-ai/Cargo.toml` | `version = "1.0.0"` |
| `xcalibre-epub/Cargo.toml` | `version = "1.0.0"` |
| `xcalibre-plugin-sdk/Cargo.toml` | `version = "1.0.0"` |
| `api/Cargo.toml` | `version = "1.0.0"` |

---

## R25-T06 — Remove chmlib

**Files:** `processing/Cargo.toml`, `processing/src/text/chm.rs`, `processing/src/metadata/chm.rs`

`chmlib-sys 1.0.0` has a build-script bug on Windows: `cc-rs` generates a UNC path (`\\chm_lib.c`)
that MSVC rejects with C1083. The `cfg(not(target_os = "windows"))` gate was not reliably
excluding it due to Cargo.lock + Actions cache interaction.

Fix: removed `chmlib` from `processing/Cargo.toml` entirely. Rewrote both extractors:

- `processing/src/text/chm.rs` — pure-Rust raw-bytes HTML scanner; strips tags, falls back to
  ASCII word recovery. No native library required.
- `processing/src/metadata/chm.rs` — filename-based title recovery via `recover_title()`.

`pub mod chm` restored as unconditional in both `text/mod.rs` and `metadata/mod.rs`.
Single unconditional `DetectedFormat::Chm` dispatch arms in `pipeline/text.rs` and
`pipeline/metadata.rs`.

---

## R25-T07 — Remove updater pubkey placeholder

**File:** `src-tauri/tauri.conf.json`

Removed entire `plugins.updater` section that contained `"pubkey": "LOCAL_TEST_PUBLIC_KEY"`.
When a pubkey is present Tauri requires `.sig` bundle signatures; `tauri-action` then lists
`.sig` files as expected artifacts that can't exist without `TAURI_SIGNING_PRIVATE_KEY`, causing
"No artifacts were found" on every Linux/Windows build.

---

## R25-T08 — Release job permissions

**File:** `.github/workflows/release.yml`

`workflow_dispatch` triggers get read-only `GITHUB_TOKEN` by default. The macOS build succeeded
but upload to the GitHub release returned 403. Fix: added `permissions: contents: write` to the
build job.

---

## R25-T09 — Cargo.lock regeneration

**File:** `Cargo.lock`

After removing `chmlib` from `processing/Cargo.toml` in T06, the regenerated `Cargo.lock`
(with `chmlib` and `chmlib-sys` entries removed) was not committed. The release workflow checks
out the tag, so CI was still resolving and attempting to build `chmlib-sys`. Committed the
updated lock file explicitly.

---

## R25-T10 — Bundle targets

**Files:** `src-tauri/tauri.conf.json`, `.github/workflows/release.yml`

`bundle.targets` was `["dmg", "app", "msi", "nsis"]` — no Linux types. On Ubuntu the build
succeeded but Tauri produced zero bundle artifacts, causing "No artifacts were found".

- Changed `targets` to `"all"` in `tauri.conf.json` (builds platform-appropriate bundles).
- Set Linux matrix `args: "--bundles deb"` — AppImage bundler downloads `linuxdeploy` from
  GitHub CDN at runtime and intermittently fails with HTTP 500; `.deb` builds entirely locally.

---

## R25-T11 — Release.yml cache key

**File:** `.github/workflows/release.yml`

Cache key was `${{ runner.os }}-${{ matrix.args }}-cargo-...`. When `matrix.args` is
`"--bundles deb,appimage"`, the key contains commas which GitHub Actions rejects immediately
with "Key Validation Error". Changed to `${{ runner.os }}-cargo-...`.

---

## R25-T12 — v1.0.0 tag

**Git operation**

The `v1.0.0` tag was created at commit `1d7cbd0` — before any of the release fixes landed on
`main`. The release workflow does `ref: ${{ github.event.inputs.tag || github.ref }}` so every
workflow_dispatch with `tag=v1.0.0` checked out the old pre-fix commit, re-introducing all the
build failures regardless of what was on `main`.

Fix: `git tag -f v1.0.0 HEAD && git push origin v1.0.0 --force` after each fix commit. The
final tag points to `7355d59` (all twelve fixes included).

# xcalibre — Claude Session Instructions

Read this file at the start of every session. These rules apply to all work in this project.

---

## Project

Cross-platform desktop ebook library manager — Tauri 2 shell, Rust backend, React/TypeScript/Tailwind frontend.

- Full design: `docs/ARCHITECTURE.md` and `docs/DEVELOPER_GUIDE.md`
- Plugin ABI reference: `xcalibre-plugin-sdk/src/lib.rs`
- Working directory: `~/Documents/localProject/xcalibre`

## Stack

- Shell: Tauri 2 (`src-tauri/`)
- Backend: Rust — `processing/`, `xcalibre-ai/`, `xcalibre-epub/`, `xcalibre-plugin-sdk/`, `api/`
- Frontend: React 18 + TypeScript + Tailwind (`ui/`)
- Database: SQLite with FTS5 (file: `{app_data}/xcalibre/library.db`)
- CI/CD: GitHub Actions — `.github/workflows/release.yml`

## Key Paths

- `src-tauri/src/commands.rs` — all Tauri commands (IPC boundary between UI and Rust)
- `processing/src/` — ingest pipeline, DB queries, format detection, converters, plugin loader
- `xcalibre-ai/src/` — AI provider abstraction (Ollama, OpenAI, Gemini, LM Studio, OpenRouter)
- `xcalibre-epub/src/` — EPUB read/write/edit library
- `xcalibre-plugin-sdk/src/lib.rs` — stable C ABI for third-party plugins
- `api/src/` — xcalibre-server HTTP client
- `ui/src/` — React frontend
- `docs/` — ARCHITECTURE.md, DEVELOPER_GUIDE.md, ROADMAP.md, USER_GUIDE.md

---

## Non-Negotiable Constraints

- `cargo test --workspace` must pass at zero failures before any commit
- `cargo clippy -- -D warnings` must pass at zero warnings
- No `unwrap()` in production code — use `?` and proper error types
- Plugin ABI (`xcalibre-plugin-sdk`) is stable — never break `extern "C"` signatures without a major version bump
- Path traversal prevention on all file-serving routes
- API keys are never sent to the WebView — use `AiConfigPublic { has_api_key: bool }`

---

## Git Commit Protocol

Commit after every meaningful unit of work:

```bash
cd ~/Documents/localProject/xcalibre
git add <specific files — never git add -A>
git commit -m "<Description>"
```

Never skip a commit. Never amend a prior commit when adding the next change — always create a new commit.

---

## Versioning

**Sources of truth (all eight must carry the same version string):**

| File | Field |
|---|---|
| `src-tauri/Cargo.toml` | `version = "X.Y.Z"` |
| `src-tauri/tauri.conf.json` | `"version": "X.Y.Z"` |
| `ui/package.json` | `"version": "X.Y.Z"` |
| `processing/Cargo.toml` | `version = "X.Y.Z"` |
| `xcalibre-ai/Cargo.toml` | `version = "X.Y.Z"` |
| `xcalibre-epub/Cargo.toml` | `version = "X.Y.Z"` |
| `xcalibre-plugin-sdk/Cargo.toml` | `version = "X.Y.Z"` |
| `api/Cargo.toml` | `version = "X.Y.Z"` |

Full versioning policy (increment rules, release procedure) is in
`docs/ARCHITECTURE.md` → **Versioning Policy** section. Follow that document exactly.

Pushing a tag triggers `.github/workflows/release.yml`, which builds installers for all four
platforms (macOS arm64, macOS x86_64, Windows, Linux) and creates the GitHub release automatically.

**Current version: 1.0.0** (tag `v1.0.0`)

---

## Git State Checks

At the start of any session, check for in-progress git operations before doing any work:

```bash
git status   # look for "rebase in progress", "revert in progress", "merge in progress"
git log --oneline -5
git tag --list | sort -V | tail -5
```

If an in-progress operation exists, surface it to the user and ask whether to abort or continue before proceeding. Never commit or tag over an unresolved git state.

---

## Building Locally

```bash
# Install UI dependencies (first time or after package.json changes)
cd ui && npm install && cd ..

# Run all tests
cargo test --workspace

# Lint
cargo clippy -- -D warnings

# Dev build (opens Tauri window)
cargo tauri dev

# Release build (produces installers in src-tauri/target/release/bundle/)
cargo tauri build
```

Prerequisites: Rust 1.78+, Node.js 20+, Tauri CLI 2.x.

---

## Adding a New Format

See `docs/DEVELOPER_GUIDE.md` → **Adding a New Format** (10-step guide).

## Adding a New AI Provider

See `docs/DEVELOPER_GUIDE.md` → **Adding a New AI Provider** (5-step guide).

## Plugin Development

See `docs/DEVELOPER_GUIDE.md` → **Plugin Development** (full walkthrough with Rust example).

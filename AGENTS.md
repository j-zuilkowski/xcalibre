# xcalibre — Codex Session Instructions

Read this file at the start of every session. These rules apply to all work in this project.

---

## Project

Cross-platform desktop ebook library manager — Tauri 2 shell, Rust backend, React/TypeScript/Tailwind frontend.

Full design: `docs/ARCHITECTURE.md` and `docs/DEVELOPER_GUIDE.md`
Plugin ABI reference: `xcalibre-plugin-sdk/src/lib.rs`
Working directory: `~/Documents/localProject/xcalibre`

## Stack

- Shell: Tauri 2 (`src-tauri/`)
- Backend: Rust — `processing/`, `xcalibre-ai/`, `xcalibre-epub/`, `xcalibre-plugin-sdk/`, `api/`
- Frontend: React 18 + TypeScript + Tailwind (`ui/`)
- Database: SQLite with FTS5 (file: `{app_data}/xcalibre/library.db`)
- CI/CD: GitHub Actions — `.github/workflows/release.yml`

## Key Paths

- `src-tauri/src/commands.rs` — all Tauri IPC commands (security boundary between UI and Rust)
- `processing/src/` — ingest pipeline, DB queries, format detection, converters, plugin loader
- `xcalibre-ai/src/` — AI provider abstraction (Ollama, OpenAI, Gemini, LM Studio, OpenRouter)
- `xcalibre-epub/src/` — EPUB read/write/edit library
- `xcalibre-plugin-sdk/src/lib.rs` — stable C ABI for third-party plugins
- `api/src/` — xcalibre-server HTTP client
- `ui/src/` — React frontend

## Non-Negotiable Constraints

- `cargo test --workspace` must pass at zero failures before any commit
- `cargo clippy -- -D warnings` must pass at zero warnings
- `cargo audit` must pass at zero vulnerabilities
- No `unwrap()` in production code — use `?` and proper error types
- Plugin ABI (`xcalibre-plugin-sdk`) is stable — never break `extern "C"` signatures without a major version bump
- Magic-byte-first format detection — never rely on file extension alone
- API keys are never sent to the WebView — use `AiConfigPublic { has_api_key: bool }`
- Path traversal prevention on all file-serving routes

## Running a Task

**DO NOT output code as text. Use your tools to write files and run commands.**

1. For every `Write \`path\` with this exact content:` instruction → call your **write_file** tool.
2. For every `In \`path\`, replace X with Y` instruction → call **read_file**, make the replacement, call **write_file**.
3. For every command in the `Then run:` block → call your **shell** tool to execute it.
4. If any shell command fails → fix the code, rewrite the file, re-run.

Do not print code. Do not ask for confirmation. Do not explain. Just execute.

## Dev Commands

```bash
cargo build --workspace              # compile check
cargo test --workspace               # run all tests
cargo test --workspace -- <filter>   # run specific test
cargo clippy --workspace -- -D warnings  # lint
cargo audit                          # CVE check
cd ui && npm install && npm test     # frontend tests
```

Verification block (run after every stage before committing):
```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Versioning

**Current version: 1.0.0** (tag `v1.0.0`)

All eight files below must carry the same version string at every commit:

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

Full versioning policy and release procedure: `docs/ARCHITECTURE.md` → **Versioning Policy**.

Pushing a tag triggers `.github/workflows/release.yml`, which builds installers for all four
platforms (macOS arm64, macOS x86_64, Windows, Linux) and creates the GitHub release automatically.

## Code Style

- Rust edition 2021
- No `unwrap()` in production code — use `?` and proper error types
- All Tauri commands return `Result<T, String>` — propagate errors to the frontend
- Log with `tracing` macros — never `eprintln!` in library code
- Tests use an in-memory SQLite pool — never share state between tests

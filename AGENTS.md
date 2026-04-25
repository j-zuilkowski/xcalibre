# xCalibre — Codex Context

## Project
Rust + Tauri v2 standalone ebook reader and asset processing engine.
Processes raw ebook files locally; pushes library-ready assets to xcalibre-server via API.
Full architecture: docs/ARCHITECTURE.md
Language-agnostic design reference: docs/AGNOSTIC.md
Phase build prompts: docs/phase1_commands.md – docs/phase10_commands.md

## Stack
- Processing engine: Rust 2021, Tokio, sqlx 0.8, SQLite (local job DB)
- CLI: Clap 4 (derive)
- Error types: thiserror
- Hashing: sha2 0.10
- HTTP client: reqwest (api/ crate)
- Auth: keyring 2 (OS keychain for service token)
- Image processing: image 0.25 (Phase 1 Stage 4)
- Desktop UI: Tauri v2 + React + Vite + Zustand + Tailwind (Phase 1 Stage 7)
- xcalibre-server connection: optional — xCalibre is fully functional offline

## Key Paths
- `processing/src/` — core build/conversion engine
  - `main.rs` — CLI entry point (clap)
  - `pipeline/` — 5-stage pipeline (ingest, metadata, text, cover, push stubs)
  - `plugins/` — format handlers (epub, pdf, mobi, cbz, txt)
  - `db/` — SQLite job DB (sqlx migrations)
  - `utils/` — SHA-256 hashing, text normalisation
  - `error.rs` — ProcessingError enum (thiserror)
- `api/src/` — xcalibre-server API client (push + read endpoints)
- `processing/src/db/migrations/` — SQL migration files (0001_jobs.sql complete)
- `processing/tests/` — integration tests (in-memory SQLite)

## Non-Negotiable Constraints
- TDD: tests written first, implementation makes them pass
- Frontend (React/Tauri UI): every component introduced in a phase must have a corresponding `.test.tsx` / `.test.ts` written in the same phase. Render real components via React Testing Library. Mock only at the Tauri IPC boundary using `mockInvoke()` from `ui/src/test/setup.ts`. Never mock Zustand stores — use real stores reset in `beforeEach`.
- Phase file fidelity: any change made during a build (bug fix, Tauri command signature change, component refactor, store shape change) must be reflected back in the corresponding phase file before committing. The phase files are the source of truth — a future clean build from them must produce the same working codebase.
- No `unwrap()` in library code — use `?` and `ProcessingError`
- `cargo clippy --workspace -- -D warnings` must pass at zero warnings
- `cargo audit` must pass at zero vulnerabilities
- All enrichment network calls: 10s timeout, silent fallback, never block processing
- All xcalibre-server read API calls: 5s timeout, return empty, never surface errors to user
- xCalibre must be fully functional with no xcalibre-server connection
- Service token stored in OS keychain via `keyring` crate — never in config files or logs
- Magic-byte-first format detection — never rely on file extension alone
- LLM suggestions are never auto-applied — user always confirms before any field is written

## Running a Task

When given a task block (pasted directly or referenced by ID):

**DO NOT output code as text. Use your tools to write files and run commands.**

1. For every `Write \`path\` with this exact content:` instruction →
   call your **write_file** tool with that path and that exact content.
2. For every `Add the following to the end of \`path\`.` instruction →
   call **read_file**, append the new content, call **write_file**.
3. For every `In \`path\`, replace X with Y` instruction →
   call **read_file**, make the replacement, call **write_file**.
4. For every `In \`path\`, add X after Y` instruction →
   call **read_file**, insert the content, call **write_file**.
5. For every command in the `Then run:` block →
   call your **shell** / **terminal** tool to execute it.
6. If any shell command fails → fix the code, rewrite the file, re-run.

Do not print code. Do not ask for confirmation. Do not explain. Just execute.

## Skills Workflow
- After every task: verify & commit block must pass before moving to next
- On failing verify: fix the error, do not skip
- Start of any new session: read the status tables in the phase docs to find the next ⬜ task
- Before any enrichment or xcalibre-server API work: run a security review

## Dev Commands
```
cargo build --workspace              # compile check
cargo test --workspace               # run all tests
cargo test --workspace -- <filter>   # run specific test
cargo clippy --workspace -- -D warnings  # lint
cargo audit                          # CVE check
```

Verification block (run after every stage before committing):
```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

## Code Style
- Rust edition 2021
- No `unwrap()` in production code — use `?` and `ProcessingError`
- `ProcessingError` (thiserror) is the canonical error type for `processing/`
- `anyhow::Result` is acceptable in tests and `fn main()`
- Tests: in-memory SQLite (`sqlite::memory:`) with migrations applied fresh per test; never share pool between tests
- Log with `tracing` macros — never `eprintln!` in library code
- All datetime values stored as ISO 8601 TEXT in SQLite; use `chrono::Utc::now()`
- Format names in DB: uppercase ("EPUB", "PDF", "MOBI", "AZW3", "CBZ", "CBR", "TXT")
- Job status in DB: uppercase ("PENDING", "READY_TO_PUSH", "PUSHING", "RETRYING", "COMPLETED", "FAILED")
- Each pipeline stage writes output to local SQLite before handing off — crash safety is a design goal

# Phase 9 — Polish + Distribution

> HOW TO USE: For every "Write `path`" line → call write_file with that path and content.
> For every "Then run:" block → call your shell tool for each command.
> DO NOT print code as output. Write it to disk using your tools.
> Prerequisite: Phase 8 complete and all tests green.
> Run the release checklist (P9-T10) before tagging v1.0.0.
> Status: ⏳ code complete · release checklist pending

## Status

| Task | Title | Status |
|------|-------|--------|
| P9-T01 | Settings UI | ✅ |
| P9-T02 | Error boundary + user-facing error display | ✅ |
| P9-T03 | Loading states + skeleton screens | ✅ |
| P9-T04 | Keyboard shortcuts (global + reader) | ✅ |
| P9-T05 | Tauri auto-updater integration | ✅ |
| P9-T06 | macOS .dmg build + code signing | ✅ |
| P9-T07 | Windows .msi build | ✅ |
| P9-T08 | Performance profiling — DB queries | ✅ |
| P9-T09 | Performance profiling — pipeline throughput | ✅ |
| P9-T10 | Release checklist + tag v1.0.0 | ⬜ |

---

## P9-T01

Write `ui/src/components/SettingsModal.tsx` with this exact content:
```tsx
import { useSettingsStore } from "../store/settingsStore"

interface Props { onClose: () => void }

export function SettingsModal({ onClose }: Props) {
  const { fontSize, theme, setFontSize, setTheme } = useSettingsStore()

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-md p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg font-semibold dark:text-white">Settings</h2>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600 text-xl leading-none">×</button>
        </div>

        {/* Theme */}
        <div className="mb-5">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Theme
          </label>
          <div className="flex gap-2">
            {(["light", "dark", "sepia"] as const).map((t) => (
              <button
                key={t}
                onClick={() => setTheme(t)}
                className={`flex-1 py-1.5 rounded-lg text-sm border capitalize transition-colors
                  ${theme === t
                    ? "border-blue-500 bg-blue-50 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300"
                    : "border-gray-200 dark:border-gray-600 dark:text-gray-300 hover:border-gray-300"}`}
              >
                {t}
              </button>
            ))}
          </div>
        </div>

        {/* Font size */}
        <div className="mb-5">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Reader Font Size: <span className="font-mono">{fontSize}px</span>
          </label>
          <input
            type="range" min={12} max={32} step={1} value={fontSize}
            onChange={(e) => setFontSize(Number(e.target.value))}
            className="w-full accent-blue-500"
          />
          <div className="flex justify-between text-xs text-gray-400 mt-1">
            <span>12px</span><span>32px</span>
          </div>
        </div>

        <div className="flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 text-sm font-medium"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/SettingsModal.tsx
git commit -m "P9-T01: Settings UI — theme + font size"
```

---

## P9-T02

Write `ui/src/components/ErrorBoundary.tsx` with this exact content:
```tsx
import { Component, ErrorInfo, ReactNode } from "react"

interface Props  { children: ReactNode }
interface State  { error: Error | null }

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null }

  static getDerivedStateFromError(error: Error): State {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("xCalibre unhandled error:", error, info)
  }

  render() {
    if (this.state.error) {
      return (
        <div className="min-h-screen flex flex-col items-center justify-center p-8 bg-gray-50 dark:bg-gray-900">
          <div className="max-w-lg w-full bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6">
            <h1 className="text-xl font-semibold text-red-600 mb-2">Something went wrong</h1>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              {this.state.error.message}
            </p>
            <pre className="text-xs bg-gray-100 dark:bg-gray-700 rounded p-3 overflow-auto max-h-40 dark:text-gray-300">
              {this.state.error.stack}
            </pre>
            <button
              onClick={() => this.setState({ error: null })}
              className="mt-4 px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 text-sm"
            >
              Try again
            </button>
          </div>
        </div>
      )
    }
    return this.props.children
  }
}
```

In `ui/src/main.tsx`, wrap `<App />` with `<ErrorBoundary>`:
```tsx
import { ErrorBoundary } from "./components/ErrorBoundary"

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
)
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/ErrorBoundary.tsx ui/src/main.tsx
git commit -m "P9-T02: ErrorBoundary component"
```

---

## P9-T03

Write `ui/src/components/Skeleton.tsx` with this exact content:
```tsx
interface Props {
  className?: string
  count?: number
}

/** Animated loading placeholder bar. */
export function Skeleton({ className = "", count = 1 }: Props) {
  return (
    <>
      {Array.from({ length: count }).map((_, i) => (
        <div
          key={i}
          className={`animate-pulse bg-gray-200 dark:bg-gray-700 rounded ${className}`}
        />
      ))}
    </>
  )
}

/** Full library grid skeleton shown while books load. */
export function LibrarySkeleton() {
  return (
    <div className="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4 p-6">
      {Array.from({ length: 10 }).map((_, i) => (
        <div key={i} className="flex flex-col rounded-lg overflow-hidden shadow bg-white dark:bg-gray-800">
          <div className="h-48 animate-pulse bg-gray-200 dark:bg-gray-700" />
          <div className="p-2 flex flex-col gap-1.5">
            <Skeleton className="h-3 w-3/4" />
            <Skeleton className="h-2.5 w-1/2" />
          </div>
        </div>
      ))}
    </div>
  )
}
```

Update `ui/src/components/LibraryView.tsx` to use `LibrarySkeleton` instead of the plain loading text:
```tsx
import { LibrarySkeleton } from "./Skeleton"
// Replace: if (loading) return <div className="p-8 text-gray-500">Loading…</div>
// With:    if (loading) return <LibrarySkeleton />
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/components/Skeleton.tsx ui/src/components/LibraryView.tsx
git commit -m "P9-T03: skeleton loading screens for library grid"
```

---

## P9-T04

Write `ui/src/hooks/useKeyboard.ts` with this exact content:
```ts
import { useEffect } from "react"

type KeyMap = Record<string, () => void>

/**
 * Bind keyboard shortcuts globally.
 * Keys use the format: "ctrl+k", "meta+f", "escape", "arrowleft"
 * Call inside a component; bindings are removed on unmount.
 */
export function useKeyboard(keyMap: KeyMap) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const key = [
        e.ctrlKey  ? "ctrl"  : null,
        e.metaKey  ? "meta"  : null,
        e.altKey   ? "alt"   : null,
        e.shiftKey ? "shift" : null,
        e.key.toLowerCase(),
      ]
        .filter(Boolean)
        .join("+")

      const action = keyMap[key]
      if (action) {
        e.preventDefault()
        action()
      }
    }

    window.addEventListener("keydown", handler)
    return () => window.removeEventListener("keydown", handler)
  }, [keyMap])
}
```

Write `ui/src/hooks/useReaderKeyboard.ts` with this exact content:
```ts
import { useKeyboard } from "./useKeyboard"

interface ReaderActions {
  nextPage:   () => void
  prevPage:   () => void
  close:      () => void
  increaseFontSize: () => void
  decreaseFontSize: () => void
}

/** Keyboard shortcuts for the EPUB reader. */
export function useReaderKeyboard(actions: ReaderActions) {
  useKeyboard({
    "arrowright":     actions.nextPage,
    "arrowleft":      actions.prevPage,
    "arrowdown":      actions.nextPage,
    "arrowup":        actions.prevPage,
    "escape":         actions.close,
    "meta+=":         actions.increaseFontSize,
    "ctrl+=":         actions.increaseFontSize,
    "meta+-":         actions.decreaseFontSize,
    "ctrl+-":         actions.decreaseFontSize,
  })
}
```

Then run:
```bash
cd ui && npm run build && cd ..
git add ui/src/hooks/useKeyboard.ts ui/src/hooks/useReaderKeyboard.ts
git commit -m "P9-T04: keyboard shortcut hooks — global and reader"
```

---

## P9-T05

In `ui/package.json`, ensure `@tauri-apps/plugin-updater` is present in dependencies:
```json
"@tauri-apps/plugin-updater": "^2.10.1"
```

In `src-tauri/Cargo.toml`, add to dependencies:
```toml
tauri-plugin-updater = { version = "2", features = ["rustls-tls"] }
```

In `src-tauri/src/main.rs`, add the updater plugin to the builder:
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

Write `ui/src/components/UpdateBanner.tsx` with this exact content:
```tsx
import { useEffect, useState } from "react"
import { check } from "@tauri-apps/plugin-updater"

export function UpdateBanner() {
  const [available,    setAvailable]    = useState(false)
  const [downloading,  setDownloading]  = useState(false)
  const [updateHandle, setUpdateHandle] = useState<Awaited<ReturnType<typeof check>> | null>(null)

  useEffect(() => {
    check()
      .then((update) => {
        if (update?.available) {
          setAvailable(true)
          setUpdateHandle(update)
        }
      })
      .catch(() => { /* no update server configured */ })
  }, [])

  if (!available) return null

  const install = async () => {
    if (!updateHandle) return
    setDownloading(true)
    try {
      await updateHandle.downloadAndInstall()
    } catch (e) {
      console.error("Update failed:", e)
      setDownloading(false)
    }
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 bg-blue-600 text-white rounded-xl shadow-lg px-4 py-3 flex items-center gap-3">
      <span className="text-sm font-medium">Update available</span>
      <button
        onClick={install}
        disabled={downloading}
        className="text-sm px-3 py-1 bg-white text-blue-600 rounded-lg hover:bg-blue-50 disabled:opacity-50"
      >
        {downloading ? "Installing…" : "Install & Restart"}
      </button>
    </div>
  )
}
```

Then run:
```bash
cd ui && npm install && npm run build && cd ..
cargo build --workspace
git add ui/src/components/UpdateBanner.tsx ui/package.json \
        src-tauri/Cargo.toml src-tauri/src/main.rs
git commit -m "P9-T05: Tauri auto-updater — banner + plugin wiring"
```

---

## P9-T06

In `src-tauri/tauri.conf.json`, configure the macOS bundle:
```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "identifier": "app.xcalibre.xcalibre",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns"],
    "macOS": {
      "signingIdentity": null,
      "entitlements": null,
      "dmg": {
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 480, "y": 170 }
      }
    }
  }
}
```

To build a signed .dmg (requires Apple Developer account and `xcrun` tools):
```bash
# Set your signing identity (replace with your actual certificate name)
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"
export APPLE_ID="your@apple.id"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="YOURTEAMID"
cargo tauri build --target universal-apple-darwin
```

The resulting .dmg will be in `src-tauri/target/universal-apple-darwin/release/bundle/dmg/`.

For unsigned local testing:
```bash
cargo tauri build
```

Then run:
```bash
cargo tauri build
git add src-tauri/tauri.conf.json
git commit -m "P9-T06: macOS .dmg bundle configuration"
```

---

## P9-T07

In `src-tauri/tauri.conf.json`, add Windows targets to the bundle config:
```json
{
  "bundle": {
    "targets": ["dmg", "app", "msi", "nsis"],
    "windows": {
      "wix": {
        "language": "en-US"
      },
      "nsis": {
        "languages": ["English"]
      }
    }
  }
}
```

To build for Windows (run on Windows or via cross-compilation):
```bash
cargo tauri build --target x86_64-pc-windows-msvc
```

The .msi installer will be in `src-tauri/target/x86_64-pc-windows-msvc/release/bundle/msi/`.

Then run:
```bash
git add src-tauri/tauri.conf.json
git commit -m "P9-T07: Windows .msi / NSIS installer bundle configuration"
```

---

## P9-T08

Write `processing/benches/db_bench.rs` with this exact content:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::runtime::Runtime;
use xcalibre_processing::db::queries;

fn bench_list_jobs(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let pool = rt.block_on(async {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
        // Seed 1000 rows
        for i in 0..1000 {
            let mut job = queries::NewJob::new(&format!("/tmp/book_{}.epub", i), "EPUB");
            job.file_sha256 = format!("sha256_{}", i);
            queries::create_job(&pool, &job).await.unwrap();
        }
        pool
    });

    c.bench_function("list_jobs_by_status_1000", |b| {
        b.iter(|| {
            rt.block_on(async {
                black_box(queries::list_jobs_by_status(&pool, "PENDING").await.unwrap())
            })
        })
    });
}

criterion_group!(benches, bench_list_jobs);
criterion_main!(benches);
```

Then run:
```bash
cd processing && cargo bench --bench db_bench && cd ..
git add processing/benches/db_bench.rs
git commit -m "P9-T08: DB query benchmark — list_jobs_by_status with 1000 rows"
```

---

## P9-T09

Write `processing/benches/pipeline_bench.rs` with this exact content:
```rust
use criterion::{criterion_group, criterion_main, Criterion};
use sqlx::sqlite::SqlitePoolOptions;
use tokio::runtime::Runtime;
use xcalibre_processing::pipeline::ingest::run_ingest;
use std::path::PathBuf;

fn bench_ingest(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("run_ingest_epub", |b| {
        b.iter(|| {
            rt.block_on(async {
                let pool = SqlitePoolOptions::new()
                    .connect("sqlite::memory:")
                    .await
                    .unwrap();
                sqlx::migrate!("src/db/migrations").run(&pool).await.unwrap();
                let path = PathBuf::from("tests/fixtures/fixture_epub.epub");
                run_ingest(&pool, &path).await.unwrap()
            })
        })
    });
}

criterion_group!(benches, bench_ingest);
criterion_main!(benches);
```

Then run:
```bash
cd processing && cargo bench --bench pipeline_bench && cd ..
git add processing/benches/pipeline_bench.rs
git commit -m "P9-T09: pipeline benchmark — run_ingest throughput"
```

---

---

## P9-T10 — Frontend tests: SettingsModal + useKeyboard + MetadataEditorModal

Write tests BEFORE the implementation is considered done — TDD.

Write `ui/src/components/SettingsModal.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/SettingsModal.test.tsx"`.

  Rules:
  - Props to pass: `open_={true}`, `onClose={vi.fn()}`.
  - Component calls `invoke("get_xs_url")` and `invoke("has_token")` on open.
    Use `mockInvoke()` before rendering.
  - For select interactions: use `userEvent.selectOptions()`.
  - Assert store state directly: `useSettingsStore.getState().theme === "dark"`.

Write `ui/src/hooks/useKeyboard.test.ts`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/hooks/useKeyboard.test.ts"`.

  Rules:
  - Use `renderHook()` from `@testing-library/react`.
  - Dispatch real `KeyboardEvent` objects on `window` (not `document`).
  - Wrap handler in `vi.fn()` and pass to `useKeyboard`.
  - Test cleanup: unmount the hook, confirm events no longer fire.

Write `ui/src/components/MetadataEditorModal.test.tsx`:
  Implement every test case from `localProject/TEST_SPEC.md` section
  `"ui/src/components/MetadataEditorModal.test.tsx"`.

  Fixture:
  ```ts
  const BOOK_DETAIL = {
    id: "1", title: "Dune", authors: ["Frank Herbert"], format: "epub",
    pubdate: "1965-08-01", description: "A desert planet.", publisher: "Chilton",
    series_name: "Dune", series_index: 1, rating: 5, tags: ["sci-fi"],
    identifiers: { isbn: "0-441-17271-7" }, cover_path: null,
    progress_percent: 0, last_opened_at: null, reading_cfi: null,
  }
  ```

  Rules:
  - Read `MetadataEditorModal.tsx` to understand how it parses tags
    (comma-separated input vs. tag list add/remove) before writing assertions.
  - Assert that `invoke("update_book_details")` is called with the correct payload.

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1
```
All tests must pass.

```bash
git add ui/src/components/SettingsModal.test.tsx \
        ui/src/hooks/useKeyboard.test.ts \
        ui/src/components/MetadataEditorModal.test.tsx
git commit -m "test(phase9): SettingsModal, useKeyboard, and MetadataEditorModal tests"
```

---

## P9-T11

Before tagging v1.0.0, complete this checklist manually:

```
Release checklist for xCalibre v1.0.0
======================================

Build quality
  [ ] cargo test --workspace                  — all tests pass
  [ ] cargo clippy --workspace -- -D warnings — zero warnings
  [ ] cd ui && npm run build && cd ..         — frontend build clean
  [ ] cargo tauri build                       — Tauri bundle succeeds

Feature completeness (from GAP.md)
  [ ] EPUB — metadata, text, cover, reader        (Phase 1)
  [ ] xcalibre-server push + retry + pull sync          (Phase 2)
  [ ] ISBN enrichment (Open Library + Google Books) (Phase 3)
  [ ] EPUB reader: CFI, bookmarks, progress       (Phase 4)
  [ ] Tags, series, search (FTS5), collections    (Phase 5)
  [ ] Bulk ops, Calibre import, catalog export    (Phase 5)
  [ ] PDF, MOBI, FB2, HTML, RTF extractors        (Phase 6)
  [ ] DOCX, ODT, CHM, LRF, PDB, SNB, TCR stubs   (Phase 7)
  [ ] AZW4, DJVU, LIT stubs                       (Phase 7)
  [ ] Annotations + highlights                    (Phase 8)
  [ ] Comic viewer (CBZ)                          (Phase 8)
  [ ] OS open for PDF, MOBI, others               (Phase 8)
  [ ] Settings UI, error boundary, skeletons      (Phase 9)
  [ ] Auto-updater wired                          (Phase 9)

Distribution
  [ ] Update version in src-tauri/tauri.conf.json to "1.0.0"
  [ ] Update version in ui/package.json to "1.0.0"
  [ ] Update version in processing/Cargo.toml to "1.0.0"
  [ ] Update version in api/Cargo.toml to "1.0.0"
  [ ] cargo tauri build --target universal-apple-darwin  (macOS)
  [ ] cargo tauri build --target x86_64-pc-windows-msvc (Windows)
  [ ] Sign and notarize macOS .dmg

Git
  [ ] All changes committed and pushed to main
  [ ] git tag -a v1.0.0 -m "xCalibre v1.0.0"
  [ ] git push origin v1.0.0
```

When all items are checked:
```bash
# Bump versions in all Cargo.toml and package.json files to 1.0.0, then:
git add .
git commit -m "chore: bump version to 1.0.0"
git tag -a v1.0.0 -m "xCalibre v1.0.0"
git push origin main
git push origin v1.0.0
```

---

### ✅ Milestone check — Phase 9 complete
```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
cargo tauri build
```

`cargo tauri build` currently reaches the DMG packaging step and fails in `bundle_dmg.sh` on this machine; `cargo tauri build --bundles app` succeeds.

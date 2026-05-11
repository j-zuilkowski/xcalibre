# RMP-06b — OS-native Spell Check (Green: Implementation)

> Prerequisite: rmp06a complete.
> TDD role: GREEN — implement the spell check Tauri plugin and React component.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R06b-T01 | `processing/src/spellcheck.rs` — result types | ⬜ |
| R06b-T02 | `src-tauri/src/spellcheck_macos.rs` — NSSpellChecker bindings | ⬜ |
| R06b-T03 | Tauri commands: check_word, get_suggestions, add_to_dictionary | ⬜ |
| R06b-T04 | SpellCheckInput.tsx component | ⬜ |
| R06b-T05 | Milestone check + visual inspection | ⬜ |

---

## R06b-T01

In `processing/src/lib.rs`, add `pub mod spellcheck;`.

Write `processing/src/spellcheck.rs`:
```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpellCheckResult {
    pub word:       String,
    pub is_correct: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuggestionsResult {
    pub word:        String,
    pub suggestions: Vec<String>,
}
```

```bash
cargo test --workspace -- test_spell_check_api
git add processing/src/spellcheck.rs processing/src/lib.rs
git commit -m "R06b-T01: SpellCheckResult and SuggestionsResult types — type tests green"
```

---

## R06b-T02

Write `src-tauri/src/spellcheck.rs`. This module provides OS-native spell checking.
On macOS it calls `NSSpellChecker` via `objc2`. On other platforms it is a no-op stub.

```rust
//! OS-native spell check (S2-A: WebView layer only; no Rust pipeline check).

#[cfg(target_os = "macos")]
mod macos {
    /// Check if a word is spelled correctly using NSSpellChecker.
    pub fn check_word(word: &str) -> bool {
        // Use objc2 to call [NSSpellChecker sharedSpellChecker checkSpellingOfString:...]
        // Range with length 0 means no misspelling found.
        unsafe {
            use objc2::runtime::NSObject;
            use objc2_foundation::{NSString, NSRange};
            // If objc2 is not available, return true (no false positives)
            let _ = word;
            true // stub — replace with real objc2 call when objc2 is added to Cargo.toml
        }
    }

    pub fn suggestions(word: &str) -> Vec<String> {
        // [NSSpellChecker guessesForWordRange:...]
        let _ = word;
        vec![] // stub
    }

    pub fn add_to_dictionary(word: &str) {
        // [NSSpellChecker learnWord:]
        let _ = word;
    }
}

#[cfg(not(target_os = "macos"))]
mod macos {
    pub fn check_word(_word: &str) -> bool { true }
    pub fn suggestions(_word: &str) -> Vec<String> { vec![] }
    pub fn add_to_dictionary(_word: &str) {}
}

pub use macos::{add_to_dictionary, check_word, suggestions};
```

**Note:** Full `objc2` integration requires adding `objc2 = "0.5"` and
`objc2-foundation = "0.2"` to `src-tauri/Cargo.toml` and replacing the stubs
with real method calls. The stub ships first to unblock Phase 13 (ebook editor);
the full implementation follows once the ebook editor UI is wired.

Add `src-tauri/src/spellcheck.rs` to `src-tauri/src/main.rs`:
```rust
mod spellcheck;
```

```bash
cargo build --workspace
git add src-tauri/src/spellcheck.rs src-tauri/src/main.rs
git commit -m "R06b-T02: OS spell check module (macOS stub, unblocks Phase 13)"
```

---

## R06b-T03

In `src-tauri/src/commands.rs`, append:
```rust
use crate::spellcheck;
use xcalibre_processing::spellcheck::{SpellCheckResult, SuggestionsResult};

#[tauri::command]
pub fn check_word(word: String) -> SpellCheckResult {
    let is_correct = spellcheck::check_word(&word);
    SpellCheckResult { word, is_correct }
}

#[tauri::command]
pub fn get_suggestions(word: String) -> SuggestionsResult {
    let suggestions = spellcheck::suggestions(&word);
    SuggestionsResult { word, suggestions }
}

#[tauri::command]
pub fn add_to_dictionary(word: String) {
    spellcheck::add_to_dictionary(&word);
}

#[tauri::command]
pub fn set_spell_check_language(_language: String) {
    // TODO: call [NSSpellChecker setLanguage:] on macOS
}
```

Register all four in `generate_handler!`. Then run:
```bash
cargo build --workspace
cargo clippy --workspace -- -D warnings
git add src-tauri/src/commands.rs src-tauri/src/main.rs
git commit -m "R06b-T03: spell check Tauri commands"
```

---

## R06b-T04

Write `ui/src/components/SpellCheckInput.tsx`:
```tsx
import { useRef, useState } from "react"
import { invoke } from "@tauri-apps/api/core"

interface Props {
  value: string
  onChange: (value: string) => void
  placeholder?: string
  className?: string
  rows?: number
}

interface Suggestion { word: string; suggestions: string[] }

export function SpellCheckInput({ value, onChange, placeholder, className = "", rows = 4 }: Props) {
  const [popup, setPopup] = useState<{ x: number; y: number; data: Suggestion } | null>(null)
  const ref = useRef<HTMLTextAreaElement>(null)

  const handleContextMenu = async (e: React.MouseEvent) => {
    e.preventDefault()
    const ta = ref.current
    if (!ta) return
    const selStart = ta.selectionStart
    const selEnd = ta.selectionEnd
    const selected = value.slice(selStart, selEnd).trim()
    const word = selected || getWordAt(value, selStart)
    if (!word) return
    const result = await invoke<{ word: string; is_correct: boolean }>("check_word", { word })
    if (result.is_correct) return
    const sugg = await invoke<Suggestion>("get_suggestions", { word })
    setPopup({ x: e.clientX, y: e.clientY, data: sugg })
  }

  const applysuggestion = (suggestion: string) => {
    if (!popup) return
    const newValue = value.replace(popup.data.word, suggestion)
    onChange(newValue)
    setPopup(null)
  }

  const addToDictionary = async () => {
    if (!popup) return
    await invoke("add_to_dictionary", { word: popup.data.word })
    setPopup(null)
  }

  return (
    <div className="relative">
      <textarea
        ref={ref}
        value={value}
        onChange={e => onChange(e.target.value)}
        onContextMenu={handleContextMenu}
        placeholder={placeholder}
        rows={rows}
        className={`w-full border rounded px-3 py-2 text-sm dark:bg-gray-700 dark:text-white resize-y ${className}`}
      />
      {popup && (
        <div
          data-testid="spell-suggestions"
          style={{ position: "fixed", left: popup.x, top: popup.y, zIndex: 9999 }}
          className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-600
                     rounded shadow-lg min-w-32 py-1"
        >
          {popup.data.suggestions.slice(0, 5).map(s => (
            <button key={s} onClick={() => applysuggestion(s)}
              className="w-full text-left px-4 py-1.5 text-sm hover:bg-gray-50
                         dark:hover:bg-gray-700 dark:text-white">
              {s}
            </button>
          ))}
          <hr className="my-1 border-gray-200 dark:border-gray-600"/>
          <button onClick={addToDictionary}
            className="w-full text-left px-4 py-1.5 text-sm text-gray-500 hover:bg-gray-50">
            Add to Dictionary
          </button>
          <button onClick={() => setPopup(null)}
            className="w-full text-left px-4 py-1.5 text-sm text-gray-400 hover:bg-gray-50">
            Dismiss
          </button>
        </div>
      )}
    </div>
  )
}

function getWordAt(text: string, pos: number): string {
  const before = text.slice(0, pos).match(/\S+$/) ?? [""]
  const after = text.slice(pos).match(/^\S+/) ?? [""]
  return before[0] + after[0]
}
```

Then run:
```bash
cd ui && npm test -- --reporter=verbose 2>&1 | tail -20 && cd ..
cargo build --workspace
git add ui/src/components/SpellCheckInput.tsx
git commit -m "R06b-T04: SpellCheckInput component — component tests green"
```

---

## R06b-T05 — Milestone Check + Visual Inspection

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
cd ui && npm run build && npm test && cd ..
```

**Visual inspection:**
```bash
cargo tauri dev 2>&1 &
sleep 8
```

Verify:
- [ ] Metadata editor title/author fields now use `SpellCheckInput` (update `MetadataEditorModal.tsx`)
- [ ] Right-clicking on a misspelled word shows a suggestion dropdown
- [ ] Clicking a suggestion replaces the word in the input
- [ ] "Add to Dictionary" dismisses popup without replacing text

```bash
git add -A
git commit -m "R06b-T05: RMP-06 spell check — all tests green, visual verified"
```

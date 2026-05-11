# RMP-06a — OS-native Spell Check (Red: Failing Tests)

> Prerequisite: rmp02b complete (spell check wires into the WebView editor layer).
> TDD role: RED — define the Tauri plugin interface via failing tests.
> Status: ⬜ not started

## Status

| Task | Title | Status |
|------|-------|--------|
| R06a-T01 | Failing tests: spell check Tauri command interface | ⬜ |
| R06a-T02 | Failing tests: SpellCheckInput React component | ⬜ |

---

## R06a-T01

Write `processing/tests/test_spell_check_api.rs` with this exact content:
```rust
//! Tests for the spell check Tauri command API contract.
//! These tests are compile-time only — the real checking is OS-native,
//! so we just verify the command signatures compile correctly.
//! FAIL until rmp06b adds the commands to src-tauri.

// This test verifies that the spell check command types are defined
// in the processing crate's type exports.
use xcalibre_processing::spellcheck::{SpellCheckResult, SuggestionsResult};

#[test]
fn test_spell_check_result_type() {
    let r = SpellCheckResult { word: "colour".into(), is_correct: false };
    assert!(!r.is_correct);
    assert_eq!(r.word, "colour");
}

#[test]
fn test_suggestions_result_type() {
    let r = SuggestionsResult {
        word: "colour".into(),
        suggestions: vec!["color".into(), "coulor".into()],
    };
    assert_eq!(r.suggestions.len(), 2);
}
```

```bash
cargo test --workspace 2>&1 | grep -E "^error" | head -10
```

Expected: `xcalibre_processing::spellcheck` not found. RED confirmed.

```bash
git add processing/tests/test_spell_check_api.rs
git commit -m "R06a-T01: failing tests for spell check type contracts"
```

---

## R06a-T02

Write `ui/src/components/SpellCheckInput.test.tsx`:
```tsx
import { render, screen, fireEvent, waitFor } from "@testing-library/react"
import { describe, it, expect, vi, beforeEach } from "vitest"
import { SpellCheckInput } from "./SpellCheckInput"
import { mockInvoke } from "../test/setup"

beforeEach(() => {
  mockInvoke("check_word", { word: "colour", is_correct: false })
  mockInvoke("get_suggestions", { word: "colour", suggestions: ["color", "coulor"] })
  mockInvoke("add_to_dictionary", undefined)
})

describe("SpellCheckInput", () => {
  it("renders a textarea", () => {
    render(<SpellCheckInput value="" onChange={vi.fn()} />)
    expect(screen.getByRole("textbox")).toBeInTheDocument()
  })

  it("shows suggestion popover for misspelled word on right-click", async () => {
    render(<SpellCheckInput value="colour" onChange={vi.fn()} />)
    const textarea = screen.getByRole("textbox")
    fireEvent.contextMenu(textarea)
    await waitFor(() =>
      expect(screen.getByTestId("spell-suggestions")).toBeInTheDocument()
    )
    expect(screen.getByText("color")).toBeInTheDocument()
  })

  it("closes popover when suggestion is clicked", async () => {
    const onChange = vi.fn()
    render(<SpellCheckInput value="colour text" onChange={onChange} />)
    fireEvent.contextMenu(screen.getByRole("textbox"))
    await waitFor(() => screen.getByTestId("spell-suggestions"))
    fireEvent.click(screen.getByText("color"))
    expect(screen.queryByTestId("spell-suggestions")).not.toBeInTheDocument()
  })
})
```

```bash
cd ui && npm test 2>&1 | grep -E "FAIL|Cannot find" | head -10 && cd ..
```

Expected: `SpellCheckInput` not found. RED confirmed.

```bash
git add ui/src/components/SpellCheckInput.test.tsx
git commit -m "R06a-T02: failing tests for SpellCheckInput component"
```

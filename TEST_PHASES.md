# xcalibre Desktop — Test Implementation

Frontend tests are now embedded directly in the development phase files,
co-located with the components they test (TDD).

| Component(s) | Phase file | Tasks |
|---|---|---|
| vitest setup, Tauri IPC mock | `docs/phase1_commands.md` | P1-T36 |
| libraryStore, settingsStore, LibraryView | `docs/phase1_commands.md` | P1-T37, P1-T38 |
| CollectionsSidebar, FilterBar | `docs/phase5_commands.md` | P5-T23 |
| SearchBar, BulkActionBar | `docs/phase5_commands.md` | P5-T24 |
| SettingsModal, useKeyboard, MetadataEditorModal | `docs/phase9_commands.md` | P9-T10 |

Test case specifications: `localProject/TEST_SPEC.md`

## Phase maintenance rule

Any change made during a build must be reflected back in the corresponding
phase file before committing. See `AGENTS.md` Non-Negotiable Constraints.

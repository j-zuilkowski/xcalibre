# xcalibre

A fast, local-first ebook library manager for macOS, Windows, and Linux — built with Tauri 2, Rust, and React.

xcalibre lets you import, organise, read, annotate, and convert your ebook collection entirely on your own machine. An optional [xcalibre-server](https://github.com/j-zuilkowski/xcalibre-server) integration syncs your library and annotations across devices.

---

## Screenshots

> _Screenshots coming soon_

---

## Features

- **25-format support** — EPUB, PDF, MOBI/AZW3, CBZ/CBR, FB2, DOCX, ODT, RTF, HTML, CHM, LIT, DjVu, SNB, LRF, PDB, TCR, KFX, and more
- **In-app EPUB reader** — custom rendering engine with CFI position tracking, bookmarks, and highlights
- **Format conversion** — convert to 15 output formats (TXT, HTML, DOCX, PDF, MOBI, KEPUB, FB2, RTF, HTMLZ, LRF, PDB, PML, RB, SNB, TCR)
- **Full-text search** — FTS5-powered with field-prefix syntax (`title:`, `authors:`, `tags:`)
- **AI book chat** — RAG-based Q&A on your books using Ollama, OpenAI, Gemini, LM Studio, or OpenRouter
- **Metadata editor** — bulk edit, ISBN enrichment via Open Library and Google Books
- **Collections and virtual libraries** — group books manually or by saved search queries
- **Annotations** — highlights and notes with optional server sync
- **EPUB editor** — in-place ZIP editing with spell check
- **Plugin system** — extend metadata sources and conversion outputs via native dylib plugins
- **Multiple libraries** — create and switch between separate book databases
- **Backup and restore** — full library backup including book files

See [FEATURES.md](FEATURES.md) for the full feature list.

---

## Installation

### Pre-built releases

Download the latest installer from the [Releases](https://github.com/j-zuilkowski/xcalibre/releases) page.

| Platform | File |
|----------|------|
| macOS | `.dmg` |
| Windows | `.msi` |
| Linux | `.AppImage` or `.deb` |

### Build from source

**Prerequisites:** Rust 1.78+, Node.js 20+, Tauri CLI 2.x

```bash
git clone https://github.com/j-zuilkowski/xcalibre.git
cd xcalibre/ui && npm install
cd .. && cargo tauri build
```

---

## Quick start

1. Launch xcalibre.
2. Drag ebook files onto the library window, or use **File → Import**.
3. Click a book to open it in the reader.
4. Use the search bar (`Cmd+F` / `Ctrl+F`) to search across titles, authors, and full text.
5. Open **Settings → AI Provider** to configure an AI backend for book chat.

See the [User Guide](docs/USER_GUIDE.md) for detailed instructions.

---

## Documentation

| Document | Description |
|----------|-------------|
| [User Guide](docs/USER_GUIDE.md) | End-user documentation — all features explained |
| [Developer Guide](docs/DEVELOPER_GUIDE.md) | Architecture, adding formats/providers, plugin development |
| [Features](FEATURES.md) | Full feature list with status |
| [Roadmap](docs/ROADMAP.md) | Planned phases and dependency graph |
| [Architecture](docs/ARCHITECTURE.md) | System design decisions |
| [Plugin SDK](xcalibre-plugin-sdk/src/lib.rs) | Plugin ABI reference (inline docs) |

---

## Plugin development

xcalibre supports native plugins for metadata sources and conversion outputs.
Plugins are distributed as ZIP archives containing a shared library and a
`plugin.json` manifest.

See the [Plugin Development](docs/DEVELOPER_GUIDE.md#10-plugin-development) section
of the Developer Guide for a complete walkthrough including a working Rust example.

---

## Architecture

```
xcalibre (Tauri 2 shell)
├── xcalibre-processing    Rust — pipeline, DB, format detection, converters
├── xcalibre-ai            Rust — AI provider abstraction (Ollama, OpenAI, Gemini, …)
├── xcalibre-epub          Rust — EPUB read/write/edit library
├── xcalibre-plugin-sdk    Rust — stable ABI for third-party plugins
├── api                    Rust — xcalibre-server HTTP client
└── ui                     React 18 + TypeScript + Tailwind
```

---

## Contributing

1. Fork the repository.
2. Create a feature branch: `git checkout -b my-feature`.
3. Read the [Developer Guide](docs/DEVELOPER_GUIDE.md) before writing code.
4. Run `cargo test --workspace` and ensure all tests pass.
5. Open a pull request.

---

## License

MIT — see [LICENSE](LICENSE) for details.

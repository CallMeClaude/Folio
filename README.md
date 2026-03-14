# Folio

A beautiful, distraction-free word processor for Linux, built with GTK4, libadwaita, Pango, and Cairo.

## Features

- **Real paginated layout** — A4, A5, A3, Letter, Legal, Tabloid with configurable margins and orientation
- **Rich text** — Bold, italic, underline, strikethrough, superscript, subscript, text colour, highlight, links
- **Block styles** — Title, Heading 1/2, Caption, Quote, Code, Bullet list, Numbered list, Checklist, Divider, Image
- **Find & Replace** — Literal and regex modes, Prev/Next navigation, Replace/Replace All
- **Export** — PDF (Cairo native), Plain text, Markdown, HTML
- **Focus mode** — dims inactive blocks, hides toolbar and sidebar
- **Typewriter mode** — keeps the active block centred vertically
- **Dark mode** — system-follow or manual override
- **Auto-save** — saves every 2 seconds when there are unsaved changes
- **Undo/Redo** — powered by [loro](https://loro.dev/) CRDT
- **Font management** — 5 bundled fonts, install from .ttf, download from CDN URL
- **Document browser** — launch screen lists recent documents

## Building

### Prerequisites

- Rust 1.75+
- GTK4 development headers (`gtk4-devel` / `libgtk-4-dev`)
- libadwaita (`libadwaita-devel` / `libadwaita-1-dev`)
- Pango, Cairo, GLib

### Arch Linux

```sh
sudo pacman -S gtk4 libadwaita pango cairo
cargo build --release
```

### Running

```sh
cargo run -p folio-gtk
```

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| Ctrl+S | Save |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z / Ctrl+Y | Redo |
| Ctrl+F | Find & Replace |
| Ctrl+Shift+F | Toggle Focus Mode |
| Ctrl+Shift+T | Toggle Typewriter Mode |
| Escape | Exit Focus Mode |

## File Format

Documents are saved as `.folio` bundles — ZIP archives containing:

- `document.json` — metadata (title, paper size, margins, timestamps)
- `content.loro` — the full document content as a [loro](https://loro.dev/) CRDT snapshot
- `assets/` — embedded images, referenced by UUID

## License

GPL-3.0-only — see [LICENSE](LICENSE)

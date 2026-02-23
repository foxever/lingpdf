# lingpdf

A lightweight, cross-platform PDF reader built with Rust and GPUI.

## Features

- **PDF Rendering**: High-fidelity page rendering using PDFium engine with HiDPI/Retina display support
- **Navigation**: Page navigation, zoom, rotation, fit width/page
- **Outline**: Table of contents sidebar with bookmark navigation
- **Print**: Native print dialog support (macOS)
- **Themes**: Light/dark mode support
- **i18n**: Multi-language support (English, Chinese, Spanish)
- **Recent Files**: Quick access to recently opened documents

## Tech Stack

- **Language**: Rust
- **UI Framework**: GPUI 0.2.2
- **PDF Engine**: PDFium (pdfium-render 0.8)
- **Platforms**: Windows / macOS / Linux

## Quick Start

### Requirements
- Rust 1.70+

### Build & Run

```bash
# Clone
git clone <repository-url>
cd lingpdf

# Run
cargo run

# Open a PDF file
cargo run -- <path-to-pdf>
```

### Cross-platform Build

```bash
# Current platform
./build.sh

# All platforms
./build.sh --all

# Or use Makefile
make build        # Current platform
make build-all    # All platforms
```

## Usage

| Action | Control |
|--------|---------|
| Open file | Click 📂 button |
| Navigate | ◀ / ▶ buttons, ←/→ arrow keys |
| First/Last page | ⏮ / ⏭ buttons |
| Scroll | Mouse wheel, click left/right 1/3 of page |
| Zoom | − / + buttons, 1:1 for reset |
| Fit | ↔ fit width, □ fit page |
| Rotate | ↻ / ↺ buttons |
| Print | 🖨️ button (macOS native dialog) |
| Text Selection | Click 👋/🖱️ button to toggle hand/text select mode, then drag to select |
| Copy Text | Select text, then press ⌘+C (macOS) or Ctrl+C (Windows/Linux) |
| Sidebar | 📑 / 📖 toggle outline |
| Scroll mode | 📄 / 📜 toggle page/smooth scroll |
| Theme | Click 🌙 / ☀️ icon |
| Language | Click flag icon 🇺🇸/🇨🇳/🇪🇸 |
| Fullscreen | Menu → View → Fullscreen |

## Roadmap

### Done
- [x] PDF rendering with PDFium
- [x] Page navigation and zoom
- [x] Page rotation
- [x] Outline navigation
- [x] Light/dark themes
- [x] Multi-language support
- [x] Cross-platform CI/CD
- [x] Keyboard navigation (arrow keys)
- [x] Mouse wheel scrolling
- [x] Click-to-navigate
- [x] Full-screen mode

### TODO

#### Core Features
- [x] Search (text search API implemented, UI needs dialog)
- [ ] Bookmarks (save page positions)
- [x] Recent files (quick access)
- [ ] Drag & drop (open files)
- [x] Print support (macOS native dialog)

#### Navigation
- [x] Page thumbnails sidebar (text list)
- [ ] Go to page (jump to specific page, needs dialog)
- [ ] Previous/Next document

#### Display
- [x] Fit width/height/page modes
- [x] Continuous scroll mode (smooth scroll)
- [ ] Presentation mode
- [x] Custom zoom levels

#### Advanced
- [x] Text selection and copy
- [ ] Annotation support
- [ ] Form filling
- [ ] Digital signatures
- [ ] PDF encryption/decryption

## License

MIT License

<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="128" height="128" alt="clippeR Icon">
</p>

<h1 align="center">clippeR</h1>

<p align="center">
  A lightning-fast, minimalist clipboard manager for Windows.<br>
  Built with Rust, Tauri 2.0, and Vanilla JS.
</p>

<p align="center">
  <a href="../../releases"><img src="https://img.shields.io/github/v/release/mehmetturuncx/clippeR?style=flat-square&color=3daee9" alt="Release"></a>
  <img src="https://img.shields.io/badge/platform-Windows-blue?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/license-MIT-green?style=flat-square" alt="License">
</p>

---

## ✨ Features

- 🚀 **Zero Idle CPU** — Fully event-driven architecture with no polling. Uses native `WM_CLIPBOARDUPDATE` events.
- 🖼️ **Image Support** — Automatically captures and displays copied images as PNG previews.
- 🔍 **Instant Search** — Filter through your clipboard history in real time.
- 💾 **Persistent History** — Stored locally in SQLite. Survives restarts, never loses data.
- ⚡ **Global Shortcut** — `Alt + V` to toggle the window from anywhere.
- 🖱️ **System Tray** — Lives quietly in your taskbar. Click away to dismiss.
- 🧹 **Clean Delete** — Deleting entries also removes associated image files from disk.

## 📥 Installation

1. Download the latest installer from the [Releases](../../releases) page.
2. Run `clipper_x64-setup.exe`.
3. Press `Alt + V` or click the tray icon to start.

## 🛠️ Development

```bash
git clone https://github.com/mehmetturuncx/clippeR.git
cd clippeR
npm install
npm run tauri dev   # Run locally
npm run tauri build # Build executable
```

**Prerequisites:** [Node.js](https://nodejs.org/) (v16+), [Rust](https://www.rust-lang.org/tools/install), MSVC Build Tools

## ⚙️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vanilla HTML / CSS / JS |
| Backend | Rust |
| Framework | Tauri 2.0 |
| Database | SQLite (rusqlite) |
| Clipboard | arboard + clipboard-master |

## 📄 License

MIT

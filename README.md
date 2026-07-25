# clippeR 📎🦀

A lightweight, fast clipboard manager built with [Rust](https://www.rust-lang.org/) and [Tauri](https://tauri.app/), inspired by KDE Plasma's Klipper.

<p align="center">
  <img src="app_icon_rust_transparent.png" width="128" height="128" alt="clippeR Icon">
</p>

## ✨ Features

- 🖥️ **KDE Klipper-Inspired UI**: Clean, flat interface with sharp corners and a solid dark theme following the Breeze Dark color palette.
- 🚀 **Blazing Fast**: Powered by Rust and Tauri 2.0, utilizing almost zero background resources.
- 💾 **Persistent History**: Clipboard history is stored in a local SQLite database — survives restarts and never loses your data.
- ⚡ **Global Shortcut**: Instantly access your clipboard from anywhere using `Alt + V`.
- 🖱️ **System Tray Integration**: Lives quietly in your system tray. Click the icon or press the shortcut to open. Click away to dismiss.
- 🔽 **Smart Expansion**: Long or multiline texts are collapsed by default. Click the expand button to reveal the full content.
- 🗑️ **Quick Actions**: Delete individual entries or clear the entire history with a single click.
- 🎯 **Professional SVG Icons**: All action buttons use crisp, scalable vector icons.

## 📥 Installation

No development tools required — just download and run:

1. Go to the [Releases](../../releases) page.
2. Download the latest `clipper_x64-setup.exe` (recommended) or `.msi` installer.
3. Run the installer.
4. The clippeR icon will appear in your system tray. Click it or press `Alt + V` to start.

## 🛠️ Building from Source

### Prerequisites
- [Node.js](https://nodejs.org/) (v16+)
- [Rust](https://www.rust-lang.org/tools/install)
- MSVC C++ Build Tools (Windows)

### Development
```bash
git clone https://github.com/yourusername/clippeR.git
cd clippeR
npm install
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
```
Output: `src-tauri/target/release/bundle/nsis/` and `src-tauri/target/release/bundle/msi/`

## ⚙️ Tech Stack

| Layer | Technology |
|-------|-----------|
| Frontend | Vanilla HTML / CSS / JS |
| Backend | Rust |
| Framework | Tauri 2.0 |
| Database | SQLite (rusqlite) |
| Icons | Inline SVG (Material Design) |

## 📄 License

This project is open-source and available under the MIT License.

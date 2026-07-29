<p align="center">
  <img src="app_icon_rust_transparent.png" width="128" height="128" alt="clippeR Icon">
</p>

<h1 align="center">clippeR</h1>

<p align="center">
  A lightning-fast, minimalist clipboard manager for Windows.
</p>

## ✨ Features

- 🚀 **Blazing Fast**: Event-driven architecture ensures ~0% background CPU usage.
- 🖼️ **Image Support**: Automatically captures and displays copied images.
- 🔍 **Instant Search**: Quickly filter and find your past clipboard entries.
- 💾 **Persistent**: History is safely stored locally (SQLite) and survives PC restarts.
- ⚡ **Global Shortcut**: Access your clipboard instantly from anywhere with `Alt + V`.
- 🎨 **Minimal UI**: Clean, sharp, and distraction-free dark mode interface.
- 🖱️ **System Tray**: Lives quietly in your taskbar. Click away to dismiss.

## 📥 Installation

1. Download the latest installer from the [Releases](../../releases) page.
2. Run the `clipper_x64-setup.exe` file.
3. Press `Alt + V` or click the tray icon to start!

## 🛠️ Development

Built with Vanilla JS, Rust, and Tauri 2.0.

```bash
git clone https://github.com/mehmetturuncx/clippeR.git
cd clippeR
npm install
npm run tauri dev   # Run locally
npm run tauri build # Build executable
```

## 📄 License

MIT License

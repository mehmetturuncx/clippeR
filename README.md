# clippeR 📎🦀

A modern, fast, and sleek Clipboard Manager built with [Rust](https://www.rust-lang.org/) and [Tauri](https://tauri.app/).

<p align="center">
  <img src="app_icon_rust_transparent.png" width="128" height="128" alt="clippeR Icon">
</p>

## ✨ Features

- 🎨 **Glassmorphism UI**: A beautiful, translucent popup interface that blends perfectly into your modern desktop environment.
- 🚀 **Blazing Fast**: Powered by Rust and Tauri 2.0, utilizing almost zero background resources.
- 💾 **Persistent SQLite History**: Your clipboard history is safely stored in a local SQLite database, ensuring you never lose copied text across system restarts.
- ⚡ **Global Shortcut**: Access your clipboard anywhere, anytime instantly using the `Alt + V` shortcut.
- 🖱️ **Smart Auto-Hide**: Operates quietly in your system tray and gracefully hides itself the moment you click away.
- 🔽 **Smart Text Expansion**: Long texts and multiline codes are neatly collapsed to save space. Just click the "▼" button to expand and read the full content.

## 📥 Installation (For Users)

You don't need to be a developer to use clippeR! You can simply download the pre-built installer:

1. Go to the [Releases](../../releases) page of this repository.
2. Download the latest `clipper_x64-setup.exe` or `.msi` file.
3. Run the installer.
4. Look for the elegant Rust-geared paperclip icon in your system tray and start managing your clipboard!

## 🛠️ Development (For Developers)

If you'd like to build the project from source or contribute:

### Prerequisites
- [Node.js](https://nodejs.org/) (v16 or newer)
- [Rust](https://www.rust-lang.org/tools/install)
- OS-specific Tauri prerequisites (e.g., MSVC C++ Build Tools for Windows).

### Running Locally

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/clippeR.git
   cd clippeR
   ```

2. Install frontend dependencies:
   ```bash
   npm install
   ```

3. Run in development mode:
   ```bash
   npm run tauri dev
   ```

### Building for Production

To compile the application into a standalone executable or installer:

```bash
npm run tauri build
```
Once finished, you can find the `.exe` and `.msi` setup files in the `src-tauri/target/release/bundle/` directory.

## ⚙️ Tech Stack
- **Frontend**: Vanilla HTML/CSS/JS (Ultra-fast & lightweight)
- **Backend**: Rust
- **Framework**: Tauri 2.0
- **Database**: SQLite (`rusqlite`)

## 📄 License
This project is open-source and available under the MIT License.

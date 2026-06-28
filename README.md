# GrammarFix

A lightweight Tauri-based grammar correction tool that lives in your system tray. Paste text, fix grammar, copy result — done.

## Features

- **Instant grammar correction** via Gemini 3.1 Flash Lite
- **System tray** — minimizes to tray, never gets in the way
- **Global shortcut** — `Alt+P` to show/hide from anywhere
- **Auto-copy** — corrected text is automatically copied to clipboard
- **Auto-start** — optional launch on Windows startup
- **Lightweight** — minimal resource usage, fast startup

## Download

Grab the latest `GrammarFix.exe` from the [`release/`](release/) folder.

## Usage

1. Launch `GrammarFix.exe`
2. Click the gear icon and enter your Gemini API key
3. Paste your text and click **Fix Grammar** (or `Ctrl+Enter`)
4. Corrected text is auto-copied — paste it wherever you need

### Shortcuts

| Shortcut | Action |
|----------|--------|
| `Alt+P` | Show/hide window |
| `Ctrl+Enter` | Fix grammar |
| `✕` button | Minimize to tray |
| Tray → Close | Quit app |

## Building from Source

### Prerequisites

- [Rust](https://rustup.rs/)
- [Node.js](https://nodejs.org/)
- Microsoft Visual Studio C++ Build Tools

### Build

```bash
npm install
npm run tauri dev    # Development
npm run tauri build  # Production build
```

The standalone executable will be at `src-tauri/target/release/GrammarFix.exe`.

## Tech Stack

- **Frontend:** Vanilla HTML/CSS/JS
- **Backend:** Rust (Tauri v2)
- **API:** Google Gemini (`gemini-3.1-flash-lite`)
- **Plugins:** global-shortcut, autostart

## Configuration

API key is stored at `%AppData%/GrammarFix/config.json`.

## License

MIT

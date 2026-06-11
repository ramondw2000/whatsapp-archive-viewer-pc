# WhatsApp Archive Viewer (PC)

A desktop application for viewing WhatsApp chat exports on Windows, macOS, and Linux.

Built with **Tauri 2**, **React 19**, and **Rust**.

## Features

- Import WhatsApp chat exports (ZIP files) — Android and iOS formats
- View messages, media, and attachments inline
- Search and filter messages
- Customize chat backgrounds
- Edit contact profiles with notes and phone numbers
- Export and re-import chats

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** toolchain — install via [rustup.rs](https://rustup.rs)
- Platform-specific build tools:
  - **Windows**: [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: `libwebkit2gtk-4.0-dev build-essential curl wget file libssl-dev libayatana-appindicator3-dev librsvg2-dev`

## Installation

1. Install dependencies (includes the Tauri CLI):
```bash
npm install
```

## Development

Start the development server with hot-reload:
```bash
npm run tauri:dev
```

## Building

Build a release bundle for your current platform:
```bash
npm run tauri:build
```

Output locations:
- **Windows**: `src-tauri/target/release/bundle/nsis/WhatsApp Archive Viewer (PC)_<version>_x64-setup.exe`
- **macOS**: `src-tauri/target/release/bundle/dmg/`
- **Linux**: `src-tauri/target/release/bundle/appimage/`

> **Note:** The first build takes 15–30 minutes as Rust compiles all dependencies from scratch. Subsequent builds are incremental and much faster. If you encounter a linker error about incompatible PDB files, run `cargo clean` inside `src-tauri/` and retry.

## Testing

Run backend (Rust) tests:
```bash
cd src-tauri
cargo test
```

## Project Structure

```
project-code/
├── src/              # React 19 frontend (TypeScript)
├── src-tauri/        # Rust backend (Tauri 2)
│   ├── src/lib.rs    # All IPC commands and business logic
│   ├── capabilities/ # Tauri permission grants
│   └── icons/        # App and installer icons
└── public/           # Static assets
```

## Platform-Specific Notes

### Windows
- Produces an NSIS installer; VC++ redistributable bundled automatically
- Requires Visual Studio Build Tools 2019 or later

### macOS
- Produces a DMG installer
- Requires code signing for distribution outside development

### Linux
- Produces an AppImage
- May require additional system libraries depending on the distribution

## Security

This application handles user-private WhatsApp chat data. The following hardening measures are in place:

- **Path confinement** — all IPC file operations are restricted to the app data directory; path traversal inputs are rejected
- **ZIP extraction safety** — uses `enclosed_name()` to prevent zip-slip; enforces entry count (10,000) and per-file size (500 MB) caps
- **Extension allowlist** — only safe media extensions can be opened via the OS handler
- **URL scheme validation** — only `http://` and `https://` URLs can be opened externally
- **Asset protocol scope** — webview asset access is restricted to app-owned subdirectories only
- **Image decode limits** — dimension cap (16,000×16,000 px) and allocation cap (256 MB) prevent decompression bombs
- **CSP** — no external script or font sources; `unsafe-eval` absent

## License

This project is private.

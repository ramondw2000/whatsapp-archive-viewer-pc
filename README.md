# WhatsApp Archive Viewer (PC)

A desktop application for viewing WhatsApp chat exports on Windows, macOS, and Linux.

Built with **Tauri 2**, **React 19**, and **Rust**.

## Features

- Import WhatsApp chat exports (ZIP files) — Android and iOS formats, including batch import of
  multiple ZIPs at once and restoring from a prior app backup ZIP
- Group detection and member-added/removed/left tracking across 32 languages (see table below)
- View messages, media, and attachments inline (images, video, audio, GIFs, stickers, files, locations,
  contact cards, polls), with a media gallery and lightbox viewer
- Full-text search within a chat, with advanced filters (date range, sender, message type)
- Star/favorite messages
- Customize chat backgrounds (default or custom images, with history)
- Edit contact profiles — name, notes, phone number, photo, and name history
- Link a person's identity across every group they're in and their 1-on-1 chat, so profile edits made
  from any of them stay in sync ("contacts" feature — see `TODO.md` for current limitations). A custom
  name overrides the raw exported sender name in both message bubbles and the Group Info participant list
- Multi-select messages (long-press to start, click to toggle more, shift-click to select a range) with
  bulk favorite-toggling — an amber ring marks which message a shift-click will range from
- Light/dark theme toggle; defaults to the OS's light/dark setting the first time the app runs
- Export a single chat or all chats to ZIP, and re-import them later

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

Run frontend (React/TypeScript) tests:
```bash
npm test
```

Synthetic per-language WhatsApp export ZIPs for manually verifying group detection and member
tracking are in `src-tauri/tests/test_cases/language_samples/` — import one via the app's own
"+ Import" button to sanity-check a specific language.

### Language support status

Group detection and member-added/removed/left tracking (`detect_group_chat` /
`extract_membership_events` in `src-tauri/src/lib.rs`). "Tested against a real export" means an actual
user-provided WhatsApp export confirmed the wording; everything else is best-effort, sourced from a
third-party decompiled WhatsApp APK string dump (`github.com/GigaDroid/Decompiled-Whatsapp`), never
confirmed against real app output.

| Code | Language | Tested against a real export | Known issues |
|---|---|---|---|
| `nl` | Dutch | ✅ Yes | — |
| `en` | English | ❌ No | — |
| `fr` | French | ❌ No | — |
| `az` | Azerbaijani | ❌ No | "Removed" pattern has target-before-actor word order; multi-word names can be captured ambiguously |
| `ca` | Catalan | ❌ No | — |
| `cs` | Czech | ❌ No | — |
| `da` | Danish | ❌ No | — |
| `de` | German | ❌ No | — |
| `es` | Spanish | ❌ No | — |
| `et` | Estonian | ❌ No | — |
| `fi` | Finnish | ❌ No | — |
| `hr` | Croatian | ❌ No | — |
| `hu` | Hungarian | ❌ No | — |
| `id` | Indonesian | ❌ No | — |
| `it` | Italian | ❌ No | — |
| `lt` | Lithuanian | ❌ No | — |
| `lv` | Latvian | ❌ No | — |
| `ms` | Malay | ❌ No | No "subject changed" phrase available (detection-list only, doesn't affect member tracking) |
| `nb` | Norwegian Bokmål | ❌ No | — |
| `pl` | Polish | ❌ No | — |
| `pt` | Portuguese (Portugal) | ❌ No | — |
| `pt-BR` | Portuguese (Brazil) | ❌ No | Reuses Portugal's "group created" phrase; no Brazil-specific one found in the source |
| `ro` | Romanian | ❌ No | — |
| `sk` | Slovak | ❌ No | No "subject changed" phrase available (detection-list only, doesn't affect member tracking) |
| `sl` | Slovenian | ❌ No | — |
| `sq` | Albanian | ❌ No | — |
| `sv` | Swedish | ❌ No | — |
| `sw` | Swahili | ❌ No | — |
| `tl` | Tagalog | ❌ No | — |
| `tr` | Turkish | ❌ No | — |
| `uz` | Uzbek | ❌ No | No fixed verb between actor and target; multi-word names can be captured ambiguously |
| `vi` | Vietnamese | ❌ No | — |

If a language's detection looks wrong for a real export, that's expected for anything marked "No" above
— see `TODO.md` and the test fixtures folder for how to investigate and fix a specific language.

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
- **PIN lock** — an optional app-level PIN (Settings → Protection) gates access to the UI; the PIN and its recovery answer are hashed with Argon2, never stored or compared in plain text. This is a local access gate, not encryption — the underlying chat database on disk is unencrypted, so it doesn't protect against someone with direct filesystem access

## License

This project is private.

# WhatsApp Archive Viewer (PC)

A desktop application for viewing WhatsApp chat exports on Windows, macOS, and Linux.

## Features

- Import and view WhatsApp chat exports (ZIP files)
- View messages, media, and attachments
- Search and filter messages
- Customize chat backgrounds
- Edit contact profiles with notes and phone numbers
- Export chats

## Prerequisites

- Node.js 18+ and npm
- Rust toolchain (for Tauri)
- Platform-specific build tools:
  - **Windows**: Visual Studio C++ Build Tools
  - **macOS**: Xcode Command Line Tools
  - **Linux**: libwebkit2gtk-4.0-dev, build-essential, curl, wget, file, libssl-dev, libayatana-appindicator3-dev, librsvg2-dev

## Installation

1. Navigate to the project directory:
```bash
cd whatsapp-archive-viewer-pc
```

2. Install dependencies:
```bash
npm install
```

3. Install Tauri CLI (if not already installed):
```bash
npm install -g @tauri-apps/cli
```

Alternatively, run the setup script from the parent directory:
```bash
cd ..
.\setup.ps1
```

## Development

Run the development server:
```bash
cd whatsapp-archive-viewer-pc
npm run tauri:dev
```

## Building

Build for your current platform:
```bash
cd whatsapp-archive-viewer-pc
npm run tauri:build
```

The built application will be in the `src-tauri/target/release/bundle/` directory.

## Testing

Run frontend tests:
```bash
cd whatsapp-archive-viewer-pc
npm test
```

Run backend (Rust) tests:
```bash
cd whatsapp-archive-viewer-pc
cd src-tauri
cargo test
```

## Platform-Specific Notes

### Windows
- Builds NSIS installer by default
- Includes VC++ redistributable for compatibility

### macOS
- Builds DMG installer by default
- Requires code signing for distribution

### Linux
- Builds AppImage by default
- May require additional system dependencies

## Project Structure

- `src/` - React frontend
- `src-tauri/` - Rust backend (Tauri)
- `public/` - Static assets

## License

This project is private.

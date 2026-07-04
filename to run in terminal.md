# Windows

## Prerequisites
- Install Node.js (LTS version recommended)
- Install Rust: https://rustup.rs/

## Running the app
```powershell
cd whatsapp-archive-viewer-pc\project-code
npm install
npm run tauri:dev
```

# Linux

## Prerequisites
- Install Node.js (LTS version recommended)
- Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

## Running the app
```bash
cd whatsapp-archive-viewer-pc/project-code
npm install --no-bin-links
node node_modules/@tauri-apps/cli/tauri.js dev
```
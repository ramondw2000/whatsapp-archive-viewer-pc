// windows:
cd whatsapp-archive-viewer-pc\project-code
npm install
npm run tauri:dev

// linux:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
cd '/media/ramon.dewilde/USB-station/programming/whatsapp archive app stuff/whatsapp-archive-viewer-pc/project-code'
npm install --no-bin-links
node node_modules/@tauri-apps/cli/tauri.js dev
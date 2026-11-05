# How to Start the Application

## First-Time Setup (New Machine)

Run the setup script to automatically install all dependencies:

```powershell
Start-Process powershell -Verb RunAs -ArgumentList '-ExecutionPolicy', 'Bypass', '-File', 'D:\programming\whatsapp-archive-viewer-pc\setup.ps1'
```

This installs:
- **Node.js** (LTS via winget)
- **Rust** (stable MSVC toolchain)
- **Visual Studio C++ Build Tools** (required by Rust on Windows)
- **npm dependencies**

If everything is already installed, the script will confirm and exit.

> **Note:** The script must run as Administrator (it will prompt for UAC). After setup, restart your terminal before running the app.

## Development Mode

To run the app in development mode:

```powershell
cd D:\programming\whatsapp-archive-viewer-pc\project-code
try { taskkill /F /IM node.exe } catch {}; npm run tauri dev
```

This command will:
- Terminate any running `node.exe` processes (to free up ports)
- Build the Tauri application
- Launch the app in development mode

## Building Installer

This command creates the Windows installer (.exe):

```powershell
cd D:\programming\whatsapp-archive-viewer-pc\project-code
npx tauri build
```

The installer will be generated at:
`D:\cargo-target\whatsapp-archive-viewer-pc\release\bundle\nsis\WhatsApp Archive Viewer (PC)_0.1.0_x64-setup.exe`

## Troubleshooting

**Port already in use error:**
- Use the quick start command above - it automatically kills existing Node processes
- If the error persists, wait 10-20 seconds for the port to be released

**Build errors / `link.exe` not found:**
- Run `setup.ps1` as Administrator — it will detect and fix missing dependencies
- If the error persists, open Visual Studio Installer manually, click **Modify** on your installation, and check **Desktop development with C++**
- Restart your terminal after installing
- Clear the build cache: `npm run tauri clean`

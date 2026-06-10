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

## Android Setup (First Time)

Run the Android setup script as Administrator to install all Android build requirements:

```powershell
Start-Process powershell -Verb RunAs -ArgumentList "-NoExit", "-Command", "Set-Location 'D:\programming\whatsapp-archive-viewer-pc'; .\setup-android.ps1"
```

This installs:
- **Microsoft OpenJDK 17**
- **Android Studio** (via winget)
- **Android NDK, CMake, Command-line Tools** (via Android Studio SDK Manager)
- **Rust Android targets** (aarch64, armv7, i686, x86_64)

After the script finishes, initialize the Android project (one-time only):

```powershell
cd D:\programming\whatsapp-archive-viewer-pc\project-code
npx tauri android init
```

> **Note:** Before running the script, open Android Studio once and complete the setup wizard so the Android SDK is downloaded. Then re-run the script.

## Building Android APK

Use the build script (workaround for a Windows symlink limitation with `npx tauri android build`):

```powershell
cd D:\programming\whatsapp-archive-viewer-pc
.\build-android.ps1
```

This script:
1. Builds the frontend
2. Compiles Rust for all Android targets
3. Copies `.so` files into place (bypassing the symlink step)
4. Runs Gradle to produce the APK

The APK will be generated at:
`whatsapp-archive-viewer-pc\project-code\src-tauri\gen\android\app\build\outputs\apk\universal\release\app-universal-release-unsigned.apk`

To install on your phone:
1. Copy the APK to your phone
2. Enable **Install from unknown sources** in phone settings
3. Tap the APK to install

## Run on Android Device / Emulator

```powershell
cd D:\programming\whatsapp-archive-viewer-pc\project-code
npx tauri android dev
```

Requires a physical Android phone with **USB debugging enabled**, or an Android emulator running in Android Studio.

## Troubleshooting

**Port already in use error:**
- Use the quick start command above - it automatically kills existing Node processes
- If the error persists, wait 10-20 seconds for the port to be released

**Build errors / `link.exe` not found:**
- Run `setup.ps1` as Administrator — it will detect and fix missing dependencies
- If the error persists, open Visual Studio Installer manually, click **Modify** on your installation, and check **Desktop development with C++**
- Restart your terminal after installing
- Clear the build cache: `npm run tauri clean`

**Android symlink error (`Onjuiste functie` / os error 1):**
- `npx tauri android build` fails on Windows because it tries to create a cross-location symlink
- Use `build-android.ps1` instead — it copies the `.so` files directly and calls Gradle
- The Rust `.so` is compiled to `D:\cargo-target\whatsapp-archive-viewer-pc\<target>\release\`

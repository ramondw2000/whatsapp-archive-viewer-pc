# WhatsApp Archive Viewer - Setup Script
# Run this script as Administrator to install all dependencies
# Usage: Right-click -> Run with PowerShell (as Admin)
#   OR:  Start-Process powershell -Verb RunAs -ArgumentList '-File', '.\setup.ps1'

$ErrorActionPreference = "Stop"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " WhatsApp Archive Viewer - Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if running as admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
    Write-Host "ERROR: This script must be run as Administrator!" -ForegroundColor Red
    Write-Host "Right-click PowerShell -> Run as Administrator, then run this script again." -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

# --- Quick check: is everything already installed? ---
$allGood = $true
$hasNode = [bool](Get-Command node -ErrorAction SilentlyContinue)
$hasRust = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
$vsWherePath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasCppTools = $false
if (Test-Path $vsWherePath) {
    $cppPath = & $vsWherePath -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($cppPath) { $hasCppTools = $true }
}
$projectDir = Join-Path $PSScriptRoot "project-code"
$hasNodeModules = Test-Path (Join-Path $projectDir "node_modules")

if ($hasNode -and $hasRust -and $hasCppTools -and $hasNodeModules) {
    Write-Host "All dependencies are already installed!" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Node.js:    $(& node --version)" -ForegroundColor Green
    Write-Host "  Rust:       $(& rustc --version)" -ForegroundColor Green
    Write-Host "  C++ Tools:  $cppPath" -ForegroundColor Green
    Write-Host "  npm deps:   installed" -ForegroundColor Green
    Write-Host ""
    Write-Host "To start the app:" -ForegroundColor Cyan
    Write-Host "  cd project-code" -ForegroundColor White
    Write-Host "  npm run tauri dev" -ForegroundColor White
    Write-Host ""
    Read-Host "Press Enter to exit"
    exit 0
}

# --- 1. Check/Install Node.js ---
Write-Host "[1/4] Checking Node.js..." -ForegroundColor Yellow
$node = Get-Command node -ErrorAction SilentlyContinue
if ($node) {
    $nodeVersion = & node --version
    Write-Host "  Node.js $nodeVersion found." -ForegroundColor Green
} else {
    Write-Host "  Node.js not found. Installing via winget..." -ForegroundColor Yellow
    winget install -e --id OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ERROR: Failed to install Node.js. Please install manually from https://nodejs.org" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
    # Refresh PATH
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
    Write-Host "  Node.js installed." -ForegroundColor Green
}

# --- 2. Check/Install Rust ---
Write-Host "[2/4] Checking Rust..." -ForegroundColor Yellow
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
if ($rustc) {
    $rustVersion = & rustc --version
    Write-Host "  $rustVersion found." -ForegroundColor Green
} else {
    Write-Host "  Rust not found. Installing with MSVC toolchain..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" --default-host x86_64-pc-windows-msvc --default-toolchain stable -y
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ERROR: Failed to install Rust. Please install manually from https://rustup.rs" -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
    # Refresh PATH
    $env:PATH = [System.Environment]::GetEnvironmentVariable("PATH", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("PATH", "User")
    Write-Host "  Rust installed." -ForegroundColor Green
}

# --- 3. Install Visual Studio C++ Build Tools ---
Write-Host "[3/4] Checking Visual Studio C++ Build Tools..." -ForegroundColor Yellow

$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsInstaller = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vs_installer.exe"
$hasCpp = $false
$vsPath = $null

if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($vsPath) {
        $hasCpp = $true
    }
}

if ($hasCpp) {
    Write-Host "  Visual Studio C++ tools found at: $vsPath" -ForegroundColor Green
} else {
    Write-Host "  C++ tools not found. Installing..." -ForegroundColor Yellow
    Write-Host "  This may take several minutes (3-6 GB download)..." -ForegroundColor Gray

    if (Test-Path $vsWhere) {
        # VS installer exists - find ANY existing VS installation and add C++ to it
        $existingPath = & $vsWhere -latest -products * -property installationPath 2>$null
        $productId = & $vsWhere -latest -products * -property productId 2>$null
        if ($existingPath) {
            Write-Host "  Found existing VS installation at: $existingPath" -ForegroundColor Gray
            Write-Host "  Product: $productId" -ForegroundColor Gray
            # Use NativeDesktop for Community/Professional/Enterprise, VCTools for BuildTools
            if ($productId -match "BuildTools") {
                $workload = "Microsoft.VisualStudio.Workload.VCTools"
            } else {
                $workload = "Microsoft.VisualStudio.Workload.NativeDesktop"
            }
            Write-Host "  Adding workload '$workload'..." -ForegroundColor Gray
            & $vsInstaller modify --installPath "$existingPath" --add $workload --includeRecommended --passive
            Write-Host "  Waiting for VS Installer to finish..." -ForegroundColor Gray
            Write-Host "  (Watch the VS Installer window - it may take several minutes)" -ForegroundColor Gray
            # Wait for the installer process to finish
            do {
                Start-Sleep -Seconds 5
                $vsProc = Get-Process -Name "setup" -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "*Visual Studio*" }
            } while ($vsProc)
        } else {
            Write-Host "  VS installer found but no installation. Installing Build Tools..." -ForegroundColor Gray
            & $vsInstaller install --channelId VisualStudio.17.Release --productId Microsoft.VisualStudio.Product.BuildTools --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive
        }
    } else {
        # No VS installer at all - download and install Build Tools from scratch
        Write-Host "  Downloading Visual Studio Build Tools installer..." -ForegroundColor Gray
        $vsUrl = "https://aka.ms/vs/17/release/vs_BuildTools.exe"
        $vsExe = "$env:TEMP\vs_BuildTools.exe"
        Invoke-WebRequest -Uri $vsUrl -OutFile $vsExe
        & $vsExe --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --passive
        Remove-Item $vsExe -ErrorAction SilentlyContinue
    }

    # Verify installation succeeded
    $vsPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($vsPath) {
        Write-Host "  Visual Studio C++ tools installed at: $vsPath" -ForegroundColor Green
    } else {
        Write-Host "  WARNING: C++ tools installation may have failed." -ForegroundColor Red
        Write-Host "  Manual fix:" -ForegroundColor Yellow
        Write-Host "    1. Open Visual Studio Installer" -ForegroundColor Gray
        Write-Host "    2. Click 'Modify' on your VS installation" -ForegroundColor Gray
        Write-Host "    3. Check 'Desktop development with C++'" -ForegroundColor Gray
        Write-Host "    4. Click 'Modify' to install" -ForegroundColor Gray
        Read-Host "Press Enter to continue anyway"
    }
}

# --- 4. Install npm dependencies ---
Write-Host "[4/4] Installing npm dependencies..." -ForegroundColor Yellow
$projectDir = Join-Path $PSScriptRoot "project-code"
if (-not (Test-Path (Join-Path $projectDir "package.json"))) {
    Write-Host "  ERROR: Cannot find project-code/package.json" -ForegroundColor Red
    Write-Host "  Make sure this script is in the project root directory." -ForegroundColor Yellow
    Read-Host "Press Enter to exit"
    exit 1
}

Push-Location $projectDir
try {
    & npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  ERROR: npm install failed." -ForegroundColor Red
        Read-Host "Press Enter to exit"
        exit 1
    }
    Write-Host "  npm dependencies installed." -ForegroundColor Green
} finally {
    Pop-Location
}

# --- Done ---
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host " Setup Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
Write-Host "To start the app:" -ForegroundColor Cyan
Write-Host "  cd project-code" -ForegroundColor White
Write-Host "  npm run tauri dev" -ForegroundColor White
Write-Host ""
Write-Host "NOTE: You may need to restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
Write-Host ""
Read-Host "Press Enter to exit"

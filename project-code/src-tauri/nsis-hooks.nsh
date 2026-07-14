; ---------------------------------------------------------------------------
; Install/uninstall hooks
; ---------------------------------------------------------------------------

!macro NSIS_HOOK_PREINSTALL
  ; If a previous version is installed, silently uninstall it first
  ReadRegStr $0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\WhatsApp Archive Viewer (PC)" "UninstallString"
  ${If} $0 != ""
    ExecWait '"$0" /S _?=$INSTDIR'
  ${EndIf}
  ReadRegStr $0 HKCU "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\WhatsApp Archive Viewer (PC)" "UninstallString"
  ${If} $0 != ""
    ExecWait '"$0" /S _?=$INSTDIR'
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Start Menu shortcut (Tauri has no built-in checkbox for this)
  CreateDirectory "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"
  CreateShortcut "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk" "$INSTDIR\whatsapp-archive-viewer-pc.exe"

  ; Check if VC++ Redistributable is already installed
  ReadRegDWord $1 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
  ${If} $1 == 1
    DetailPrint "Visual C++ Redistributable already installed, skipping."
  ${Else}
    ; Ask the user before installing
    MessageBox MB_YESNO "WhatsApp Archive Viewer (PC) requires the Microsoft Visual C++ Redistributable, which is not currently installed.$\r$\n$\r$\nInstall it now?" IDNO skip_vcredist_install

    ; Run the real installer UI (no /quiet) so the user gets a genuine progress bar + Cancel button
    ExecWait '"$INSTDIR\vc_redist.x64.exe" /norestart' $2

    ; Re-check the registry to confirm it actually completed
    ReadRegDWord $1 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
    ${If} $1 != 1
      MessageBox MB_ICONEXCLAMATION|MB_OK "The Visual C++ Redistributable install was not completed.$\r$\nWhatsApp Archive Viewer (PC) requires it to run, so the installation will now be rolled back."
      ExecWait '"$INSTDIR\uninstall.exe" /S'
      Quit
    ${EndIf}
    Goto vcredist_done

    skip_vcredist_install:
    MessageBox MB_ICONEXCLAMATION|MB_OK "WhatsApp Archive Viewer (PC) may not run correctly without the Visual C++ Redistributable.$\r$\nYou can install it later by re-running this installer."

    vcredist_done:
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Clean up desktop shortcut
  Delete "$DESKTOP\WhatsApp Archive Viewer (PC).lnk"

  ; Clean up Start Menu folder
  Delete "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk"
  RMDir "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"

  ; The app stores its data under %APPDATA%\WhatsAppArchiveViewer, not under
  ; the Tauri bundle identifier folder the built-in "Delete app data" logic
  ; targets — remove it here when the user opted in.
  ${If} $DeleteAppDataCheckboxState == 1
    RmDir /r "$APPDATA\WhatsAppArchiveViewer"
  ${EndIf}
!macroend
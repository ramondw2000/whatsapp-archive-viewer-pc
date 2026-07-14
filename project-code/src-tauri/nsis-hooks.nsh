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
    ; Run the real installer UI (no /quiet) so the user gets a genuine progress bar + Cancel button
    ExecWait '"$INSTDIR\vc_redist.x64.exe" /norestart' $2

    ; Re-check the registry to confirm it actually completed
    ReadRegDWord $1 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"
    ${If} $1 != 1
      MessageBox MB_ICONEXCLAMATION|MB_OK "The Visual C++ Redistributable install was not completed.$\r$\nWhatsApp Archive Viewer (PC) requires it to run, so the installation will now be rolled back."
      ExecWait '"$INSTDIR\uninstall.exe" /S'
      Quit
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Clean up desktop shortcut
  Delete "$DESKTOP\WhatsApp Archive Viewer (PC).lnk"

  ; Clean up Start Menu folder
  Delete "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk"
  RMDir "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"
!macroend
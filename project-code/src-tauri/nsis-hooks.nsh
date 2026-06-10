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
  ; Desktop shortcut is handled by Tauri's built-in "Create desktop shortcut" finish page checkbox
  ; Start Menu shortcut (Tauri has no built-in checkbox for this)
  CreateDirectory "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"
  CreateShortcut "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk" "$INSTDIR\whatsapp-archive-viewer-pc.exe"
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Clean up desktop shortcut
  Delete "$DESKTOP\WhatsApp Archive Viewer (PC).lnk"

  ; Clean up Start Menu folder
  Delete "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk"
  RMDir "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"
!macroend

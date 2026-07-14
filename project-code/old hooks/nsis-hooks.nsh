; Global variable for VC++ Redistributable uninstall checkbox
Var /GLOBAL UninstallVCRedist

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

  ; Show message before installing VC++ Redistributable
  MessageBox MB_OK "Installing Microsoft Visual C++ Redistributable...$\r$\nThis may take a moment. Please wait."

!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; Desktop shortcut is handled by Tauri's built-in "Create desktop shortcut" finish page checkbox
  ; Start Menu shortcut (Tauri has no built-in checkbox for this)
  CreateDirectory "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"
  CreateShortcut "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk" "$INSTDIR\whatsapp-archive-viewer-pc.exe"
!macroend

; Custom uninstall page for VC++ Redistributable checkbox
!macro NSIS_HOOK_PREUNINSTALL
  Page custom nsis_uninstall_vc_redist_page "" "" 3
!macroend

Function nsis_uninstall_vc_redist_page
  nsDialogs::Create 1018
  Pop $0

  ${NSD_CreateCheckbox} 0 0 100% 12u "Uninstall VC++ Redistributable (may affect other applications)"
  Pop $1
  ${NSD_SetState} $1 ${BST_UNCHECKED}
  ${NSD_OnClick} $1 nsis_uninstall_vc_redist_onclick

  nsDialogs::Show
FunctionEnd

Function nsis_uninstall_vc_redist_onclick
  Pop $1
  ${NSD_GetState} $1 $UninstallVCRedist
FunctionEnd

!macro NSIS_HOOK_POSTUNINSTALL
  ; Clean up desktop shortcut
  Delete "$DESKTOP\WhatsApp Archive Viewer (PC).lnk"

  ; Clean up Start Menu folder
  Delete "$SMPROGRAMS\WhatsApp Archive Viewer (PC)\WhatsApp Archive Viewer (PC).lnk"
  RMDir "$SMPROGRAMS\WhatsApp Archive Viewer (PC)"

  ; Uninstall VC++ Redistributable if checkbox was checked
  ${If} $UninstallVCRedist == "1"
    ExecWait '"$INSTDIR\vc_redist.x64.exe" /uninstall /quiet /norestart'
  ${EndIf}
!macroend

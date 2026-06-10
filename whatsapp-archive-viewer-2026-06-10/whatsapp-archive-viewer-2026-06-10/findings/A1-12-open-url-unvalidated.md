# A1-12 — open_url: unvalidated URL/scheme passed to opener

**Severity:** Medium
**CWE:** CWE-601
**Status:** Open
**Location:** src-tauri/src/lib.rs:3513
**Vuln class:** Open-redirect / dangerous-scheme launch via opener

## Exploit sketch
`open_url(url)` passes a fully renderer-controlled string to `tauri_plugin_opener::open_url`. Map links extracted from chat content (`extract_maps_url`) and other URLs flow here. A non-http scheme (e.g. `file://`, on some platforms a custom protocol handler, or a `javascript:`/`smb:`/`ms-msdt:`-style URI) can be launched via the OS handler. No scheme allowlist (http/https) is enforced.

## Why this matters
Opening attacker-chosen URIs with the system handler can reach local files or registered protocol handlers, a known vector for code execution / SSRF-to-local on desktop. Severity Medium because it requires the malicious URL to reach the call (chat content / IPC), which is attacker-controllable here.

## Evidence
```
3512: #[tauri::command]
3513: fn open_url(url: String) -> Result<(), String> {
3514:     tauri_plugin_opener::open_url(&url, None::<&str>)
3515:         .map_err(|e| format!("Failed to open URL: {}", e))
3516: }
```
`open_maps`/`open_vcard_whatsapp` also call open_url with constructed URLs (3504, etc.).

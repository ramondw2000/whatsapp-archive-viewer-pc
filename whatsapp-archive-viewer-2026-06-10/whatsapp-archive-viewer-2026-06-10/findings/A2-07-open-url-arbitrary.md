# A2-07 — `open_url` opens any frontend-supplied URI via OS opener

**Severity:** Medium
**CWE:** CWE-601
**Status:** Open
**Location:** src-tauri/src/lib.rs:3512
**Vuln class:** Unvalidated URI / argument-injection into OS handler

## Exploit sketch
`open_url(url)` passes an arbitrary frontend string straight to
`tauri_plugin_opener::open_url`. A webview foothold can invoke
`open_url("file:///...")` or a custom-scheme/`smb://`/`javascript:`-style URI, or a
crafted `whatsapp://`/protocol-handler payload, triggering external apps. `open_whatsapp`
similarly builds `whatsapp://send?phone={phone}` from vCard data, though phone is digit-filtered.

## Why this matters
Lets script in the webview drive the OS URL/protocol handlers (open arbitrary local files /
launch registered protocol handlers), aiding phishing or local-app abuse. No scheme allow-list.

## Evidence
```
3512:#[tauri::command]
3513:fn open_url(url: String) -> Result<(), String> {
3514:    tauri_plugin_opener::open_url(&url, None::<&str>)
```
The capability also grants `opener:allow-open-url` and `opener:allow-open-path`. Add a
scheme allow-list (http/https only).

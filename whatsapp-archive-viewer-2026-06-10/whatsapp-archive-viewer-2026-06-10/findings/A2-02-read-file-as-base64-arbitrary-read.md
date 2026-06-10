# A2-02 — `read_file_as_base64` is an unrestricted arbitrary-file-read IPC primitive

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:5198
**Vuln class:** Path traversal / arbitrary file read via IPC

## Exploit sketch
Frontend invokes `read_file_as_base64({ path: "/etc/passwd" })` (or any absolute path,
or `../../`). The command does `fs::read(&path)` with zero validation, base64-encodes the
bytes and returns them. Any webview script foothold reads any file the process can access.

## Why this matters
Direct arbitrary-read primitive reachable from JS with no scope check; pairs with any
XSS / supply-chain script to exfiltrate secrets. Worse than the asset-protocol path because
it returns raw bytes to JS regardless of mime/extension.

## Evidence
```
5198:async fn read_file_as_base64(path: String) -> Result<String, String> {
5200:    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
5201:    let b64 = base64_encode(&bytes);
...
5220:    Ok(format!("data:{};base64,{}", mime, b64))
```
No allow-list, no canonicalization, no confinement to app data dir.

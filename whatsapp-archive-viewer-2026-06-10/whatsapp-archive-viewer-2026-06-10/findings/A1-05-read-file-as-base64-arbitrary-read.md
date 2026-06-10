# A1-05 — read_file_as_base64: unrestricted arbitrary file read by absolute path

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:5199
**Vuln class:** Arbitrary file read (no path scoping at all)

## Exploit sketch
`read_file_as_base64(path)` takes a fully attacker-controlled path from the renderer and does `fs::read(&path)` with zero validation, returning the bytes as a base64 data URL. Any renderer code (including content rendered from a malicious archive, or any XSS) calls `invoke('read_file_as_base64', { path: '/home/victim/.ssh/id_rsa' })`.

## Why this matters
Direct, no-tricks-needed arbitrary file disclosure of anything the app user can read. The MIME defaults to image/jpeg but the raw bytes are fully recoverable from base64.

## Evidence
```
5198: #[tauri::command]
5199: async fn read_file_as_base64(path: String) -> Result<String, String> {
5200:     let bytes = fs::read(&path).map_err(|e| e.to_string())?;
5201:     let b64 = base64_encode(&bytes);
```

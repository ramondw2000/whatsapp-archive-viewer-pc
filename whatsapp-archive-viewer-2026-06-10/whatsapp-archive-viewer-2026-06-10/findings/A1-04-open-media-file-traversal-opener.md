# A1-04 — open_media_file: path traversal into tauri-plugin-opener (open arbitrary file)

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:3532
**Vuln class:** Path traversal reaching open_path (arbitrary file/handler launch)

## Exploit sketch
`open_media_file(chat_id, filename)` strips only Unicode marks then `join(filename)`; with `filename = "../../../../../../tmp/evil.desktop"` (or a `.html`, `.pdf`, `.exe` planted earlier via A1-01) the resolved path escapes the media dir. If it `exists()`, it is handed to `tauri_plugin_opener::open_path`, which launches it with the OS default handler.

## Why this matters
Opening an attacker-controlled file with the system handler can trigger code execution (e.g. `.desktop`/`.lnk`/script/installer). Chains cleanly with the Zip-Slip write primitive (A1-01) to drop-then-launch.

## Evidence
```
3520: fn open_media_file(chat_id: String, filename: String) -> Result<(), String> {
3524:     let filename: String = filename.chars().filter(|c| !matches!(*c, ... )).collect();
3532:     let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);
3534:     if !media_path.exists() {
3536:         return Err(format!("File not found: {:?}", media_path));
3542:     tauri_plugin_opener::open_path(media_path.to_string_lossy().as_ref(), None::<&str>)
```

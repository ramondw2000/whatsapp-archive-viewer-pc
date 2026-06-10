# A3-02 — "Open" attachment passes attacker-controlled filename to open_media_file (path traversal → arbitrary file launch)

**Severity:** High
**CWE:** CWE-22 (Path Traversal) → CWE-749 (exposed dangerous method: open_path)
**Status:** Confirmed
**Location:** project-code/src/App.tsx:1461 and :648 (frontend sinks); backend open_media_file at project-code/src-tauri/src/lib.rs:3520
**Vuln class:** Frontend half of backend path-traversal; archive-controlled filename → open_path

## Exploit sketch
For a file attachment, the "Open" button calls `invoke("open_media_file", { chatId, filename: msg.media! })`. `msg.media`/filename is taken verbatim from the WhatsApp archive (attacker-controlled). The backend does `app_data.join("chats").join(&chat_id).join("media").join(&filename)` with only a bidi-character strip and an `exists()` check — no traversal sanitization. A filename of `../../../../../../etc/passwd` (or `..\\..\\..\\Windows\\...\\malicious.exe`) escapes the media dir, and `tauri_plugin_opener::open_path` launches it with the system default app. The vCard "Add Contact" path (line 648) reaches the same command with the same unvalidated filename.

## Why this matters
A crafted archive can make the user open arbitrary local files / executables outside the sandboxed media directory with one click, enabling local code execution or leaking sensitive files into a viewer.

## Evidence
```
src/App.tsx
1461:  invoke("open_media_file", { chatId, filename: msg.media! })
 648:  invoke("open_media_file", { chatId, filename: vcardFilename })

src-tauri/src/lib.rs
3520: fn open_media_file(chat_id: String, filename: String) -> Result<(), String> {
3524:   let filename: String = filename.chars().filter(|c| !matches!(*c, ... bidi ...)).collect();
3532:   let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);
3534:   if !media_path.exists() { return Err(...) }
3542:   tauri_plugin_opener::open_path(media_path.to_string_lossy().as_ref(), None::<&str>)
```
No check for `..`, absolute paths, or canonicalized containment within the media dir.

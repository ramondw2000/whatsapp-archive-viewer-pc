# A1-02 — Path traversal -> arbitrary file read via get_media_as_base64 / get_media_with_dims

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:2771
**Vuln class:** Path traversal on user-controlled filename (file read)

## Exploit sketch
Frontend (attacker-influenced via crafted archive content, since media filenames originate from the chat `.txt` and are passed straight back into these IPC calls) invokes `get_media_as_base64(chat_id, filename)`. `filename` is only stripped of Unicode bidi/zero-width marks — `..` and `/` survive. `app_data.join("chats").join(chat_id).join("media").join("../../../../../../etc/passwd")` resolves outside the media dir; if the path exists it is read and returned base64-encoded. `chat_id` is equally unsanitized (e.g. `../../../..`).

## Why this matters
Arbitrary file disclosure of any file readable by the app user (SSH keys, browser data, other apps' secrets), exfiltrated to the renderer which can ship it out over `connect-src 'self'`/unpkg.

## Evidence
```
2763: let filename_clean: String = filename.chars().filter(|c| !matches!(*c,
2765:     '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
2767:     '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
2769: )).collect();
2771: let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename_clean);
2775: let bytes = if media_path.exists() {
2776:     fs::read(&media_path).map_err(|e| e.to_string())?
```
Identical sink in `get_media_with_dims` at line 2918, and `parse_vcard` at 3321, `open_vcard_whatsapp` at 3417, `get_media_path` at 3556.

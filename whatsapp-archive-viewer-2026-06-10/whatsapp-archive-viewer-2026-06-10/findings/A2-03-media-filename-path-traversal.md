# A2-03 — Media commands join unsanitized `filename` → path traversal read

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:2771 (also 2918, 3532, 3556)
**Vuln class:** Path traversal / arbitrary file read

## Exploit sketch
`get_media_as_base64(chat_id, filename, ..)`, `get_media_with_dims`, `open_media_file`,
`get_media_path` build `app_data/chats/<chat_id>/media/<filename>` via `PathBuf::join`.
`filename` is frontend-controlled and only stripped of Unicode direction marks — NOT of
`..` or absolute components. `filename = "../../../../etc/passwd"` (or an absolute path on
the joined chat_id) escapes the media dir; `Path::join` with an absolute component discards
the prefix entirely. Result is base64 of the target file returned to JS, or `open_path` on it.

## Why this matters
Arbitrary file read (and `open_media_file` → arbitrary file launch via the OS opener)
reachable from the webview. `try_extract_from_zip` then also caches the bytes at the
traversed `media_path`.

## Evidence
```
2763:    let filename_clean: String = filename.chars().filter(|c| !matches!(*c,
2765:        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | ...
2769:    )).collect();
2771:    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename_clean);
2775:    let bytes = if media_path.exists() { fs::read(&media_path)... }
```
`chat_id` is equally unsanitized; neither is canonicalized/confined.

# A1-08 — delete_chat / clear path traversal -> arbitrary directory removal

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:4140
**Vuln class:** Path traversal (recursive directory deletion)

## Exploit sketch
`delete_chat(chat_id)` builds `chats/<chat_id>` and `imports/<chat_id>` from a raw `chat_id` and calls `fs::remove_dir_all`. With `chat_id = "../../../../../../home/victim/Documents"` the path escapes and recursively deletes an arbitrary directory the app user owns (if it exists).

## Why this matters
Destructive arbitrary recursive deletion of user data from one IPC call. `chat_id` flows unvalidated from the frontend.

## Evidence
```
4136: fn delete_chat(chat_id: String) -> Result<(), String> {
4140:     let chat_dir = app_data.join("chats").join(&chat_id);
4142:     let import_dir = app_data.join("imports").join(&chat_id);
4156:     if chat_dir.exists() {
4158:         fs::remove_dir_all(&chat_dir).map_err(|e| e.to_string())?;
4164:     if import_dir.exists() {
4166:         fs::remove_dir_all(&import_dir).map_err(|e| e.to_string())?;
```

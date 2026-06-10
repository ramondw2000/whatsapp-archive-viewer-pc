# A1-07 — rename_media_file: path traversal on both source and destination

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:3213
**Vuln class:** Path traversal (arbitrary file move/rename)

## Exploit sketch
`rename_media_file(chat_id, idx, old_filename, new_filename)` joins both names onto the media dir with no sanitization. `old_filename = "../../../../etc/secret"`, `new_filename = "../../../../tmp/leak"` performs `fs::rename` of an arbitrary existing file to an attacker-chosen location (e.g. into the asset-readable media dir for later disclosure). Destination existence check only prevents clobber, not traversal.

## Why this matters
Move/exfiltrate arbitrary files the app user owns into a readable location, or relocate/destroy arbitrary files (rename across the FS). Combined with A1-02/A1-03 yields read-out of moved secrets.

## Evidence
```
3207: fn rename_media_file(chat_id: String, message_idx: i64, old_filename: String, new_filename: String) -> Result<(), String> {
3211:     let media_dir = app_data.join("chats").join(&chat_id).join("media");
3213:     let old_path = media_dir.join(&old_filename);
3215:     let new_path = media_dir.join(&new_filename);
3241:     fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename file: {}", e))?;
```

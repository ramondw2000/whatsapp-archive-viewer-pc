# A1-06 — import_zip_from_bytes: path traversal arbitrary file write

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:5719
**Vuln class:** Path traversal on user-controlled filename (file write)

## Exploit sketch
`import_zip_from_bytes(b64, filename)` decodes attacker bytes and writes them to `import_cache/<filename>` using the raw `filename`. With `filename = "../../../../../../home/victim/.config/autostart/x.desktop"` the write lands anywhere the app user can write. `fs::create_dir_all(parent)` even creates the traversed parent dirs. Content is fully attacker-controlled (the b64 payload).

## Why this matters
Arbitrary file write/overwrite with arbitrary content directly from a single IPC call — strongest primitive for planting executables/config → code execution.

## Evidence
```
5717: async fn import_zip_from_bytes(b64: String, filename: String) -> Result<Vec<String>, String> {
5718:     let bytes = base64_decode(&b64).map_err(|e| format!("Failed to decode base64: {}", e))?;
5719:     let cache_path = get_app_data_dir().join("import_cache").join(&filename);
5720:     if let Some(parent) = cache_path.parent() {
5721:         fs::create_dir_all(parent).map_err(|e| e.to_string())?;
5722:     }
5723:     fs::write(&cache_path, &bytes).map_err(|e| format!("Failed to write import cache: {}", e))?;
```

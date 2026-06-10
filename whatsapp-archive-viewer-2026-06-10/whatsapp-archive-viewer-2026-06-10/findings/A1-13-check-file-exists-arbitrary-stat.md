# A1-13 — check_file_exists: arbitrary filesystem existence probe

**Severity:** Low
**CWE:** CWE-22
**Status:** Open
**Location:** src-tauri/src/lib.rs:112
**Vuln class:** Information disclosure (path probing)

## Exploit sketch
`check_file_exists(path)` does `Path::new(&path).exists()` on a fully renderer-controlled absolute path. Lets the renderer enumerate arbitrary filesystem paths (presence of user files, installed software, usernames) with no restriction.

## Why this matters
File-system reconnaissance primitive that aids targeting of the read/write/open traversal bugs above. Low on its own (boolean oracle only).

## Evidence
```
111: #[tauri::command]
112: fn check_file_exists(path: String) -> bool {
113:     std::path::Path::new(&path).exists()
114: }
```

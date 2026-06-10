# A1-01 — Zip-Slip in import_chat ZIP extraction writes outside import dir

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:1275
**Vuln class:** Zip-Slip / path traversal on archive extraction

## Exploit sketch
A malicious WhatsApp `.zip` contains a non-`.txt` entry named e.g. `../../../../../../home/victim/.bashrc` (or on Windows `..\..\..\Users\victim\...`). In the ZIP branch the entry name is taken verbatim from `file.name()`, joined onto `import_dir`, the parent is `ensure_dir_exists`-created, then `File::create(&out_path)` + `io::copy` writes the attacker's content. No `enclosed_name()` / `..` sanitization is performed before `File::create`.

## Why this matters
Arbitrary file creation/overwrite anywhere the app user can write, achieved simply by getting the victim to open a crafted archive. Can plant or clobber config files, autostart entries, shell rc files → potential code execution.

## Evidence
```
1272: for i in 0..archive.len() {
1273:     let mut file = archive.by_index(i).map_err(|e| format!("ZIP extraction error: {}", e))?;
1274:     let name = file.name().to_string();
1275:     let out_path = import_dir.join(&name);
...
1299:     } else {
1300:         media_files.push(name.clone());
1301:         if let Some(parent) = out_path.parent() {
1302:             ensure_dir_exists(parent);
1303:         }
1304:         if let Ok(mut out_file) = File::create(&out_path) {
1305:             let _ = std::io::copy(&mut file, &mut out_file);
1306:         }
1307:     }
```
Note: the later copy-to-media phase (1333 `Path::file_name()`) flattens names, but the slip already occurred at 1304 in `import_dir`.

# A2-04 — Zip-slip: regular archive import writes entries to raw `name`

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:1275
**Vuln class:** Zip-slip / arbitrary file write via crafted archive

## Exploit sketch
In `import_chat_inner` the ZIP branch does `out_path = import_dir.join(&name)` where `name`
is the raw archive entry path, then `File::create(&out_path)` and copies bytes. A crafted
ZIP with an entry named `../../../../<home>/.bashrc` (or, on Windows, an absolute path)
escapes `import_dir` and writes attacker-controlled bytes anywhere the process can write.
The `7z`/`unrar` branches shell out with `-y` and also extract attacker-named paths.

## Why this matters
Arbitrary file write from an attacker-supplied archive → code execution (drop into autostart,
overwrite shell rc, plant DLL next to exe). The later "flatten to media/<filename>" copy does
NOT undo the initial raw-path write.

## Evidence
```
1272:        for i in 0..archive.len() {
1273:            let mut file = archive.by_index(i)...;
1274:            let name = file.name().to_string();
1275:            let out_path = import_dir.join(&name);
...
1304:                if let Ok(mut out_file) = File::create(&out_path) {
1305:                    let _ = std::io::copy(&mut file, &mut out_file);
```
No check that `out_path` stays within `import_dir`; `zip` crate's `enclosed_name()` is not used.
NOTE: the export-import path (lib.rs:5948+) DOES reject `filename.contains('/')`, so it is safe;
this regular-import path is the vulnerable one.

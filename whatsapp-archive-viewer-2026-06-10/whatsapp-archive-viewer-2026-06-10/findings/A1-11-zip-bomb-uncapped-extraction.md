# A1-11 — Uncapped archive extraction (zip bomb / decompression DoS)

**Severity:** Medium
**CWE:** CWE-409
**Status:** Open
**Location:** src-tauri/src/lib.rs:1305
**Vuln class:** Decompression bomb / resource exhaustion

## Exploit sketch
All extraction paths use `std::io::copy` / `read_to_end` with no per-entry or total size cap, no entry-count cap, and no free-space check. A malicious `.zip`/`.7z`/`.rar` (the latter shelled out to `7z`/`unrar` with `-y`) can be a decompression bomb (e.g. nested-deflate, sparse) that fills the disk or exhausts memory. TXT files are fully `read_to_end` into memory; media is streamed but unbounded in total. `import_zip_from_bytes` additionally base64-decodes the whole archive into memory first.

## Why this matters
A single opened archive can hard-fill the user's disk or OOM the app — denial of service / data-loss conditions on the user's machine.

## Evidence
```
1304: if let Ok(mut out_file) = File::create(&out_path) {
1305:     let _ = std::io::copy(&mut file, &mut out_file);   // no size limit
1306: }
...
1284: let mut txt_bytes = Vec::new();
1285: file.read_to_end(&mut txt_bytes)...                    // unbounded into RAM
```
No `MAX`/size-limit constants exist anywhere in lib.rs (grep for size caps returned only cache-capacity).

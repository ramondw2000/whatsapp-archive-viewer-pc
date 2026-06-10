# A1-09 — extract_file_from_zip / try_extract_from_zip: traversal write + 7z/unrar passthrough

**Severity:** Medium
**CWE:** CWE-22
**Status:** Open
**Location:** src-tauri/src/lib.rs:3788
**Vuln class:** Path traversal on filename used as output path and as external-tool arg

## Exploit sketch
`extract_file_from_zip(chat_id, filename)` builds `media_path = chats/<chat_id>/media/<filename>` (raw filename → traversal of the write target in the ZIP branch, `File::create(&media_path)` at 3878). For 7z/rar branches the raw `filename` is passed as a CLI argument to `7z x ... <filename>` / `unrar x ... <filename>` (3808/3836); a crafted entry name or argument-like value influences extraction. Match logic `name.contains(&filename)` (3876) is also loose. Same write-target pattern in `try_extract_from_zip` (2873 `File::create(media_path)` with media_path derived from raw frontend filename in callers A1-02).

## Why this matters
Arbitrary file write outside media dir and loose archive-entry matching; 7z/unrar arg passing widens the surface. Lower severity than A1-06 because output dir is `-o<media parent>` for 7z, but the ZIP-branch `File::create(&media_path)` honors `..` in filename.

## Evidence
```
3788: let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);
3876: if name.contains(&filename) || name.ends_with(&filename) {
3878:     let mut out_file = File::create(&media_path).map_err(|e| format!("Failed to create file: {}", e))?;
3880:     std::io::copy(&mut zipfile, &mut out_file)...
```

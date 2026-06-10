# A1-10 — import_from_export: Windows backslash zip-slip (only `/` blocked)

**Severity:** Medium
**CWE:** CWE-22
**Status:** Open
**Location:** src-tauri/src/lib.rs:5950
**Vuln class:** Incomplete path-separator sanitization on extraction

## Exploit sketch
The export-import extractor guards with `!filename.contains('/')` after stripping the `<folder>/media/` prefix. On Windows, `\` is also a path separator but is NOT rejected. An export ZIP entry like `Chat/media/..\..\..\evil.exe` (the slice after `media/` = `..\..\..\evil.exe`) passes the `'/'` check, then `chat_dir.join("media").join(filename)` traverses out on Windows. Also note the entry `name` matching on `/meta.json` is fine, but the media/custom write trusts the post-prefix remainder.

## Why this matters
On the Windows build (the primary target — NSIS, vc_redist, winreg) a crafted export ZIP escapes the chat dir to write arbitrary files. Cross-platform `/`-traversal is blocked, hence Medium not High.

## Evidence
```
5948: if entry_name.starts_with(&media_prefix) && entry_name.len() > media_prefix.len() {
5949:     let filename = &entry_name[media_prefix.len()..];
5950:     if !filename.contains('/') && !filename.is_empty() {
5951:         let dst = chat_dir.join("media").join(filename);
5952:         let mut out = File::create(&dst).map_err(|e| e.to_string())?;
```
Same pattern in the merge branch at 5885-5901.

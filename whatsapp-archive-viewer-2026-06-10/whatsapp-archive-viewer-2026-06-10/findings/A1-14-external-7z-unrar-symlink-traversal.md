# A1-14 — 7z/unrar shelled out with -y: symlink & path traversal delegated to external tool

**Severity:** Medium
**CWE:** CWE-22
**Status:** Open
**Location:** src-tauri/src/lib.rs:1161
**Vuln class:** Archive extraction traversal / symlink extraction via external command

## Exploit sketch
For `.7z` and `.rar` the app runs `7z x <archive> -o<import_dir> -y` (1161) and `unrar x <archive> <import_dir> -y` (1214) with no traversal/symlink guards. `7z` will extract stored symlinks by default (no `-snl-` control) and both tools can be coaxed into writing traversed paths depending on version/flags; `-y` auto-confirms overwrites. The app does not validate extracted entry names for the 7z/rar branches (unlike the in-process ZIP loop, which is itself vulnerable per A1-01). Output dir is the chat-scoped import_dir, but a symlink entry pointing at `/etc` plus a follow-up write enables escape.

## Why this matters
On systems with `7z`/`unrar` installed, a crafted archive can plant symlinks or traversed files outside the import dir, or overwrite existing files via `-y`. Depends on external tool behavior/version, hence Medium/Open.

## Evidence
```
1161: let output = std::process::Command::new("7z")
1163:     .arg("x")
1165:     .arg(&zip_path)
1167:     .arg(format!("-o{}", import_dir.to_string_lossy()))
1169:     .arg("-y")
...
1214: std::process::Command::new("unrar").arg("x").arg(&zip_path)
1220:     .arg(&import_dir.to_string_lossy().to_string()).arg("-y")
```

# ORCH-01 — Click-to-execute code execution from a malicious archive (no traversal required)

**Severity:** Critical
**CWE:** CWE-94 (Code Injection) / CWE-676 (Use of dangerous OS-handler launch) / CWE-434 (Unrestricted file type)
**Status:** Confirmed
**Location:** src-tauri/src/lib.rs:3528 (`open_media_file`) + :1305 (extraction `File::create(&out_path)`) + :3111 (`extract_maps_url`) + src/App.tsx (Open / Open Maps buttons)
**Vuln class:** RCE chain / arbitrary code execution

## Exploit sketch
Orchestrator synthesis of A1-01, A1-04, A2-04, A3-01, A3-02 — the worst-case the per-slice agents each only half-saw.
1. Attacker crafts a WhatsApp export ZIP. A media entry is named `Invoice.pdf.exe` (or `.hta` / `.lnk` / `.bat` / `.desktop`). Extraction at lib.rs:1305 writes the entry verbatim (`import_dir.join(file.name())` → `File::create`) — **the original extension is preserved; nothing validates file type.**
2. Victim opens the archive (the app's single primary use case) and clicks the attachment's "Open" button. The frontend calls `invoke("open_media_file", { filename })`; the backend (lib.rs:3528) `join`s it under the chat media dir and calls `tauri_plugin_opener::open_path(...)`.
3. `open_path` hands the file to the **OS default handler** → on Windows `Invoice.pdf.exe` executes; `.hta`/`.lnk`/`.bat` run; on Linux a `.desktop` launches. No second prompt.
Alternate trigger, no attachment needed: a message line containing `file:///C:/Windows/System32/calc.exe#maps.google.com` is accepted by `extract_maps_url` (substring `.contains("maps.google.com")`, line 3135) and stored as the "maps" URL; clicking "Open Maps" calls `openUrl(file://…)` → OS launches the target.

## Why this matters
Opening a chat archive received from another person is the entire purpose of this app. A single click on attacker-controlled content yields arbitrary code execution with the victim's privileges. **Path traversal is not even required** — plain extension confusion on a normally-extracted media file is sufficient — which is why this outranks the individual traversal findings: the easiest exploit has no exotic precondition.

## Evidence
lib.rs:1303-1308 (extraction writes attacker-named entry verbatim):
```
                media_files.push(name.clone());
                if let Some(parent) = out_path.parent() {
                    ensure_dir_exists(parent);
                }
                if let Ok(mut out_file) = File::create(&out_path) {
                    let _ = std::io::copy(&mut file, &mut out_file);
                }
```
lib.rs:3528-3540 (`open_media_file` → OS handler, only bidi-char strip, no type/`..` check):
```
fn open_media_file(chat_id: String, filename: String) -> Result<(), String> {
    let app_data = get_app_data_dir();
    let filename: String = filename.chars().filter(|c| !matches!(*c, ... )).collect();
    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);
    if !media_path.exists() { return Err(...); }
    tauri_plugin_opener::open_path(media_path.to_string_lossy().as_ref(), None::<&str>)
```
lib.rs:3135-3137 (`extract_maps_url` substring filter accepts `file://…#maps.google.com`):
```
        if patterns.iter().any(|p| part.contains(p)) && !part.contains("youtube.com") && !part.contains("youtu.be") {
            return Some(part.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '?' && c != '=' && c != '.' && c != ':' && c != ',' && c != '-' && c != '_' && c != '%' && c != '&' && c != '+' && c != '#').to_string());
```
```

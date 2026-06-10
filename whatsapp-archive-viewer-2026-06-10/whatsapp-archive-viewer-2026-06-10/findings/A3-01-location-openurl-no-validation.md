# A3-01 — Location "Open Maps" passes attacker-controlled URL to opener with no scheme validation or confirmation

**Severity:** High
**CWE:** CWE-601 (URL Redirection / Open Redirect to dangerous scheme) / CWE-20
**Status:** Confirmed
**Location:** project-code/src/App.tsx:5210 (sink); backend extract_maps_url at project-code/src-tauri/src/lib.rs:3111
**Vuln class:** Unvalidated URL → opener plugin (open_url) launch of arbitrary scheme/handler

## Exploit sketch
A location message's `msg.media` is derived from archive text by `extract_maps_url`, which accepts any whitespace token that merely `.contains("maps.google.com")` (substring, not host). An attacker crafts a message containing e.g. `file:///C:/Windows/System32/calc.exe#maps.google.com` or `smb://attacker/share#maps.google.com` or `vbscript:...#maps.google.com`. The viewer types it as a `location`, renders an "Open Maps" button, and `onClick={() => openUrl(msg.media!)}` hands the raw string to the opener plugin — no scheme allowlist, no confirmation modal (unlike linkified text links). The OS handler for that scheme fires.

## Why this matters
Victim opening a malicious archive and clicking "Open Maps" can trigger arbitrary protocol handlers / open arbitrary local paths via the default app, a path to code execution or local-file exfiltration outside the asset scope.

## Evidence
```
src/App.tsx
5180:      {msg.type === "location" && msg.media && (
5210:            onClick={() => openUrl(msg.media!).catch(console.error)}

src-tauri/src/lib.rs
3135:        if patterns.iter().any(|p| part.contains(p)) && !part.contains("youtube.com") ...
3137:            return Some(part.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '?' && c != '=' && c != '.' && c != ':' && c != ',' && c != '-' && c != '_' && c != '%' && c != '&' && c != '+' && c != '#').to_string());

capabilities/default.json
9:    "opener:allow-open-url",
10:    "opener:allow-open-path",
```
`trim_matches` keeps `:` `/` `#` `?` so a full `scheme:...#maps.google.com` survives. No allowlist on `opener:allow-open-url`.

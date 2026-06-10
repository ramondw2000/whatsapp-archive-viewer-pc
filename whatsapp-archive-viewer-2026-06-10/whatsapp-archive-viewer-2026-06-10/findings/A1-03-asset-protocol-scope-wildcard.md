# A1-03 — Asset protocol scope is `**/*` (entire filesystem readable via asset://)

**Severity:** High
**CWE:** CWE-22
**Status:** Confirmed
**Location:** src-tauri/tauri.conf.json:24
**Vuln class:** Over-broad asset protocol scope / arbitrary file read

## Exploit sketch
`assetProtocol.scope` is `["**/*"]`, so `convertFileSrc("/etc/passwd")` / `asset://localhost/...` in the renderer can fetch ANY absolute path on the host. Combined with `get_media_path` (lib.rs:3556) which returns absolute paths, or any path the renderer constructs, the webview reads arbitrary files directly. CSP `img-src`/`media-src` allow `asset:` and `https://asset.localhost`, so the content loads.

## Why this matters
Any XSS or attacker-controlled renderer content (e.g. an SVG/HTML sticker rendered from a malicious archive) can read arbitrary local files and render/exfiltrate them. Removes any path-scoping defense the per-command checks might provide.

## Evidence
```
21:    "security": {
22:      "assetProtocol": {
23:        "enable": true,
24:        "scope": ["**/*"]
25:      },
```
Scope should be narrowed to the app data `chats/*/media` and `custom` directories only.

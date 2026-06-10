# A2-01 — Asset protocol scope grants read of ALL files on disk

**Severity:** High
**CWE:** CWE-552
**Status:** Confirmed
**Location:** src-tauri/tauri.conf.json:24 (config for lib.rs media surface)
**Vuln class:** Overly permissive file-access scope / arbitrary file read

## Exploit sketch
`assetProtocol.scope = ["**/*"]` lets any code running in the webview (e.g. injected
via a malicious chat message rendered as HTML, an XSS in the SPA, or a compromised
`https://unpkg.com` dependency allowed by the CSP) load `asset://localhost/<any-abs-path>`
and read arbitrary files: `~/.ssh/id_rsa`, `/etc/passwd`, browser cookie DBs, etc.

## Why this matters
Converts any webview script-execution foothold into full local-file exfiltration. The
app already pulls scripts from `https://unpkg.com` (CSP `script`/`connect-src`), widening
the foothold surface.

## Evidence
```
22:      "assetProtocol": {
23:        "enable": true,
24:        "scope": ["**/*"]
25:      },
26:      "csp": "... script... https://unpkg.com; connect-src 'self' ... https://unpkg.com"
```
Scope should be restricted to the app data dir (`.../WhatsAppArchiveViewer/**`).

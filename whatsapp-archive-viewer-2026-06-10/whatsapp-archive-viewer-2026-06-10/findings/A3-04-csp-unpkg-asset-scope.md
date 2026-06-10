# A3-04 — Permissive CSP (remote unpkg.com, broad asset scope) widens blast radius of any HTML injection

**Severity:** Medium
**CWE:** CWE-693 (Protection Mechanism Failure) / CWE-942 (overly permissive cross-domain)
**Status:** Open
**Location:** project-code/src-tauri/tauri.conf.json:26; assetProtocol scope at :22-25
**Vuln class:** CSP weakness / defense-in-depth gap

## Exploit sketch
`script-src` is not set, so it inherits `default-src 'self'` — inline/eval scripts are blocked, which is good. But: `img-src` and `connect-src` both allow `https://unpkg.com`, an arbitrary public CDN. If any HTML/markup injection vector is ever introduced (today React auto-escapes, so none confirmed), an attacker could beacon/exfiltrate via `unpkg.com` image/connect requests and load remote images. `style-src 'unsafe-inline'` permits attacker-influenced inline styles (e.g. the CSS `url('...')` background at App.tsx:8322 fed by archive-derived `photo_path`). `assetProtocol.scope` is `["**/*"]` — the asset protocol can serve ANY file on disk, so any place a filename/path reaches an `asset://`/`convertFileSrc` URL has no scope containment.

## Why this matters
The CSP and asset scope provide little containment: a single future HTML-injection or a path-confusion bug becomes data exfiltration (unpkg) or arbitrary local-file read (asset scope `**/*`). Remote CDN in connect-src/img-src has no apparent legitimate use in the frontend.

## Evidence
```
tauri.conf.json
22:      "assetProtocol": { "enable": true, "scope": ["**/*"] },
26:      "csp": "default-src 'self'; img-src 'self' data: asset: ... https://unpkg.com; media-src ...; style-src 'self' 'unsafe-inline'; font-src 'self'; connect-src 'self' ipc: https://ipc.localhost https://unpkg.com"
```
No `script-src` (falls back to `default-src 'self'`); `unpkg.com` in img-src + connect-src; asset scope `**/*`.

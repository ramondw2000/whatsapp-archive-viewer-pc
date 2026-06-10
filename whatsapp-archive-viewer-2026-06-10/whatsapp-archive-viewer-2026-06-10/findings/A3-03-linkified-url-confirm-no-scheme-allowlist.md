# A3-03 — Linkified message URLs opened via opener after weak confirm modal; no scheme allowlist

**Severity:** Medium
**CWE:** CWE-601 / CWE-20
**Status:** Open
**Location:** project-code/src/utils/textRendering.tsx:20-42; project-code/src/App.tsx:2228-2233
**Vuln class:** Unvalidated URL → opener plugin (mitigated only by a click-through modal)

## Exploit sketch
`renderMessageText` linkifies tokens matching `/((?:https?:\/\/|www\.)[^\s<>"']+)/gi`. On click it routes to `openLinkWithConfirm` → a confirmation modal → `openUrl(linkConfirmUrl)`. The regex restricts the *scheme prefix* to `http(s)`/`www.`, so pure `javascript:`/`file:` tokens are not linkified. However: (1) there is no positive scheme allowlist before `openUrl`, so the only barrier is the modal; (2) the modal only displays the URL (CWE-451-style spoofing — a long/obfuscated `https://` URL can mask its true destination, and the user is conditioned to click "Open"); (3) `[^\s<>"']+` permits embedded credentials/`@`-host confusion and unusual hosts, all forwarded to the OS handler. Compare A3-01 where the *same* opener sink has NO modal at all.

## Why this matters
A malicious archive can present convincing links that, once confirmed, open in the OS handler. Combined with the absence of a backend scheme allowlist on `opener:allow-open-url`, the modal is the sole defense and is bypassable via social engineering.

## Evidence
```
src/utils/textRendering.tsx
20:    const URL_RE = /((?:https?:\/\/|www\.)[^\s<>"']+)/gi;
28:    const url = rawUrl.toLowerCase().startsWith('www.') ? 'https://' + rawUrl : rawUrl;
36:    onClick={e => { e.preventDefault(); onLinkClick(url); }}

src/App.tsx
2228: function confirmOpenLink() {
2230:    openUrl(linkConfirmUrl).catch(console.error);   // no scheme allowlist
```

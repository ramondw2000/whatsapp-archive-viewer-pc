# A3-06 — Message text / sender / filename rendering — DOM XSS

**Severity:** Low
**CWE:** CWE-79
**Status:** False-Positive
**Location:** project-code/src/utils/textRendering.tsx; project-code/src/App.tsx (message render paths)
**Vuln class:** Stored/DOM XSS via attacker-controlled archive text

## Exploit sketch
Attacker message text containing `<img src=x onerror=alert(1)>` or `<script>`. Tested whether it reaches the DOM as raw HTML.

## Why this matters
In a Tauri webview, XSS pivots to IPC → native commands. Would be Critical if present.

## Evidence / Why ruled out
- No `dangerouslySetInnerHTML`, `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write`, `eval`, or `new Function` anywhere under `src/` (grep returned zero hits).
- `renderMessageText` (textRendering.tsx) splits text into React children (strings, `<mark>`, `<a>` with text child). All untrusted substrings are rendered as JSX text nodes, which React HTML-escapes. The linkified `<a>` uses `href="#"` + onClick (the dangerous part is the opener call — see A3-03), and `title={url}` is attribute-escaped by React.
- `highlightText` likewise emits `<mark>{part}</mark>` text children.
- Sender names, filenames (alt/text), timestamps are rendered as `{value}` JSX children — escaped.
React's default escaping is the mitigation; no raw-HTML sink exists. Ruled out as XSS. (The exploitable issues are the opener/path sinks A3-01/A3-02/A3-03, not markup injection.)

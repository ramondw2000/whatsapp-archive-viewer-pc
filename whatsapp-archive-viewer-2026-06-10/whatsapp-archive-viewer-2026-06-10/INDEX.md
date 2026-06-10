# Security Audit: whatsapp-archive-viewer-pc

2026-06-10 | delegate-3 (split by risk surface) + orchestrator second-look | Scope: Tauri 2 desktop app — Rust backend `src-tauri/src/lib.rs` (7,491 LoC), React/TS frontend (`src/App.tsx` + utils, ~9.6k LoC), `tauri.conf.json`, capabilities. `protocol-asset` enabled.

**Threat model:** Both the contents of an opened archive (chat text, sender names, filenames, media) AND the data crossing the IPC boundary are 100% attacker-controlled — opening a third-party archive is the app's primary use case. The OS-handler launch sinks (`open_path`/`open_url`) and `dialog:allow-open` are granted in `capabilities/default.json`, so the launch findings are reachable.

## Findings

| # | Title | Severity | CWE | Status | File | Source |
|---|-------|----------|-----|--------|------|--------|
| 001 | Click-to-execute RCE: malicious media file/extension launched via OS handler (no traversal needed) | Critical | CWE-94 | Confirmed | lib.rs:3528,1305,3111 | findings/ORCH-01-click-to-execute-rce.md |
| 002 | Zip-slip — `import_chat` ZIP extraction writes entries to raw `file.name()` | High | CWE-22 | Confirmed | lib.rs:1275 | findings/A1-01-*, A2-04-* |
| 003 | `import_zip_from_bytes` — arbitrary file write (attacker content + path) | High | CWE-22 | Confirmed | lib.rs:5719 | findings/A1-06-* |
| 004 | Asset-protocol `scope:["**/*"]` — entire filesystem readable from the webview | High | CWE-552 | Confirmed | tauri.conf.json:24 | findings/A1-03-*, A2-01-*, A3-04-* |
| 005 | `read_file_as_base64(path)` — unrestricted arbitrary-file-read IPC primitive | High | CWE-22 | Confirmed | lib.rs:5199 | findings/A1-05-*, A2-02-* |
| 006 | Media commands join unsanitized `filename`/`chat_id` → traversal read (`get_media_as_base64`, `get_media_with_dims`, `get_media_path`, vCard) | High | CWE-22 | Confirmed | lib.rs:2771 | findings/A1-02-*, A2-03-* |
| 007 | `open_media_file` — path traversal into `open_path` launch | High | CWE-749 | Confirmed | lib.rs:3532 | findings/A1-04-*, A3-02-* |
| 008 | `rename_media_file` — traversal on src + dst → arbitrary file move | High | CWE-22 | Confirmed | lib.rs:3213 | findings/A1-07-* |
| 009 | `delete_chat` / `clear_all_chats` — traversal → arbitrary `remove_dir_all` | High | CWE-22 | Confirmed | lib.rs:4140 | findings/A1-08-* |
| 010 | `open_url` + `extract_maps_url` substring filter → `file://`/arbitrary-scheme launch (Open Maps button) | High | CWE-601 | Confirmed | lib.rs:3111,3513 | findings/A3-01-*, A1-12-*, A2-07-* |
| 011 | `extract_file_from_zip` / `try_extract_from_zip` — traversal write + loose name match | Medium | CWE-22 | Open | lib.rs:3788 | findings/A1-09-* |
| 012 | `import_from_export` blocks `/` but not `\` → Windows zip-slip | Medium | CWE-22 | Open | lib.rs:5950 | findings/A1-10-* |
| 013 | Uncapped extraction — zip bomb / disk + RAM exhaustion | Medium | CWE-409 | Open | lib.rs:1305 | findings/A1-11-*, A2-06-* |
| 014 | 7z/unrar shelled out with `-y`, no symlink/traversal guard | Medium | CWE-22 | Open | lib.rs:1161 | findings/A1-14-* |
| 015 | No dimension/alloc cap before `image` decode → decompression-bomb OOM | Medium | CWE-409 | Open | lib.rs (image decode) | findings/A2-05-* |
| 016 | Linkified message URLs → `openUrl` with confirm-only (spoofable) defense, no scheme allowlist | Medium | CWE-601 | Open | src/App.tsx | findings/A3-03-* |
| 017 | CSP allows `https://unpkg.com` (img/connect) + `'unsafe-inline'` styles — weak containment | Medium | CWE-693 | Open | tauri.conf.json | findings/A3-04-* |
| 018 | `symphonia` `features=["all"]` probes untrusted audio with every codec | Low | CWE-400 | Open | Cargo.toml / lib.rs | findings/A2-09-* |
| 019 | `check_file_exists` — arbitrary file-existence oracle | Low | CWE-22 | Open | lib.rs:112 | findings/A1-13-* |
| 020 | Global DB `Mutex.expect("poisoned")` — one panic-while-locked kills DB layer for session | Low | CWE-248 | Open | lib.rs | findings/A2-08-* |
| 021 | `photo_path` interpolated into inline `backgroundImage:url()` — fragile raw-string branches | Low | CWE-79 | Open | src/App.tsx | findings/A3-05-* |
| 022 | Shell command injection in external-tool calls | — | CWE-78 | False-Positive | lib.rs:1161 | findings/A1-15-* |
| 023 | SQL injection (LIMIT/OFFSET + migration `format!`) | — | CWE-89 | False-Positive | lib.rs | findings/A2-10-* |
| 024 | ReDoS in parser regexes + shipped `chats.db` secrets | — | CWE-1333 | False-Positive | lib.rs | findings/A2-11-* |
| 025 | Text-renderer / DOM XSS (the headline concern) | — | CWE-79 | False-Positive | src/utils/textRendering.tsx | findings/A3-06-* |

## Summary
- **Total: 21 active findings** (+ 4 False-Positive documented for due diligence)
- Critical: 1 | High: 9 | Medium: 7 | Low: 4
- Confirmed: 10 | Open: 11 | False-Positive: 4

## Dominant theme
A complete set of **read / write / move / delete / launch arbitrary-path primitives** exposed over IPC, with no canonicalization or scope confinement anywhere — every filesystem command takes a raw `chat_id`/`filename`/`path` and `join`s it after at most stripping Unicode bidi marks. Amplified by `assetProtocol.scope:["**/*"]`. The chain terminates in **#001 (Critical): a malicious archive → one click → arbitrary code execution**, and the easiest variant requires no path traversal at all (extension confusion on a normally-extracted media file).

## What was ruled out (and why)
- **SQL injection** (#023, high confidence): every `execute`/`query_*`/`prepare`/`execute_batch` binds user values via `?`/`params!`/`ToSql`. The only `format!`-built SQL interpolates i64-typed LIMIT/OFFSET (cannot carry syntax) or hardcoded crate literals. Archive content is always inserted via bound params.
- **DOM/markup XSS** (#025, high confidence): zero `dangerouslySetInnerHTML`/`innerHTML`/`eval`/`document.write`/`new Function` in `src/`; all archive text rendered as auto-escaped React JSX text children. The webview's real exposure is the opener launch sinks (#001/#010/#016), not HTML injection.
- **ReDoS** (#024): the `regex` crate is a linear-time NFA engine (no catastrophic backtracking). Shipped `chats.db` is 0 bytes / no tables — no leaked data or secrets. No hardcoded keys, no MD5/SHA1 found anywhere.
- **Shell command injection** (#022): all external-tool invocations use `Command::new` with separated `.arg()`s and no `sh -c`; only switch-confusion is theoretically possible (folded into #014).
- **Prototype pollution / race conditions**: JSON merges key on known fields (no attacker-controlled object keys); global state is a single `Mutex<Connection>` + atomics — serialized access, no data race (only the poison-panic DoS #020).

## Notes for triage
- #001 should be fixed first and is the headline risk: enforce an extension/MIME allowlist before `open_path`, and a positive URL-scheme allowlist (`https://` only) before `open_url`.
- The traversal cluster (#002–#009, #011–#012) shares one root cause — add `canonicalize()` + "result must stay within the intended base dir" checks to a shared helper and route every file command through it.
- #004 (`scope:["**/*"]`) should be narrowed to the app-data chats directory regardless of the other fixes — it is independent defense-in-depth.
- Status kept **Open** (not dismissed) for anything not directly exploit-verified in this pass; only the 10 findings checked against source are **Confirmed**.

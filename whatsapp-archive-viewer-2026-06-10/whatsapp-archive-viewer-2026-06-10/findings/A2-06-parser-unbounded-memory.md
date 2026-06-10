# A2-06 — Chat-log parser loads whole file + unbounded message accumulation

**Severity:** Low
**CWE:** CWE-400
**Status:** Open
**Location:** src-tauri/src/lib.rs:1191 / 1979
**Vuln class:** Uncontrolled resource consumption (memory)

## Exploit sketch
Import reads the entire `.txt` into memory (`fs::read` / `read_to_end` then decode to a
`String`), then `parse_chat_text` does `content.lines().collect::<Vec<&str>>()` and builds a
`Vec<Message>` plus a JSON backup of all messages — all unbounded. A multi-GB `.txt` inside a
small highly-compressed ZIP (the archive itself can be a "text bomb") exhausts memory.

## Why this matters
DoS on import of an attacker-supplied archive. Lower severity because import is user-initiated
and single-shot, but there is no size cap anywhere in the pipeline.

## Evidence
```
1284:                let mut txt_bytes = Vec::new();
1285:                file.read_to_end(&mut txt_bytes)...;          // whole txt, no cap
1979:    let lines: Vec<&str> = content.lines().collect();         // whole file materialized
```
No max-file-size / max-message-count guard; ZIP entries are also extracted with no size cap.

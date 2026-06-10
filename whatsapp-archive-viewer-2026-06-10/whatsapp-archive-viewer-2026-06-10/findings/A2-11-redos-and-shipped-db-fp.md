# A2-11 — ReDoS in chat-log regexes, and shipped chats.db secrets

**Severity:** Low
**CWE:** CWE-1333
**Status:** False-Positive
**Location:** src-tauri/src/lib.rs:1993-2085, 1681-1684, 3035-3036; src-tauri/chats.db
**Vuln class:** ReDoS / secret-in-repo

## Exploit sketch
Many `regex::Regex` patterns run per-line over attacker-controlled `.txt` content
(`(.+?):\s+(.+)$` etc.); `chats.db` ships in the repo.

## Why this matters / why ruled out
- **ReDoS:** the `regex` crate uses a finite-automaton engine with guaranteed linear-time
  matching and NO backtracking; catastrophic backtracking is structurally impossible. The
  patterns also contain no nested unbounded quantifiers that could matter even in a
  backtracking engine. Not exploitable as ReDoS. (Per-line cost is linear; see A2-06 for the
  separate whole-file memory concern.)
- **Shipped chats.db:** the file `src-tauri/chats.db` is 0 bytes (empty — no tables, no rows).
  Contains no real chat data, no secrets. No hardcoded keys / weak crypto (MD5/SHA1) exist
  anywhere in lib.rs; the only crypto-adjacent code is a custom base64 (not security-relevant).

## Evidence
```
$ ls -la src-tauri/chats.db   ->  0 bytes
$ sqlite: "no such table: chats" / "no such table: messages"
1997: regex r"^\[(\d{2}/\d{2}/\d{4}),?\s+...\]\s+(.+?):\s+(.+)$"   // linear-time regex crate
```
Ruled out on both counts.

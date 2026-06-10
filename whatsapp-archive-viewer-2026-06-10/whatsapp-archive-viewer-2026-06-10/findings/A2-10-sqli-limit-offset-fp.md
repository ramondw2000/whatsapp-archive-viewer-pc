# A2-10 — SQL string-interpolation of LIMIT/OFFSET and migration patterns

**Severity:** Low
**CWE:** CWE-89
**Status:** False-Positive
**Location:** src-tauri/src/lib.rs:2597 (also 667-781 migrations, 4435 search WHERE)
**Vuln class:** SQL injection

## Exploit sketch
`get_chat_messages` builds the query with `format!("... LIMIT {} OFFSET {}", lim, off)`.

## Why this matters / why ruled out
- `lim`/`off` are typed `Option<i64>` from the IPC boundary; serde rejects non-integers, so
  only integers can ever reach the `format!`. No string injection is possible — an i64 cannot
  carry SQL syntax. Not exploitable.
- The migration `format!(...)` queries (lib.rs:667-781) interpolate ONLY hardcoded crate
  literals (`"opus"`, `"tiff"`, `"maps.google.com"`, ...), never user input.
- `search_messages_filtered` (lib.rs:4435) interpolates only fixed condition strings with
  positional `?{n}` placeholders; all user values are bound via `params`/`ToSql`. Safe.
- All other queries use `?` / `params!` / named binds (verified across every `execute`,
  `query_row`, `query_map`, `prepare`). No user-controlled table/column/ORDER BY interpolation.

## Evidence
```
2597:        (Some(lim), Some(off)) => format!(
2599:            "... ORDER BY id ASC LIMIT {} OFFSET {}", lim, off),   // lim/off are i64
```
Ruled out: integer-typed; no string sink. (Style nit: still prefer bound `LIMIT ?`.)

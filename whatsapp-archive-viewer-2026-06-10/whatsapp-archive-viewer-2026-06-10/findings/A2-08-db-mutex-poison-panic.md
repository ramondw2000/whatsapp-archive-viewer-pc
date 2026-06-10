# A2-08 — Global DB mutex `.expect("poisoned")` → cascading panic DoS

**Severity:** Low
**CWE:** CWE-248
**Status:** Open
**Location:** src-tauri/src/lib.rs:24
**Vuln class:** Improper error handling / panic-induced DoS

## Exploit sketch
`get_db()` does `.lock().expect("Database mutex poisoned")`. The DB is a single global
`Mutex<Connection>`. If ANY command panics while holding the lock (e.g. an `unwrap`/index
panic on attacker data inside a `query_map` closure or a sub-call), the mutex becomes
poisoned and every subsequent `get_db()` call panics — the whole DB layer is permanently
dead for the session.

## Why this matters
A single triggerable panic-while-locked turns into a persistent denial of service for all
DB-backed features until restart. Also `init_database().expect(...)` panics the app on any
DB-open error.

## Evidence
```
24:fn get_db() -> std::sync::MutexGuard<'static, Connection> {
25:    static DB: OnceLock<Mutex<Connection>> = OnceLock::new();
26:    DB.get_or_init(|| {
27:        let conn = init_database().expect("Failed to initialize SQLite database");
28:        Mutex::new(conn)
29:    }).lock().expect("Database mutex poisoned")
```
Consider recovering from poison (`into_inner`) and returning `Result` instead of `expect`.

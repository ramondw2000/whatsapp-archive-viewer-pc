# A1-15 — (Due diligence) Command/arg injection via 7z/unrar invocation

**Severity:** Low
**CWE:** CWE-78
**Status:** False-Positive
**Location:** src-tauri/src/lib.rs:1161
**Vuln class:** OS command injection

## Exploit sketch
Considered whether `zip_path` / `filename` reaching `std::process::Command::new("7z")...arg(x)` allows shell injection. Ruled out: `Command` invokes the binary directly (no shell), and each value is passed as a single separate `arg()`, so shell metacharacters are not interpreted. There is no `sh -c`. Remaining concern is argument-injection (a filename beginning with `-` interpreted as a 7z/unrar switch), which is real but limited to switch confusion, not command execution — tracked qualitatively under A1-09/A1-14.

## Why this matters
No classic command injection here; documented so the orchestrator does not re-flag it as RCE-via-shell.

## Evidence
```
1161: let output = std::process::Command::new("7z")
1163:     .arg("x").arg(&zip_path)
1167:     .arg(format!("-o{}", import_dir.to_string_lossy())).arg("-y")
```
No `Command::new("sh")` / `.args(["-c", ...])` anywhere in lib.rs.

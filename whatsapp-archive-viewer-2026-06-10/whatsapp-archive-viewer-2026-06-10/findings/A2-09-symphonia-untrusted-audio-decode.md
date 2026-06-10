# A2-09 — `symphonia` probes/decodes untrusted audio with all codecs, no limits

**Severity:** Low
**CWE:** CWE-400
**Status:** Open
**Location:** src-tauri/src/lib.rs:302 (get_audio_duration)
**Vuln class:** Untrusted media decode / resource exhaustion

## Exploit sketch
`get_audio_duration` feeds an attacker-supplied media file (from an imported archive) to
`symphonia::default::get_probe().format(...)`. Cargo.toml enables `symphonia` with
`features = ["all"]`, exposing every demuxer/decoder to untrusted input. A malformed file can
drive the probe into large allocations or pathological CPU; only duration is needed but the
full format is opened. No size/time bounds.

## Why this matters
Broadens the untrusted-decode attack surface (all codecs) for a feature that only needs a
duration. Memory-safety bugs in any enabled decoder become reachable from imported archives.

## Evidence
```
302:fn get_audio_duration(file_path: &Path) -> Option<String> {
...
348:    match symphonia::default::get_probe().format(&hint, mss, &meta_opts, &metadata_opts) {
```
Cargo.toml:33  `symphonia = { version = "0.5", features = ["all"] }`
Restrict to the codecs actually used; cap input size before probing.

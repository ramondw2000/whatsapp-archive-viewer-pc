# A2-05 — No dimension/size cap before `image` decode (decompression bomb DoS)

**Severity:** Medium
**CWE:** CWE-409
**Status:** Open
**Location:** src-tauri/src/lib.rs:3016 (also 3051 TIFF→PNG)
**Vuln class:** Resource exhaustion / decompression bomb

## Exploit sketch
`get_image_dimensions` calls `image::load_from_memory(bytes)` and `convert_tiff_to_png`
calls `image::load_from_memory_with_format(..., Tiff)` on attacker-supplied media (from an
imported archive). A crafted image declaring e.g. 60000x60000 px forces multi-GB allocation
(`width*height*4`) during full decode → OOM / process kill. No `Limits`
(`image::io::Limits` / `with_limits`) and no max-dimension/byte cap are set before decode.

## Why this matters
A single small malicious file in an imported chat archive can OOM-crash the app on open
(images are decoded eagerly to compute dimensions and for TIFF→PNG conversion). DoS.

## Evidence
```
3012:fn get_image_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
3016:    match image::load_from_memory(bytes) {        // no Limits set
3017:        Ok(img) => { let (width, height) = img.dimensions(); ...

3049:fn convert_tiff_to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
3051:    let img = image::load_from_memory_with_format(bytes, ImageFormat::Tiff)... // full decode, no cap
```
Mitigation: use `ImageReader` with `.limits(Limits)` capping width/height/alloc before decode.

# Code Quality Report — WhatsApp Archive Viewer

Generated after completing the Testing & Code Cleanup Plan (Phases 1–6).

---

## Summary

| Category | Count | Status |
|---|---|---|
| Rust tests | 78 | ✅ All pass |
| React/TS tests | 149 | ✅ All pass |
| Clippy errors (blocking) | 1 | ✅ Fixed |
| TypeScript errors | 7 | ✅ Fixed |
| Clippy warnings (non-blocking) | 26 | ⚠️ Listed below |

---

## Fixed Issues

### Rust — Clippy Error

**`clippy::unused_io_amount`** — `src/lib.rs:308`
- `f.read(&mut buf)` does not guarantee all bytes are read.
- **Fixed**: changed to `f.read_exact(&mut buf)`.

### TypeScript Errors (all fixed)

| File | Line | Issue | Fix |
|---|---|---|---|
| `src/types.ts` | 5 | `Message` interface missing `id` field, but `App.tsx` used `fav.id` | Added `id?: number \| null` |
| `src/App.tsx` | 2091 | `pressTimers` state typed as `Map<number, number>` but `setTimeout` returns `Timeout` | Changed to `Map<number, ReturnType<typeof setTimeout> \| undefined>` |
| `src/LazyMediaImage.tsx` | 309 | `scrollTimeout` typed as `number`, should be `ReturnType<typeof setTimeout>` | Fixed type annotation |
| `src/test/utils/textRendering.test.tsx` | 2,4 | Unused `screen` and `React` imports | Removed unused imports |

---

## Remaining Clippy Warnings (26)

These are non-blocking style/idiom suggestions. Grouped by type:

### Style / Formatting (2)

| Line | Warning |
|---|---|
| `lib.rs:296` | Empty lines after doc comment |
| `lib.rs:6237` | Empty line after outer attribute |

### Idiomatic Rust — Range & Char (7)

| Line | Warning |
|---|---|
| `lib.rs:242` | Manual char comparison — use `char::is_ascii_digit()` or similar |
| `lib.rs:252` (×2) | Manual `RangeInclusive::contains` — use `.contains()` |
| `lib.rs:253` (×2) | Manual `RangeInclusive::contains` — use `.contains()` |
| `lib.rs:255` (×2) | Manual `RangeInclusive::contains` — use `.contains()` |

### Idiomatic Rust — Control Flow (4)

| Line | Warning |
|---|---|
| `lib.rs:406` | `match` can be replaced with `?` operator |
| `lib.rs:1396` | `match` for single pattern — use `if let` |
| `lib.rs:1937` | `match` expression looks like `matches!` macro |
| `lib.rs:3043` | `match` for single pattern — use `if let` |

### Idiomatic Rust — Iterator (2)

| Line | Warning |
|---|---|
| `lib.rs:1111` | `.map(..).flatten()` — use `.flat_map()` instead |
| `lib.rs:4376` | Explicit counter loop — use `.enumerate()` |
| `lib.rs:4538` | Explicit counter loop — use `.enumerate()` |

### Unnecessary Borrows (5)

| Line | Warning |
|---|---|
| `lib.rs:1264` | Borrowed expression implements required traits — remove `&` |
| `lib.rs:1370` | Reference immediately dereferenced — simplify |
| `lib.rs:3894` | Borrowed expression implements required traits — remove `&` |
| `lib.rs:5262` | Borrowed expression implements required traits — remove `&` |
| `lib.rs:5370` | Borrowed expression implements required traits — remove `&` |
| `lib.rs:5883` | Borrowed expression implements required traits — remove `&` |

### Logic / Safety (2)

| Line | Warning |
|---|---|
| `lib.rs:2381` | `unwrap()` on `media_path` after `is_some()` check — use `if let` |
| `lib.rs:3119` | Manual `div_ceil` reimplementation — use `.div_ceil()` |

### Type Complexity (2)

| Line | Warning |
|---|---|
| `lib.rs:4998` | Very complex tuple type — consider a named struct or type alias |
| `lib.rs:5527` | Very complex return type — consider a named struct or type alias |

---

## Recommendations

### High Priority
- **`lib.rs:2381`** — Replace `.is_some()` + `.unwrap()` with `if let` to eliminate a potential panic.
- **`lib.rs:4998`, `lib.rs:5527`** — Extract complex tuple types into named structs for maintainability.

### Medium Priority (18 auto-fixable)
Run `cargo clippy --fix --lib -p whatsapp-archive-viewer-pc` to apply 18 of the 26 suggestions automatically (range contains, borrows, iterator, match→? operator, etc.).

### Low Priority
- Style/formatting warnings (empty doc lines) can be fixed manually.
- `div_ceil` and `flat_map` are purely idiomatic improvements.

---

## Test Coverage Summary

### Rust (`src-tauri/src/lib.rs` + `tests/unit_tests.rs`) — 78 tests

- `sanitize_filename` (5 tests)
- `validate_chat_id` (6 tests)
- `validate_url_scheme` (7 tests)
- `is_safe_open_extension` (4 tests)
- `timestamp_to_epoch` (5 tests)
- `normalize_chat_name` (covered via parse tests)
- `detect_group_chat` (3 tests)
- `parse_chat_text` (40+ tests — message types, formats, media, Unicode)
- `merge_messages_into_chat` (3 tests)
- `is_plausible_whatsapp_date` (covered)
- DB schema, CRUD, cascade delete, FTS, sticker migration (integration tests)

### React / TypeScript (`src/test/`) — 149 tests

- **Utils**: `formatDate` (15), `formatTime` (8), `phoneNumber` (26), `textRendering` (15)
- **Components**: `LoadingOverlay` (6), `JumpButton` (9), `ExpandableText` (8), `GroupAvatar` (10), `LinkConfirmModal` (6), `FileTypeBadge` (13)
- **Dialogs**: `DeleteConfirmationDialog` (5), `UsernameDialog` (9)
- **Media**: `MediaFallback` (8), `StickerImage` (4), `ProfileImage` (7)

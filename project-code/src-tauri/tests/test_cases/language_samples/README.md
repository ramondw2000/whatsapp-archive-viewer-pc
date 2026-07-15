# Language sample WhatsApp exports

This folder contains **synthetic** sample WhatsApp chat-export ZIPs, one per
language that `project-code/src-tauri/src/lib.rs` has group-detection /
membership-event parsing support for. They exist to manually (or, later,
automatically) verify that:

1. A chat is correctly detected as a **group** (via `is_group_chat`'s
   `group_indicators` substring check, or the "3+ distinct senders" fallback).
2. Group **membership events** — someone being added, someone being removed,
   and someone leaving — are correctly extracted per-language by
   `extract_membership_events` (the `MEMBERSHIP_RE_<CODE>_*` regexes).

## This is not real data

Every file here is fabricated from scratch for testing purposes only. All
"people" are named generically (`Test Person One`, `Test Person Two`, …
`Test Person Five`), all group names are placeholders (`Test Group <CODE>`),
and all ordinary chat messages are generic filler greetings. Nothing in this
folder corresponds to a real person, real conversation, or real WhatsApp
export. There are no privacy concerns with sharing or committing these files.

## How each fixture was built

Every fixture is a ZIP containing exactly one file, `_chat.txt`, in the
Android-style WhatsApp export format:

```
DD/MM/YYYY, HH:MM - <system message text>
DD/MM/YYYY, HH:MM - <Sender Name>: <message text>
```

Each `_chat.txt` contains, in chronological order:

1. A **"created group"** system message (from `Test Person One`).
2. Three ordinary chat messages from different senders (`Test Person Two`,
   `Test Person Three`, `Test Person One`) — just enough plausible filler
   traffic to look like a real export.
3. A **third-person "added"** system message: `Test Person One` adds
   `Test Person Four` (who then also sends a short "glad to be here"
   message).
4. A **third-person "removed"** system message: `Test Person Two` removes
   `Test Person Five`.
5. A **"left"** system message: `Test Person Three` leaves (a different
   person from the one removed, as requested).
6. Where the language has a literal "subject/topic changed" phrase available
   in `lib.rs`'s `group_indicators`, a **subject-changed** system message from
   `Test Person One` is appended too (optional / best-effort — see caveats
   below for the languages where this was skipped).

All literal phrasing (e.g. `"heeft de groep aangemaakt"`, `"a créé le
groupe"`, `"đã tạo nhóm"`, etc.) was copied verbatim from the
`group_indicators` array and the `MEMBERSHIP_RE_<CODE>_*` regex literals in
`lib.rs`, not invented, so each fixture actually exercises the real parsing
regexes for that language. The Python generator script used to build these
(data-driven, one config block per language) is not checked into the repo;
regenerate similarly if you need to tweak these fixtures.

## Languages covered (32 files)

| Code | Language | File |
|---|---|---|
| nl | Dutch | `sample_export_nl.zip` |
| en | English | `sample_export_en.zip` |
| fr | French | `sample_export_fr.zip` |
| de | German | `sample_export_de.zip` |
| es | Spanish | `sample_export_es.zip` |
| az | Azerbaijani | `sample_export_az.zip` |
| ca | Catalan | `sample_export_ca.zip` |
| cs | Czech | `sample_export_cs.zip` |
| da | Danish | `sample_export_da.zip` |
| et | Estonian | `sample_export_et.zip` |
| fi | Finnish | `sample_export_fi.zip` |
| hr | Croatian | `sample_export_hr.zip` |
| hu | Hungarian | `sample_export_hu.zip` |
| id | Indonesian | `sample_export_id.zip` |
| it | Italian | `sample_export_it.zip` |
| lt | Lithuanian | `sample_export_lt.zip` |
| lv | Latvian | `sample_export_lv.zip` |
| ms | Malay | `sample_export_ms.zip` |
| nb | Norwegian Bokmål | `sample_export_nb.zip` |
| pl | Polish | `sample_export_pl.zip` |
| pt | Portuguese (Portugal) | `sample_export_pt.zip` |
| ptbr | Portuguese (Brazil) | `sample_export_ptbr.zip` |
| ro | Romanian | `sample_export_ro.zip` |
| sk | Slovak | `sample_export_sk.zip` |
| sl | Slovenian | `sample_export_sl.zip` |
| sq | Albanian | `sample_export_sq.zip` |
| sv | Swedish | `sample_export_sv.zip` |
| sw | Swahili | `sample_export_sw.zip` |
| tl | Tagalog | `sample_export_tl.zip` |
| tr | Turkish | `sample_export_tr.zip` |
| uz | Uzbek | `sample_export_uz.zip` |
| vi | Vietnamese | `sample_export_vi.zip` |

Note: the code treats Portuguese (Portugal) and Portuguese (Brazil) as two
separate regex sets (`PT_*` vs `PTBR_*`), so both get their own fixture even
though the user-facing language list only mentions "Portuguese" once.

## Known best-effort caveats (inherited from lib.rs, not fixed here)

- **Azerbaijani (`az`)** and **Tagalog (`tl`)**: previously had gaps (a
  missing `AZ_THIRD_REMOVE` regex, a colon in `AZ_THIRD_ADD`'s literal text
  that tripped the importer's `is_system` heuristic, and a missing
  `TL_THIRD_ADD` regex). All three were fixed in `lib.rs` — confirmed against
  the real Android string resources this time, not guessed — and these two
  fixtures were rebuilt to match the corrected patterns exactly (the "removed"
  line for `az` and the "added" line for `tl` both changed to the real
  confirmed wording). Regression tests for both fixes live in `lib.rs`'s test
  module (`extract_membership_events__azerbaijani_third_person_remove_*`,
  `extract_membership_events__tagalog_third_person_add_*`, and
  `parse_chat_text__azerbaijani_added_message_with_colon_is_still_typed_as_system`).
- **Malay (`ms`) and Slovak (`sk`)**: no subject/topic-changed literal phrase
  is listed in `group_indicators` for these languages, so that optional event
  is omitted from their fixtures.
- **Uzbek (`uz`)**: `UZ_THIRD_ADD`/`UZ_THIRD_REMOVE` have no fixed verb
  between the actor and the target (just `"<actor> <target>ni …$"`), so a
  multi-word actor name makes the split ambiguous under the regex's lazy
  leading `.+?`. This fixture writes the actor's name without spaces (e.g.
  `TestPersonOne`) in those two lines specifically to avoid that ambiguity so
  the target still captures cleanly — this is a workaround in the fixture,
  not a fix to the underlying regex.
- **Portuguese (Brazil) (`ptbr`)**: reuses the Portugal `"criou o grupo"`
  creation phrase since `group_indicators` doesn't list a separate
  BR-specific group-creation string.

All of the above are exactly the kind of gaps this task was meant to surface
— the fixtures were deliberately kept faithful to what's actually in the code
rather than "fixed up" to look cleaner.

## How to use these fixtures

**Just want to read the raw text?** `txt/<code>/_chat.txt` has the same content
unzipped, one plain folder per language, for quick browsing/editing without
needing to unzip anything. The `.zip` files remain the source of truth for
actually importing through the app — if you edit a `txt/<code>/_chat.txt`,
re-zip it back into `sample_export_<code>.zip` to keep them in sync.

**Manual test (recommended first pass):** open the app, use the "+ Import"
button, and pick any `sample_export_<code>.zip` from this folder. Confirm:

- The imported chat is tagged/shown as a **group chat**.
- The group's member list shows `Test Person Four` as a current member and
  `Test Person Five` and `Test Person Three` as **former members**
  (added/removed/left), except where a caveat above says the structured event
  won't be extracted for that language.

**Automated test (future work):** these ZIPs can be pointed at directly from
a Rust integration test under `src-tauri/tests/` that calls the same
import/parsing entry points (`import_from_export_inner` /
`parse_chat_text` / `extract_membership_events`) used by the app, asserting
`is_group_chat` returns `true` and that the expected membership events come
back for each language. No such automated test exists yet — this folder just
provides the fixtures for one to be built against.

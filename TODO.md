# TODO — WhatsApp Archive Viewer

Single reference for open work. This replaces `goals.md`, `checklist.md`, `project-code/checklist.md`,
`checklist-audit.md`, and `project-code/code-quality-report.md` (all deleted — they were stale
snapshots/logs; their still-relevant contents are consolidated below) and `contact-linking-spec.md`
(deleted — the feature it described has since been implemented; its still-relevant "known limitation"
is captured below).

## Confirmed gaps

From a full code-vs-checklist audit done 2026-07-15 — everything else in the old checklists was
cross-checked against the actual code and found implemented.

- **No right-click/context menu for editing messages.** The actual UI uses a hover-revealed pencil icon
  instead, and only on media messages.

## Feature ideas (not started)

- Keyboard shortcuts
- Message reactions
- Voice message transcription
- Statistics dashboard (messages per day, most active times, etc.)
- Chat comparison/analytics
- Export chat as HTML/PDF

## Technical debt

- Parser regex patterns could be consolidated — more pressing now that `detect_group_chat` /
  `extract_membership_events` cover ~30 languages' worth of literal patterns.
- Database schema versioning needed for future migrations.
- Error handling in the UI could be more user-friendly in places.
- Run `cargo clippy` for a fresh pass — a prior report (now deleted) found 26 non-blocking warnings
  including one real risk (`unwrap()` after an `is_some()` check that could be an `if let` instead);
  line numbers from that report are stale after this session's changes, so re-run rather than trusting
  old references.
- ~~Excessive blank lines throughout lib.rs/App.tsx~~ and ~~dead code~~ — done, see below.

## Recently completed (2026-07-17, v0.1.1)

- **Update checker could tell a newer build to "update" to an older published release.** It compared
  versions with a plain `!==` string check, so any difference from the latest GitHub release tag —
  not just an older current version — triggered "Update Available," including the case where the
  running build is actually ahead of the latest published release. Added `isNewerVersion`
  (`src/utils/version.ts`, numeric per-component comparison) and use that instead.
- **Shift-click range selection** (#18) implemented, then refined twice more: fixed a regression
  where the click event that naturally follows a long-press's mousedown/mouseup immediately
  re-toggled (and undid) the selection the long-press had just made, since the plain-text click
  handler added for #19 didn't skip that trailing click the way the image/video/gif handlers
  already did; and added a visual "anchor" indicator (amber ring, `.selection-anchor` in `App.css`)
  showing which message a shift-click will range from, since there was previously no way to tell.
  The anchor only moves when a click *adds* a message to the selection — deselecting some other
  message leaves the anchor alone, and deselecting the anchor itself clears it rather than leaving
  a stale amber ring on a deselected message.
- **Plain text messages can now be selected in multi-select mode** (#19).
- **New-import auto-switch** (#20): importing a chat now switches to it even if a different chat
  was already open.
- **Sender profile name override** (#22) implemented, then two follow-up gaps fixed: the Group Info
  participant list wasn't consulting the override (`GroupParticipantsDialog` now takes a
  `displayNameOverrides` prop); and saving a rename via the participant edit popup didn't refresh
  the currently-open chat, so it looked stale until you switched chats and back.
- **Theme toggle now follows the OS light/dark default** (#25). Originally shipped as a three-way
  light/dark/auto cycle, then simplified per feedback to a plain two-way light/dark toggle whose
  *initial* value (when no preference is saved yet) is read from `prefers-color-scheme` — no
  separate "auto" mode to cycle through.
- **Group Info showed "0 participants" for groups where nobody ever sent a real chat message** (#26)
  — e.g. a group where every message is a system message (created/added/removed) with no actual
  chat content. The participant list was built only from message senders; added
  `get_group_participants` (lib.rs) to merge that with names mentioned in membership system-message
  events, so former members show up (with the existing "Former member" badge) even if they never
  posted anything.

## Recently completed (2026-07-15)

- Fixed `contact_groups` not backfilling across all of a contact's groups on reimport/restart — it
  previously only kept the one group where the contact had been manually edited.
- Added WhatsApp system-message parsing (group detection + member added/removed/left tracking) for
  ~30 languages. Dutch is verified against a real export; English and all newly-added languages are
  best-effort, sourced from a third-party decompiled WhatsApp APK string dump, not verified against
  real exports — treat accordingly if a language's detection looks off for a real user.
- Added unit test coverage and synthetic per-language test-fixture ZIPs for the above (see
  `project-code/src-tauri/tests/test_cases/language_samples/` for zips, `.../language_samples/txt/`
  for the same content unzipped and browsable per language). See `project-code/README.md` for the
  full per-language tested/known-issues table.
- Fixed the two gaps the test coverage surfaced: Azerbaijani was missing a "third-person removed"
  pattern, and its "third-person added" string genuinely contains `": "` in real WhatsApp output,
  which collided with the importer's regular-message parsing (not just the system-message heuristic —
  the actual root cause was that the *regular-message* pattern matches a line before the parser ever
  gets to system-message detection, so the fix lives there, not in the `is_system` check). Tagalog was
  missing a "third-person added" pattern. Both confirmed against the real Android string resources and
  covered by regression tests.
- Removed ~9,800 unnecessary blank lines from `lib.rs` and `App.tsx` — a mechanical artifact (one blank
  line after nearly every Rust statement, two after nearly every TSX line/import), not intentional
  style; genuine section-boundary spacing was preserved. Removed 12 dead Tauri commands with zero
  frontend references (`pick_zip_file`, `import_chat`, `import_chats_batch`, `get_contact_profile`,
  `open_url`, `check_file_exists`, `debug_chat_media`, `set_chat_background`, `get_chat_background`,
  `get_background_history`, `restore_chat_background`, `clear_chat_background`) plus their
  `invoke_handler` registrations, and one unused import in `unit_tests.rs`. All 193 backend + 149
  frontend tests and typecheck pass clean.

## Known design limitations (accepted, not bugs)

- Contact auto-matching between a group participant and a 1-on-1 chat is exact normalized-string
  match only (reuses `normalize_chat_name`). If a group shows a raw phone number but the matching
  1-on-1 chat was exported under a saved contact name, they won't auto-link — the manual link picker
  in the contact's profile is the intended fallback for this case.
- A brand-new contact is only ever created via an explicit edit-click on a group participant, or when
  a matching *unlinked 1-on-1 chat* is found during import/reconciliation — never from group-scanning
  alone. This is deliberate (avoids guessing identity for someone never confirmed), but it does mean a
  person who only ever appears in groups (no 1-on-1 chat, never edit-clicked) won't get a contact
  record at all.

## Explicitly declined

- **WhatsApp Communities support ("groups of groups")** — investigated 2026-07-15 with a real Dutch
  sample export. Dropped: the "added to community" system message is a fixed generic notice that never
  names the community (only a later *removal* message does), so two different groups both currently in
  *some* community can't be told apart or linked together from their exported text alone — only a
  minority case (a group that's been removed at some point) is even theoretically groupable. Not worth
  building unless WhatsApp changes what that message says.

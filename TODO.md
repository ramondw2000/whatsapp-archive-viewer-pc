# TODO — WhatsApp Archive Viewer

Single reference for open work. This replaces `goals.md`, `checklist.md`, `project-code/checklist.md`,
`checklist-audit.md`, and `project-code/code-quality-report.md` (all deleted — they were stale
snapshots/logs; their still-relevant contents are consolidated below) and `contact-linking-spec.md`
(deleted — the feature it described has since been implemented; its still-relevant "known limitation"
is captured below).

## Confirmed gaps

From a full code-vs-checklist audit done this session (2026-07-15) — everything else in the old
checklists was cross-checked against the actual code and found implemented.

- **Shift-click range selection in multi-select mode is missing.** No shift-key handling exists in
  the message click logic.
- **Clicking a plain text message in multi-select mode doesn't select it.** Only clicking image/video/GIF
  content toggles selection; text bubbles no-op their click handler while in multi-select mode.
- **No right-click/context menu for editing messages.** The actual UI uses a hover-revealed pencil icon
  instead, and only on media messages.
- **No "set a display name for a sender" feature** (rename a sender everywhere in the chat). The closest
  existing things are renaming a single media file's display name, and the Username dialog, which only
  marks which sender is "you" — neither renames a sender across the conversation.
- **New-import auto-open only works when no chat is already open.** If a chat is open when you import
  another, the app just refreshes the current chat instead of switching to the new one.

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

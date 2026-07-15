# Feature Spec: Link Group Participants to Shared Contact Profiles

**Status:** Not started

## Problem

Every profile today (`profiles` table, keyed by `chat_id`) is strictly scoped to one chat — there's no concept of "this person" independent of a specific chat. Group chats don't store participants at all; the Group Info dialog derives the participant list on the fly from distinct message senders every time it's opened, and clicking a participant does nothing.

WhatsApp exports have no stable contact ID — a person shows up as whatever string WhatsApp wrote for them at export time (a phone number if unsaved, a contact name if saved), and that same string is the only thing available both in a group's participant list and in a 1-on-1 chat's name.

## Goal

Each participant row in the Group Info dialog gets an edit button that opens that person's profile:

- If a 1-on-1 chat for that person already exists, edit *that* profile.
- If not, create a "shadow" profile for that phone/name right away, so notes/photo/etc. entered now aren't lost.
- If a matching 1-on-1 chat is imported later, the shadow profile's data is applied to it, and from then on the two stay linked — editing from either the group-participant view or the chat's own Profile dialog updates the same underlying data.
- A person can also show up in multiple groups; the profile tracks every group they've been resolved in, in its own "Linked Groups" section.
- Because auto-matching is just string comparison, it can get it wrong (or fail to find a match at all) — both cases need a manual override: an unlink/link toggle for the 1-on-1 chat, and a way to remove a group from the Linked Groups list without that removal cascading into removing the 1-on-1 link (or vice versa).

## Design

### New `contacts` table (the shared identity)

```sql
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    normalized_key TEXT NOT NULL UNIQUE,  -- normalize_chat_name(participant_string)
    display_key TEXT NOT NULL,            -- original un-normalized participant string
    name TEXT,
    notes TEXT,
    photo_path TEXT,
    phone_number TEXT
)
```

`normalized_key` reuses the existing `normalize_chat_name()` (lib.rs:1856) — the same lowercase/trim/strip-"(Group)" normalization already used by `find_existing_chat_by_name` (lib.rs:1868) to de-dupe re-imported chats. A group participant string and a 1-on-1 chat's `original_name` live in the same identity space, so this is a direct reuse, not new matching logic.

### `chats` gets one new nullable column

```sql
ALTER TABLE chats ADD COLUMN contact_id TEXT REFERENCES contacts(id)
```
Only ever set for `is_group = 0` chats — links that specific 1-on-1 chat to its shared contact row.

### New `contact_groups` table (many-to-many, independent of the single chat link)

```sql
CREATE TABLE IF NOT EXISTS contact_groups (
    contact_id TEXT NOT NULL REFERENCES contacts(id),
    chat_id TEXT NOT NULL REFERENCES chats(id),
    excluded INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (contact_id, chat_id)
)
```
Populated only by the explicit act of clicking Edit on a participant within a specific group — no passive/background scan. `excluded` is a soft-remove flag set by the "Remove group" button; it's the only thing that can bring a removed entry back re-editing that participant from that same group.

### Keeping chat profile and contact profile in sync — mirrored writes, not shared reads

`profiles(chat_id)` stays the single source of truth for what the chat's own Profile dialog and chat list read — **no changes to any existing read path** (`get_profile`, `get_chat_list`, `get_name_history`). Instead, writes mirror across the link when one exists:

- `update_profile(chat_id, ...)` — after its existing upsert, if `chats.contact_id` is set, mirror the same COALESCE-style upsert into `contacts`.
- `remove_profile_photo(chat_id)` — same mirroring for photo removal.
- New `update_contact_profile(contact_id, ...)` — upserts `contacts`, then mirrors into the linked chat's `profiles(chat_id)` row if one exists.
- New `remove_contact_photo(contact_id)` — mirrors similarly.

### New commands

- `get_or_create_contact_for_participant(participant_name, group_chat_id) -> Contact` — the core resolver:
  1. Normalize the participant string.
  2. Look for a 1-on-1 chat whose `original_name` normalizes to the same value; if found and unlinked, create a contact seeded from that chat's current profile data and link it; if already linked, return the linked contact.
  3. Else reuse an existing unlinked `contacts` row with that key, if any.
  4. Else create a fresh one — detect if the string looks like a phone number (`^\+?\d{7,15}$` after stripping spaces/dashes/parens) and prefill `phone_number` instead of `name` when it does.
  5. Either way, upsert a row into `contact_groups` for `(contact, group_chat_id)` with `excluded = 0`.
- `get_contact_profile(contact_id)` / `get_contact_groups(contact_id)` — reads for the contact-scoped profile dialog and its Linked Groups section.
- `remove_contact_group(contact_id, chat_id)` — soft-removes one group from the list (`excluded = 1`); never touches chat data or the 1-on-1 link.
- `get_linked_participants(participant_names) -> Vec<String>` — read-only, no side effects; returns which participant names already resolve to a contact that's linked to a real chat, so the Group Info dialog can show a small "linked" icon.
- `get_unlinked_one_on_one_chats()` / `link_contact_to_chat(contact_id, chat_id)` — manual-link picker for when auto-matching can't find (or got wrong) a match.
- `unlink_contact_from_chat(contact_id)` — clears `chats.contact_id` only; doesn't touch `contact_groups` or delete any data.

### Linking at import time

When a 1-on-1 chat is created/resolved during import (main import, share/base64 import, backup-ZIP restore), check for a matching unlinked `contacts` row by normalized name; if found, link it and copy its non-null fields down into the chat's `profiles` row (fields the contact never set are left as whatever the chat's own profile already had).

### Frontend

- `GroupParticipantsDialog.tsx` — add an edit icon per row (opens the resolver → Profile dialog) and a linked-status icon for rows `get_linked_participants` returns.
- `ProfileDialog.tsx` — stays a pure/controlled component; gains a handful of **optional** props (unused by the existing chat-profile flow) for: the Linked Groups collapsible section (same expand/collapse pattern as the existing Name History section) with a Remove button per group, and the Link/Unlink toggle area.
- The chat's own Profile dialog (when its chat is linked to a contact) also shows the same Linked Groups section and an Unlink button — the one existing read path that needs a small addition (checking `chats.contact_id` when loading a chat's profile).

## Known limitation

Automatic matching only works when the participant string and the 1-on-1 chat's `original_name` are literally the same text. If a group shows a raw phone number but the 1-on-1 chat was exported under a saved contact name, the strings won't match — the manual link picker is the fallback for this case.

## Implementation checklist

- [ ] `contacts` table + migration
  - **Test:** run the app once after the migration; confirm no crash on startup, then inspect the SQLite file (`PRAGMA table_info(contacts);`) to confirm the table and its columns exist.
- [ ] `chats.contact_id` column + migration
  - **Test:** same startup check; `PRAGMA table_info(chats);` shows the new nullable `contact_id` column, existing chats unaffected (still load/display normally).
- [ ] `contact_groups` table + migration
  - **Test:** `PRAGMA table_info(contact_groups);` shows the table with `contact_id`, `chat_id`, `excluded`.
- [ ] Mirrored writes: `update_profile`, `remove_profile_photo` updated; `update_contact_profile`, `remove_contact_photo` added
  - **Test:** link a contact to a chat (via auto-match or manual link), edit name/notes/phone from the chat's own Profile dialog, reopen the linked contact's profile (via a group participant) and confirm the new values appear there too. Repeat editing from the contact side and confirm it reflects back on the chat. Remove the photo from each side and confirm it clears on both.
- [ ] `get_or_create_contact_for_participant` (incl. phone-number detection heuristic)
  - **Test:** click edit on a participant whose export string is a raw phone number (e.g. `+31 6 13792894`) → confirm the phone field is prefilled and name is blank. Click edit on a participant whose export string is a saved contact name → confirm the name field is prefilled instead. Click edit twice on the same participant → confirm the second click reuses the same contact instead of creating a duplicate.
- [ ] `get_contact_profile`, `get_contact_groups`, `remove_contact_group`
  - **Test:** edit the same participant string from two different group chats → open either group's participant profile and confirm both groups are listed in Linked Groups. Click Remove on one → confirm it disappears from the list while the other group and any 1-on-1 link stay intact.
- [ ] `get_linked_participants`
  - **Test:** open Group Info on a group before any linking exists → no linked icon on any row. Import (or manually link) a matching 1-on-1 chat for one participant → reopen Group Info → confirm the linked icon now appears only on that participant's row.
- [ ] `get_unlinked_one_on_one_chats`, `link_contact_to_chat`, `unlink_contact_from_chat`
  - **Test:** create a shadow contact for a participant whose string won't auto-match anything, open its profile, use the "Link to existing chat" picker to link it to an unrelated existing 1-on-1 chat, confirm the copy-down and linked icon behave the same as an automatic link. Then click Unlink and confirm the chat keeps whatever profile data it had, the link/icon clears, and the picker reappears.
- [ ] Import-time auto-linking hook (all three import paths)
  - **Test:** create a shadow contact for a participant string, then import a 1-on-1 chat with a matching exported name through each of the three import paths in turn (normal ZIP import, share/base64 import, backup-ZIP restore) — confirm each one links automatically and copies the shadow data down into the chat's profile.
- [ ] `GroupParticipantsDialog.tsx`: edit icon + linked-status icon
  - **Test:** visually confirm every participant row has an edit icon, and only already-linked rows show the linked icon (per the `get_linked_participants` test above).
- [ ] `ProfileDialog.tsx`: Linked Groups section + Link/Unlink toggle (as optional props)
  - **Test:** open a contact's profile with 2+ associated groups and confirm the collapsible section (same expand/collapse behavior as Name History) lists all of them with working Remove buttons. Confirm the toggle area shows "Linked to: X" + Unlink when linked, and the chat picker + Link button when not. Confirm the existing chat-only Profile dialog (opened from a chat with no contact link) renders exactly as it did before this feature — no new sections, no visual change.
- [ ] `App.tsx` wiring: `handleEditParticipant`, contact-scoped profile state, chat-side Linked Groups lookup
  - **Test:** full click-through — Group Info → click edit on a participant → dialog opens correctly populated → Save persists (reopen to confirm) → Cancel discards unsaved edits. Also open a linked 1-on-1 chat's own Profile dialog and confirm it now shows the Linked Groups section and Unlink button.
- [ ] `Contact` type (Rust struct + TS type)
  - **Test:** `cargo check` (project-code/src-tauri) and `npx tsc --noEmit` (project-code) both pass with no type errors.
- [ ] Manual end-to-end verification pass
  - **Test:** run through every test above in one sitting, in order, against a real imported archive (at least one group chat with 2+ participants and at least one matching 1-on-1 chat available to import partway through).

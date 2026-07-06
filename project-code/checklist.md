# WhatsApp Archive Viewer — Functionality Checklist

## Import / Export

- [ ] Import single chat from ZIP file
- [ ] Import multiple chats from ZIP files (batch)
- [ ] Export single chat to ZIP
- [ ] Export all chats to ZIP
- [ ] Import from export ZIP (restore)
- [ ] Import from share (base64 ZIP bytes)
- [ ] Migrate chats from old JSON format
- [ ] Get import progress status

## Chat Management

- [ ] Get chat list (all chats with metadata)
- [ ] Get chat messages (with pagination: limit/offset)
- [ ] Get chat message count
- [ ] Delete single chat
- [ ] Clear all chats
- [ ] Migrate chats (database schema upgrade)

## Search

- [ ] Search messages within a chat (simple query)
- [ ] Search messages with filters (date range, sender, type)
- [ ] Search chats (across all chats by name)

## Message Viewing

- [ ] Get media as base64 (images, videos, audio)
- [ ] Get media with dimensions (width/height for layout)
- [ ] Get media base directory path
- [ ] Get media file path
- [ ] Check if file exists in ZIP
- [ ] Extract file from ZIP to disk
- [ ] Preload media (batch extract for performance)
- [ ] Debug chat media (list all files in chat)

## Message Editing

- [ ] Set display name (rename sender in message)
- [ ] Set file tag (add custom tag to media)
- [ ] Rename media file
- [ ] Set message type (change type classification)
- [ ] Toggle message favorite (star/unstar)
- [ ] Get favorite messages (view all starred)
- [ ] Export chat modifications (save edits to JSON)
- [ ] Apply chat modifications (restore from JSON)
- [ ] Clear modification flags (reset "edited" status)

## Profiles / Contacts

- [ ] Get profile (name, notes, photo, phone)
- [ ] Update profile (edit name, notes, photo, phone)
- [ ] Remove profile photo
- [ ] Get name history (view past name changes)
- [ ] Revert profile name (restore old name)
- [ ] Pick profile photo (file picker)
- [ ] Parse vCard (extract contact info from .vcf)
- [ ] Open vCard in WhatsApp (share contact back to app)

## Backgrounds

- [ ] List default backgrounds
- [ ] Pick global background (file picker)
- [ ] Set chat background (per-chat)
- [ ] Get chat background (current)
- [ ] Get background history (view past backgrounds)
- [ ] Restore chat background (revert to previous)
- [ ] Clear chat background (remove)
- [ ] Save background from base64 (for drag-drop)

## File Operations

- [ ] Open URL (in default browser)
- [ ] Open media file (in default app)
- [ ] Check file exists (utility)
- [ ] Read file as base64 (utility)

## Platform-Specific (Windows)

- [ ] Check VC++ Redistributable installed
- [ ] Install VC++ Redistributable

## Platform-Specific (Non-Windows)

- [ ] Check VC++ Redistributable (stub)
- [ ] Install VC++ Redistributable (stub)

## Share / Interop

- [ ] Get pending share (for Android share intent)

## Frontend Features (React)

### Navigation & UI
- [ ] Chat list view with virtual scrolling
- [ ] Message list view with virtual scrolling
- [ ] Jump to top/bottom buttons
- [ ] Loading overlay
- [ ] Error boundary

### Message Display
- [ ] Text messages with clickable links
- [ ] Image display (lazy loaded)
- [ ] Video player
- [ ] Audio player
- [ ] Sticker display
- [ ] GIF player
- [ ] File attachments with type badges
- [ ] Location messages (Google Maps / Apple Maps)
- [ ] System messages (end-to-end encryption, etc.)
- [ ] Expandable text for long messages

### Chat Customization
- [ ] Chat background (custom image)
- [ ] Group avatar display
- [ ] Profile image display
- [ ] Dark mode support

### Editing Tools
- [ ] Rename sender (display name)
- [ ] Add custom file tags
- [ ] Rename media files
- [ ] Change message type
- [ ] Star/favorite messages
- [ ] View favorites list

### Search
- [ ] In-chat search with highlighting
- [ ] Filter by date range
- [ ] Filter by sender
- [ ] Filter by message type

### Dialogs
- [ ] Delete confirmation dialog
- [ ] Export success dialog
- [ ] Username dialog
- [ ] Group participants dialog
- [ ] Profile dialog
- [ ] Link confirmation modal (for URL safety)

### Multi-Select
- [ ] Long-press to select messages
- [ ] Shift-click range selection
- [ ] Bulk actions on selected messages

### Utilities
- [ ] Date formatting (various formats)
- [ ] Time formatting
- [ ] Phone number parsing
- [ ] Text rendering with link detection
- [ ] Media fallback handling (corrupted/missing files)

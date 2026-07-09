# WhatsApp Archive Viewer — Functionality Checklist

---

## Importing

- [ ] Import a single WhatsApp export ZIP via file picker
- [ ] Import multiple ZIP files at once (batch — picks multiple files, imports all)
- [ ] Import from an app backup ZIP (restores a previously exported backup; merges into existing chat if it already exists)
- [ ] Progress indicator shows during import (phases: extracting files → preparing database → saving messages)
- [ ] After import, the newly imported chat is auto-selected and opened

---

## Exporting / Backup

- [ ] Export a single chat to a ZIP file (saves messages, media, and any edits you've made)
- [ ] Export all chats to one ZIP file
- [ ] Export is cancelled gracefully when you close the save dialog without choosing a location

---

## Chat List

- [ ] All imported chats are listed, sorted by most recent message
- [ ] Each chat shows: name, last message preview, and timestamp
- [ ] Group chats are visually distinguishable from 1-on-1 chats
- [x] Search bar filters the chat list by name
- [ ] Start screen is shown when no chats have been imported yet

---

## Reading Messages

- [ ] Messages load and scroll smoothly (thousands of messages handled without slowdown)
- [ ] Text messages display correctly, including line breaks
- [ ] Long messages have an expand/collapse button
- [ ] URLs in messages are clickable (opens a confirmation dialog before going to the browser)
- [ ] Sent and received messages are visually distinct (aligned left/right)
- [ ] Message timestamps are shown
- [ ] System messages display (e.g. "Messages and calls are end-to-end encrypted")
- [ ] Jump to top / jump to bottom buttons appear when scrolled away from the ends

---

## Media in Messages

- [ ] Images display inline (lazy-loaded; only loads when scrolled into view)
- [ ] Videos play with controls (play/pause, volume, seek)
- [ ] Audio messages play with a player
- [ ] Animated GIFs play inline
- [ ] Stickers display correctly
- [ ] File attachments show with a file type badge and an Open button
- [ ] Location messages show a link to Google Maps or Apple Maps
- [ ] Contact cards (.vcf files) show an "Add Contact" button
- [ ] Poll messages show the question, options, and vote counts
- [x] Missing or corrupted media shows a fallback placeholder with the filename

---

## Media Gallery

- [ ] Opening the gallery shows all media from the current chat in a separate panel
- [ ] Gallery has four tabs: Images, Videos & GIFs, Audio, Files
- [ ] Each tab only shows media of that type
- [ ] Sort order can be switched between newest and oldest
- [ ] Clicking an image opens it in the lightbox viewer
- [ ] Clicking a video or GIF opens it in the lightbox viewer
- [ ] Lightbox: previous/next arrows navigate between items in the current tab
- [ ] Lightbox: left/right arrow keys also navigate; Escape closes it
- [ ] Lightbox: favorite toggle button works (star/unstar the message)
- [ ] Lightbox shows the sender name and timestamp of the item
- [ ] Audio and file items in the gallery can be opened directly

---

## Search

- [x] Search bar opens an overlay within the current chat
- [x] Matching messages are highlighted
- [ ] Previous/next buttons step through results one by one
- [ ] Result counter shows current position (e.g. "2 / 14")
- [ ] Clicking a result scrolls to and highlights that message in the chat
- [ ] Advanced search: filter by date range
- [ ] Advanced search: filter by sender
- [ ] Advanced search: filter by message type (image, video, audio, etc.)
- [ ] Closing search clears all highlights

---

## Favorites (Starred Messages)

- [x] Star button on a message marks it as a favorite
- [x] Unstarring removes it from favorites
- [x] Favorites panel shows all starred messages from the current chat
- [ ] Clicking a favorite in the panel jumps to that message in the chat and highlights it

---

## Multi-Select

- [ ] Long-pressing (holding) a message enters multi-select mode
- [ ] In multi-select mode, clicking additional messages adds/removes them from the selection
- [ ] Shift-clicking selects all messages between the last selected and the clicked one
- [ ] Bulk action: toggle favorite on all selected messages
- [ ] Exiting multi-select mode clears the selection

---

## Editing Messages

- [ ] Right-click (or context menu) on a message opens editing options
- [ ] Set a display name for a sender (replaces their name throughout the chat)
- [ ] Add a custom file tag to a media message (relabels its type)
- [x] Rename a media file
- [ ] Change a message's type classification

---

## Contact Cards (vCard)

- [ ] Clicking "Add Contact" on a .vcf file opens a choice dialog
- [ ] "View contact info" shows the contact's name, phone numbers, and email addresses inline
- [ ] "Open with default app" opens the .vcf file in your system's default contacts app
- [ ] "WhatsApp App" opens the contact in the WhatsApp desktop/mobile app
- [ ] "WhatsApp Web" opens the contact via WhatsApp Web in the browser

---

## Profiles

- [x] Opening a chat's profile shows: name, notes, phone number, and profile photo
- [x] Name can be edited and saved
- [x] Notes field can be edited and saved
- [x] Phone number can be edited and saved
- [x] Profile photo can be changed by picking an image file
- [x] Profile photo can be removed
- [x] Name history shows all previous names for this contact
- [x] Reverting to a previous name restores it as the current display name
- [ ] Resetting the name clears any custom name (reverts to the original from the chat)

---

## Group Chats

- [ ] Group avatar (multi-person icon) is shown for group chats in the chat list
- [ ] Group participants dialog lists all members extracted from the chat
- [ ] Username dialog lets you set which sender name is "you" in this conversation

---

## Chat Background

- [x] Background settings dialog opens from the chat header
- [x] Default tab shows built-in background images to choose from
- [x] Custom tab lets you pick any image file from your computer as the background
- [ ] A preview of the selected background is shown before confirming
- [ ] Saving applies the background to the chat view immediately
- [ ] Background history shows previously used custom backgrounds
- [ ] A background from history can be re-applied
- [ ] Individual history entries can be deleted
- [ ] Entire background history can be cleared
- [ ] Background can be cleared (reset to no background)

---

## Appearance

- [ ] Dark mode toggle switches the entire UI to a dark theme
- [ ] Light mode toggle switches back to the light theme
- [ ] Theme persists after switching chats

---

## Deleting Chats

- [ ] Delete button on a chat triggers a confirmation dialog
- [ ] If the chat has edits (renames, tags, stars), "Keep Changes & Delete" is offered
  - [ ] Choosing this saves the edits to local storage and deletes the chat
  - [ ] A "Changes Saved" dialog confirms the edits will be re-applied on next import
- [ ] "Delete Everything" deletes the chat and discards all edits
  - [ ] A "Deleted" success dialog confirms the chat was removed
- [ ] "Clear All Chats" removes all chats at once (with confirmation)

---

## Windows — First Run

- [ ] App detects whether the VC++ Redistributable is installed
- [ ] If missing, a prompt appears offering to install it
- [ ] Clicking install runs the bundled VC++ installer

---

## General UI

- [ ] Toast notifications appear briefly for quick feedback (e.g. "Background saved!", "Import failed")
- [ ] Error boundary catches unexpected crashes and shows an error message instead of a blank screen
- [ ] Link confirmation modal appears before opening any external URL, showing the full URL
- [ ] Confirming the link opens it in the default browser
- [ ] Cancelling the link modal does nothing

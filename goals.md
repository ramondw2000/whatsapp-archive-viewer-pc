# 🚧 Current Status & Known Issues

### Root Cause Analysis
WhatsApp exports have media in subdirectories (e.g., `WhatsApp Images/IMG-20260317-WA0002.jpg`), but the chat `.txt` file only references the filename (`IMG-20260317-WA0002.jpg`). This creates a path mismatch:
- **Database stores**: `IMG-20260317-WA0002.jpg`
- **File is at**: `chats/{id}/media/WhatsApp Images/IMG-20260317-WA0002.jpg`
- **App looks for**: `chats/{id}/media/IMG-20260317-WA0002.jpg`

### Attempted Fixes

#### Fix 1: URI prefix stripping for Windows WebView2 ❌ (did not resolve)
- **Hypothesis**: Tauri v2 on Windows (WebView2) rewrites `media://localhost/...` → `http://media.localhost/...`
- Added `http://media.localhost/` and `https://media.localhost/` as first-priority prefix strips in the handler
- Added `[MEDIA] Raw URI:` log to capture the actual incoming URI
- **Result**: `[MEDIA]` logs still never appear — the handler is not being called at all

#### Fix 2: CSP changes ❌ (did not resolve)
- Added `devCsp` with permissive settings for dev mode
- Added `dangerousDisableAssetCspModification: true` to prevent Tauri overriding our CSP
- Updated `img-src` and `media-src` to explicitly allow `http://media.localhost`
- **Result**: No change — images still broken, `[MEDIA]` handler still never fires

#### Fix 3: Switch to `convertFileSrc` + asset protocol ❌ (did not resolve)
- **Root hypothesis**: Custom `media://` protocol is silently blocked before reaching Rust handler
- Enabled Tauri's built-in `assetProtocol` with `scope: ["**"]` in `tauri.conf.json`
- Added `get_media_base_dir` Rust command returning absolute path to chat's media folder
- Frontend: replaced `media://localhost/${chatId}/${filename}` with `convertFileSrc(\`${mediaBaseDir}\\${filename}\`)`
- Passed `mediaBaseDir` as prop to `MediaGallery` component; updated `mediaSrc` helper
- Imported `convertFileSrc` from `@tauri-apps/api/core`
- **Result**: Still broken — images not loading

#### Debug Logging Added
- `[IMPORT]` - Shows media files found in ZIP and copy operations ✅ fires correctly
- `[PARSE]` - Shows filename extraction from chat text
- `[MEDIA]` - Shows file lookup attempts and paths ⚠️ never fires — handler not reached

### What We Know For Certain
1. **Files are on disk** — `[IMPORT]` logs confirm media copies successfully to `chats/{id}/media/`
2. **Frontend builds URLs** — `convertFileSrc` should produce `http://asset.localhost/C:/Users/...` URLs
3. **CSP allows `asset:`** — `img-src` includes `asset: https://asset.localhost http://asset.localhost`
4. **Handler never called** — neither `media://` nor `asset://` protocol handler fires on image load
5. **Root cause unknown** — something is silently swallowing the request before it reaches Rust

### Next Steps / Ideas 💡

#### Immediate — Need to Diagnose
1. **Open browser devtools** in the Tauri app (`F12` or `Ctrl+Shift+I`) — check Console and Network tabs for actual errors when images fail to load
2. **Check what URL the `<img>` tag actually has** — inspect element to see the `src` attribute value and whether it's a valid `asset://` URL
3. **Try serving via Tauri command** — return image as base64 data URL from an `invoke()` call, bypassing protocols entirely

#### Potential Solutions to Try
1. **Base64 data URL approach** — `invoke("get_media_as_base64", {chatId, filename})` → `data:image/jpeg;base64,...` — bypasses all protocol/CSP issues entirely
2. **Store full relative path** — Modify parser to include subdirectory in database
3. **Hybrid approach** — Store both filename and original path, try both lookups
4. **File extension fallback** — If exact filename fails, look for files with same extension
5. **In-app media repair** — Button to "re-index" media for a chat

#### New Features to Add
- Export chat as HTML/PDF
- Backup/restore all chats
- Keyboard shortcuts
- Message reactions
- Voice message transcription
- Statistics dashboard (messages per day, most active times, etc.)
- Chat comparison/analytics
- Favorite/bookmark messages
- Voice notes playback with waveform
- Inline image thumbnails in chat (like WhatsApp mobile)

### Technical Debt
- Parser regex patterns could be consolidated
- Database schema versioning needed for future migrations
- Error handling in UI could be more user-friendly
- Need automated tests for parser with various WhatsApp export formats

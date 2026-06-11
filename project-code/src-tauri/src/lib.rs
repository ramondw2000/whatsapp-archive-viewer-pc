use std::fs::{self, File};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;

static IMPORT_MSG_COUNT: AtomicUsize = AtomicUsize::new(0);
static IMPORT_MEDIA_COUNT: AtomicUsize = AtomicUsize::new(0);
static IMPORT_ACTIVE: AtomicUsize = AtomicUsize::new(0);
static IMPORT_PHASE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
// phases: 0=idle, 1=extracting, 2=parsing, 3=saving to db, 4=done
static BACKFILL_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static TS_RE_SLASH_FULL: OnceLock<regex::Regex> = OnceLock::new();
static TS_RE_SLASH_SHORT: OnceLock<regex::Regex> = OnceLock::new();
static TS_RE_DASH_FULL: OnceLock<regex::Regex> = OnceLock::new();
static TS_RE_DOT_FULL: OnceLock<regex::Regex> = OnceLock::new();
static SVG_RE_WIDTH: OnceLock<regex::Regex> = OnceLock::new();
static SVG_RE_HEIGHT: OnceLock<regex::Regex> = OnceLock::new();

// ============================================================================
// GLOBAL DATABASE CONNECTION
// ============================================================================

fn get_db() -> std::sync::MutexGuard<'static, Connection> {
    static DB: OnceLock<Mutex<Connection>> = OnceLock::new();
    let mutex = DB.get_or_init(|| {
        let conn = init_database().expect("Failed to initialize SQLite database");
        Mutex::new(conn)
    });
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("[DB] Mutex was poisoned; recovering connection");
            poisoned.into_inner()
        }
    }
}

// ============================================================================
// MEDIA PRELOAD CACHE
// ============================================================================

/// LRU cache for preloaded media base64 data
/// Key: "chat_id:filename", Value: base64 data URL
struct MediaPreloadCache {
    cache: HashMap<String, String>,
    order: Vec<String>,
    max_size: usize,
}

impl MediaPreloadCache {
    fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::with_capacity(max_size),
            order: Vec::with_capacity(max_size),
            max_size,
        }
    }

    fn get(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.cache.get(key) {
            // Move to end (most recently used)
            self.order.retain(|k| k != key);
            self.order.push(key.to_string());
            Some(value.clone())
        } else {
            None
        }
    }

    fn set(&mut self, key: String, value: String) {
        if self.cache.contains_key(&key) {
            // Update existing
            self.cache.insert(key.clone(), value);
            self.order.retain(|k| k != &key);
            self.order.push(key);
        } else {
            // Insert new
            if self.cache.len() >= self.max_size {
                // Remove oldest
                if !self.order.is_empty() {
                    let oldest = self.order.remove(0);
                    self.cache.remove(&oldest);
                }
            }
            self.cache.insert(key.clone(), value);
            self.order.push(key);
        }
    }


}

// Global media preload cache with 100 entry capacity
fn get_media_preload_cache() -> &'static Mutex<MediaPreloadCache> {
    static CACHE: OnceLock<Mutex<MediaPreloadCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(MediaPreloadCache::new(100)))
}

use std::io::Read;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use uuid::Uuid;

use zip::ZipArchive;
use zip::write::{ZipWriter, SimpleFileOptions};
use std::io::Write;

use rusqlite::{Connection, Result as SqliteResult, params};

use percent_encoding::percent_decode_str;

#[tauri::command]
fn check_file_exists(path: String) -> bool {
    let app_data = get_app_data_dir();
    let p = std::path::Path::new(&path);
    if !p.starts_with(&app_data) {
        return false;
    }
    p.exists()
}

/// Strip path traversal — return only the final filename component.
/// Returns Err if the result is empty or purely dot-composed.
fn sanitize_filename(s: &str) -> Result<String, String> {
    let name = std::path::Path::new(s)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;
    if name.bytes().all(|b| b == b'.') {
        return Err("Invalid filename".to_string());
    }
    Ok(name.to_string())
}

/// Ensure chat_id is a plain identifier with no path separators or dot-dot.
fn validate_chat_id(id: &str) -> Result<(), String> {
    if id.is_empty() || id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err("Invalid chat_id".to_string());
    }
    Ok(())
}

/// Extension allowlist — only permit known-safe media types through open_path.
fn is_safe_open_extension(ext: &str) -> bool {
    matches!(ext,
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "avif" |
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "m4v" | "3gp" |
        "mp3" | "m4a" | "aac" | "ogg" | "opus" | "flac" | "wav" |
        "pdf" | "vcf" | "ico"
    )
}

/// Allow only http:// and https:// URLs to be handed to the OS opener.
fn validate_url_scheme(url: &str) -> Result<(), String> {
    let l = url.to_lowercase();
    if l.starts_with("https://") || l.starts_with("http://") {
        Ok(())
    } else {
        Err("URL scheme not permitted".to_string())
    }
}



#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Message {
    #[serde(default)]
    pub id: Option<i64>,
    pub timestamp: String,

    pub sender: String,

    #[serde(rename = "type")]

    pub msg_type: String,

    pub content: String,

    pub media: Option<String>,

    pub duration: Option<String>,
    #[serde(default)]
    pub tag_ext: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub is_favorite: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatData {
    pub messages: Vec<Message>,
}



#[derive(Serialize, Deserialize, Debug)]

pub struct ChatMeta {

    pub id: String,

    pub name: String,

    pub last_message: String,

    pub timestamp: String,

    pub is_group: bool,

    pub zip_path: Option<String>,

    pub photo_path: Option<String>,

}



fn get_app_data_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Could not find data directory")
        .join("WhatsAppArchiveViewer")
}



fn ensure_dir_exists(path: &Path) {

    let _ = fs::create_dir_all(path);

}



fn is_plausible_whatsapp_date(date_str: &str, time_str: &str) -> bool {
    // Extract numeric parts from date (supports dd/mm/yyyy, dd-mm-yyyy, dd.mm.yyyy)
    let parts: Vec<&str> = date_str.split(|c| c == '/' || c == '-' || c == '.').collect();
    if parts.len() < 3 { return false; }

    let (a, b, c) = (
        parts[0].parse::<u32>().unwrap_or(0),
        parts[1].parse::<u32>().unwrap_or(0),
        parts[2].parse::<u32>().unwrap_or(0),
    );

    // Try dd/mm/yyyy or mm/dd/yyyy — either way values must be in plausible range
    let day_ok = (a >= 1 && a <= 31) || (b >= 1 && b <= 31);
    let month_ok = (a >= 1 && a <= 12) || (b >= 1 && b <= 12);
    let year = if c > 31 { c } else if a > 31 { a } else { 0 };
    let year_ok = (year >= 2009 && year <= 2099) || (c >= 9 && c <= 99); // 2-digit years

    if !day_ok || !month_ok || !year_ok { return false; }

    // Time must have valid hour and minute
    let time_core = time_str.trim_end_matches(|c: char| c.is_alphabetic() || c == ' ');
    let tparts: Vec<&str> = time_core.split(':').collect();
    if tparts.is_empty() { return false; }
    let hour = tparts[0].trim().parse::<u32>().unwrap_or(99);
    let minute = tparts.get(1).unwrap_or(&"0").trim().parse::<u32>().unwrap_or(99);

    hour <= 23 && minute <= 59
}

fn get_archive_extension(path: &str) -> String {

    Path::new(path)

        .extension()

        .and_then(|e| e.to_str())

        .unwrap_or("")

        .to_lowercase()

}



fn get_db_path() -> PathBuf {

    let app_data = get_app_data_dir();

    app_data.join("chats.db")

}



#[allow(dead_code)]
/// Read duration in seconds from an MP4 file by scanning for the mvhd box.

/// Returns None if not an MP4 or duration can't be read.

fn mp4_duration_secs(file_path: &Path) -> Option<f64> {

    let mut f = File::open(file_path).ok()?;

    let file_len = f.metadata().ok()?.len();

    let mut buf = vec![0u8; file_len.min(1_000_000) as usize];

    f.read(&mut buf).ok()?;



    // Scan for 'mvhd' marker

    let marker = b"mvhd";

    let pos = buf.windows(4).position(|w| w == marker)?;

    let data = buf.get(pos + 4..)?; // skip the 4-byte 'mvhd' tag



    // version byte: 0 = 32-bit fields, 1 = 64-bit fields

    let version = *data.first()?;

    if version == 0 {

        // skip version(1) + flags(3) + creation(4) + modification(4) = 12 bytes

        let timescale = u32::from_be_bytes(data.get(12..16)?.try_into().ok()?) as f64;

        let duration  = u32::from_be_bytes(data.get(16..20)?.try_into().ok()?) as f64;

        if timescale > 0.0 { return Some(duration / timescale); }

    } else if version == 1 {

        // skip version(1) + flags(3) + creation(8) + modification(8) = 20 bytes

        let timescale = u32::from_be_bytes(data.get(20..24)?.try_into().ok()?) as f64;

        let duration  = u64::from_be_bytes(data.get(24..32)?.try_into().ok()?) as f64;

        if timescale > 0.0 { return Some(duration / timescale); }

    }

    None

}



fn get_audio_duration(file_path: &Path) -> Option<String> {

    use symphonia::core::formats::FormatOptions;

    use symphonia::core::io::MediaSourceStream;

    use symphonia::core::probe::Hint;

    use symphonia::core::meta::MetadataOptions;

    

    let file = match File::open(file_path) {

        Ok(f) => f,

        Err(_) => return None,

    };

    

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    

    let mut hint = Hint::new();

    if let Some(extension) = file_path.extension() {

        if let Some(ext_str) = extension.to_str() {

            hint.with_extension(ext_str);

        }

    }

    

    let meta_opts: FormatOptions = Default::default();

    let metadata_opts: MetadataOptions = Default::default();

    

    match symphonia::default::get_probe().format(&hint, mss, &meta_opts, &metadata_opts) {

        Ok(probed) => {

            let format = probed.format;

            let track = match format.tracks().iter().find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL) {

                Some(track) => track,

                None => return None,

            };

            

            if let Some(time_base) = track.codec_params.time_base {

                if let Some(n_frames) = track.codec_params.n_frames {

                    let duration_secs = n_frames as f64 * time_base.numer as f64 / time_base.denom as f64;

                    let minutes = (duration_secs / 60.0) as u32;

                    let seconds = (duration_secs % 60.0) as u32;

                    return Some(format!("{:02}:{:02}", minutes, seconds));

                }

            }

        }

        Err(_) => return None,

    }

    

    None

}



fn init_database() -> SqliteResult<Connection> {

    let db_path = get_db_path();

    // Ensure parent directory exists before opening
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let conn = Connection::open(&db_path)?;

    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")?;

    

    // Create chats table

    conn.execute(

        "CREATE TABLE IF NOT EXISTS chats (

            id TEXT PRIMARY KEY,

            name TEXT NOT NULL,

            last_message TEXT,

            timestamp TEXT,

            is_group INTEGER NOT NULL DEFAULT 0,

            zip_path TEXT,

            created_at DATETIME DEFAULT CURRENT_TIMESTAMP

        )",

        [],

    )?;



    // Migration: Add zip_path column if it doesn't exist

    let has_zip_path: bool = conn.query_row(

        "SELECT COUNT(*) FROM pragma_table_info('chats') WHERE name = 'zip_path'",

        [],

        |row| row.get::<_, i64>(0).map(|count| count > 0)

    ).unwrap_or(false);

    

    if !has_zip_path {

        conn.execute("ALTER TABLE chats ADD COLUMN zip_path TEXT", [])

            .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    }

    

    // Create messages table

    conn.execute(

        "CREATE TABLE IF NOT EXISTS messages (

            id INTEGER PRIMARY KEY AUTOINCREMENT,

            chat_id TEXT NOT NULL,

            timestamp TEXT NOT NULL,

            sender TEXT NOT NULL,

            msg_type TEXT NOT NULL DEFAULT 'text',

            content TEXT NOT NULL,

            media TEXT,

            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE

        )",

        [],

    )?;

    

    // Create index for faster message retrieval

    conn.execute(

        "CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id)",

        [],

    )?;

    

    // Create profiles table

    conn.execute(

        "CREATE TABLE IF NOT EXISTS profiles (

            chat_id TEXT PRIMARY KEY,

            name TEXT,

            notes TEXT,

            photo_path TEXT,

            phone_number TEXT,

            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE

        )",

        [],

    )?;



    // Add phone_number column if it doesn't exist yet (safe for existing DBs)

    let _ = conn.execute(

        "ALTER TABLE profiles ADD COLUMN phone_number TEXT",

        [],

    );

    

    // Add flags to track manual modifications

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN display_name_modified INTEGER DEFAULT 0",

        [],

    );

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN tag_ext_modified INTEGER DEFAULT 0",

        [],

    );

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN msg_type_modified INTEGER DEFAULT 0",

        [],

    );

    

    // Add profile_modified column to track manual profile changes

    let _ = conn.execute(

        "ALTER TABLE profiles ADD COLUMN profile_modified INTEGER DEFAULT 0",

        [],

    );



    // Add last_message_epoch column if it doesn't exist yet (safe for existing DBs)

    let _ = conn.execute(

        "ALTER TABLE chats ADD COLUMN last_message_epoch INTEGER NOT NULL DEFAULT 0",

        [],

    );



    // Add original_name column if it doesn't exist yet (safe for existing DBs)

    let _ = conn.execute(

        "ALTER TABLE chats ADD COLUMN original_name TEXT",

        [],

    );



    // Add tag_ext column for manual file type tagging

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN tag_ext TEXT",

        [],

    );



    // Add display_name column for manual file rename

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN display_name TEXT",

        [],

    );



    // Backfill original_name from name for rows that predate this column

    let _ = conn.execute(

        "UPDATE chats SET original_name = name WHERE original_name IS NULL",

        [],

    );



    // Migrate stickers: fix rows stored as "image" where media filename starts with STK-

    let _ = conn.execute(

        "UPDATE messages SET msg_type = 'sticker' WHERE msg_type = 'image' AND (media LIKE 'STK-%' OR media LIKE '\u{200e}STK-%')",

        [],

    );



    // Migrate stickers stored as "text" with bare filename (no attachment marker in export)

    let _ = conn.execute(

        "UPDATE messages SET msg_type = 'sticker', media = content WHERE msg_type = 'text' AND (content LIKE 'STK-%.webp' OR content LIKE 'STK-%.WEBP')",

        [],

    );



    // Migrate audio files previously stored as "file" due to missing extension support

    for ext in &["opus", "3gp", "3gpp", "amr", "flac", "m4a", "aac"] {

        let _ = conn.execute(

            &format!("UPDATE messages SET msg_type = 'audio' WHERE msg_type = 'file' AND (media LIKE '%.{ext}' OR media LIKE '%.{upper}')",

                ext = ext, upper = ext.to_uppercase()),

            [],

        );

    }



    // Migrate m4v/ts video files previously stored as "file"

    for ext in &["m4v", "ts"] {

        let _ = conn.execute(

            &format!("UPDATE messages SET msg_type = 'video' WHERE msg_type = 'file' AND (media LIKE '%.{ext}' OR media LIKE '%.{upper}')",

                ext = ext, upper = ext.to_uppercase()),

            [],

        );

    }



    // Reclassify 3gp previously stored as "video" — WhatsApp 3gp files are voice notes

    let _ = conn.execute(

        "UPDATE messages SET msg_type = 'audio' WHERE msg_type = 'video' AND (media LIKE '%.3gp' OR media LIKE '%.3GP' OR media LIKE '%.3gpp' OR media LIKE '%.3GPP')",

        [],

    );



    // Migrate heic/heif images previously stored as "file"

    for ext in &["heic", "heif", "svg"] {

        let _ = conn.execute(

            &format!("UPDATE messages SET msg_type = 'image' WHERE msg_type = 'file' AND (media LIKE '%.{ext}' OR media LIKE '%.{upper}')",

                ext = ext, upper = ext.to_uppercase()),

            [],

        );

    }

    

    // Migrate TIFF images - update ALL tiff files to image type (unless manually modified)

    for ext in &["tiff", "tif"] {

        let result = conn.execute(

            &format!("UPDATE messages SET msg_type = 'image' WHERE (msg_type_modified IS NULL OR msg_type_modified = 0) AND (media LIKE '%.{ext}' OR media LIKE '%.{upper}')",

                ext = ext, upper = ext.to_uppercase()),

            [],

        );

        let _ = result;

    }



    // Migrate location messages previously stored as "text"

    for pattern in &["maps.google.com", "maps.apple.com", "goo.gl/maps", "maps.app.goo.gl"] {

        let _ = conn.execute(

            &format!(
    "UPDATE messages SET msg_type = 'location', media = (
                SELECT trim(word) FROM (
                    WITH RECURSIVE split(word, rest) AS (
                        SELECT '', content || ' '
                        UNION ALL SELECT substr(rest, 0, instr(rest, ' ')), substr(rest, instr(rest, ' ') + 1)
                        FROM split WHERE rest != ''
                    ) SELECT word FROM split WHERE instr(word, '{pattern}') > 0 LIMIT 1
                )
            ) WHERE msg_type = 'text' AND content LIKE '%{pattern}%'"
),

            [],

        );

    }





    // Create name history table

    conn.execute(

        "CREATE TABLE IF NOT EXISTS chat_name_history (

            id INTEGER PRIMARY KEY AUTOINCREMENT,

            chat_id TEXT NOT NULL,

            name TEXT NOT NULL,

            changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,

            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE

        )",

        [],

    )?;



    // Add duration column if it doesn't exist yet (safe for existing DBs)

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN duration TEXT",

        [],

    );



    // Add background_path column to profiles if it doesn't exist yet

    let _ = conn.execute(

        "ALTER TABLE profiles ADD COLUMN background_path TEXT",

        [],

    );

    // Add is_favorite column to messages if it doesn't exist yet

    let _ = conn.execute(

        "ALTER TABLE messages ADD COLUMN is_favorite INTEGER DEFAULT 0",

        [],

    );



    // Create background history table

    conn.execute(

        "CREATE TABLE IF NOT EXISTS chat_background_history (

            id INTEGER PRIMARY KEY AUTOINCREMENT,

            chat_id TEXT NOT NULL,

            background_path TEXT NOT NULL,

            changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,

            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE

        )",

        [],

    )?;



    Ok(conn)

}



fn backfill_epochs(conn: &Connection) {

    // Only run once per app session — this is a one-time migration for old data
    if BACKFILL_DONE.swap(true, Ordering::Relaxed) { return; }

    // Fix any chats that still have last_message_epoch = 0 (imported before this column existed)

    let chat_ids: Vec<String> = {

        let mut stmt = match conn.prepare(

            "SELECT id FROM chats WHERE last_message_epoch = 0"

        ) {

            Ok(s) => s,

            Err(_) => return,

        };

        stmt.query_map([], |row| row.get::<_, String>(0))

            .map(|rows| rows.flatten().collect())

            .unwrap_or_default()

    };



    for chat_id in chat_ids {

        let mut stmt = match conn.prepare(

            "SELECT timestamp FROM messages WHERE chat_id = ?1"

        ) {

            Ok(s) => s,

            Err(_) => continue,

        };

        let max_epoch = stmt

            .query_map(params![&chat_id], |row| row.get::<_, String>(0))

            .map(|rows| {

                rows.flatten()

                    .map(|ts| timestamp_to_epoch(&ts))

                    .max()

                    .unwrap_or(0)

            })

            .unwrap_or(0);



        if max_epoch > 0 {

            let _ = conn.execute(

                "UPDATE chats SET last_message_epoch = ?1 WHERE id = ?2",

                params![max_epoch, &chat_id],

            );

        }

    }

}



use tauri_plugin_dialog::DialogExt;

use tauri::Manager;



#[tauri::command]

fn list_default_backgrounds(app: tauri::AppHandle) -> Vec<String> {

    let image_exts = ["png", "jpg", "jpeg", "jfif", "gif", "webp", "bmp", "tiff", "tif", "avif", "svg", "ico"];

    // Try multiple possible locations
    let candidates: Vec<std::path::PathBuf> = vec![
        // Resource dir (bundled)
        app.path().resource_dir()
            .map(|p| p.join("app_backgrounds"))
            .unwrap_or_default(),
        // Next to exe
        std::env::current_exe().ok()
            .and_then(|e| e.parent().map(|p| p.join("resources").join("app_backgrounds")))
            .unwrap_or_default(),
        // In resources folder next to exe
        std::env::current_exe().ok()
            .and_then(|e| e.parent().map(|p| p.join("app_backgrounds")))
            .unwrap_or_default(),
    ];

    for backgrounds_path in &candidates {
        if backgrounds_path.exists() {
            if let Ok(entries) = fs::read_dir(backgrounds_path) {
                let mut files: Vec<String> = entries
                    .flatten()
                    .filter_map(|e| {
                        let path = e.path();
                        let ext = path.extension()?.to_str()?.to_lowercase();
                        if image_exts.contains(&ext.as_str()) {
                            path.file_name().map(|n| n.to_string_lossy().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
                files.sort();
                return files;
            }
        }
    }

    vec![]

}



#[tauri::command]

async fn pick_zip_file(app: tauri::AppHandle) -> Result<Option<String>, String> {

    let (tx, rx) = tokio::sync::oneshot::channel();



    tauri_plugin_dialog::FileDialogBuilder::new(app.dialog().clone())

        .add_filter("Archive files", &["zip", "7z", "rar"])

        .pick_file(move |file_path| {

            let _ = tx.send(file_path);

        });



    match rx.await.map_err(|e| e.to_string())? {

        Some(path) => Ok(path.as_path().map(|p| p.to_string_lossy().to_string())),

        None => Ok(None),

    }

}



#[tauri::command]

async fn pick_zip_files(app: tauri::AppHandle) -> Result<Vec<String>, String> {

    let (tx, rx) = tokio::sync::oneshot::channel();



    tauri_plugin_dialog::FileDialogBuilder::new(app.dialog().clone())

        .add_filter("Archive files", &["zip", "7z", "rar"])

        .pick_files(move |file_paths| {

            let _ = tx.send(file_paths);

        });



    match rx.await.map_err(|e| e.to_string())? {

        Some(paths) => Ok(paths.iter().map(|p| p.as_path().map(|pp| pp.to_string_lossy().to_string())).flatten().collect()),

        None => Ok(vec![]),

    }

}



#[tauri::command]

async fn import_chats_batch(zip_paths: Vec<String>) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || import_chats_batch_inner(zip_paths)).await.map_err(|e| e.to_string())?
}

fn import_chats_batch_inner(zip_paths: Vec<String>) -> Result<Vec<String>, String> {
    IMPORT_MSG_COUNT.store(0, Ordering::Relaxed);
    IMPORT_MEDIA_COUNT.store(0, Ordering::Relaxed);
    IMPORT_ACTIVE.store(1, Ordering::Relaxed);
    IMPORT_PHASE.store(1, Ordering::Relaxed);

    let mut chat_ids = Vec::new();
    let mut errors = Vec::new();

    for path in &zip_paths {
        IMPORT_PHASE.store(1, Ordering::Relaxed);

        match import_chat_inner(path.clone()) {
            Ok(id) => {
                chat_ids.push(id);
                IMPORT_PHASE.store(4, Ordering::Relaxed);
            }
            Err(e) => errors.push(format!("{}: {}", path, e)),
        }
    }

    IMPORT_ACTIVE.store(0, Ordering::Relaxed);
    IMPORT_PHASE.store(0, Ordering::Relaxed);

    if !errors.is_empty() {
        eprintln!("Batch import errors: {:?}", errors);
    }

    Ok(chat_ids)

}



#[tauri::command]

async fn import_chat(zip_path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || import_chat_inner(zip_path)).await.map_err(|e| e.to_string())?
}

fn import_chat_inner(zip_path: String) -> Result<String, String> {
    IMPORT_MSG_COUNT.store(0, Ordering::Relaxed);
    IMPORT_MEDIA_COUNT.store(0, Ordering::Relaxed);
    IMPORT_ACTIVE.store(1, Ordering::Relaxed);
    IMPORT_PHASE.store(1, Ordering::Relaxed);
    let chat_id = Uuid::new_v4().to_string();

    let app_data = get_app_data_dir();

    let import_dir = app_data.join("imports").join(&chat_id);

    let chat_dir = app_data.join("chats").join(&chat_id);



    ensure_dir_exists(&import_dir);

    ensure_dir_exists(&chat_dir);

    ensure_dir_exists(&chat_dir.join("media"));

    ensure_dir_exists(&chat_dir.join("custom"));



    let ext = get_archive_extension(&zip_path);
    let is_7z = ext == "7z";
    let is_rar = ext == "rar";
    let mut txt_content = String::new();

    let mut media_files: Vec<String> = Vec::new();



    if is_7z {

        // Extract 7z using external 7z command

        let output = std::process::Command::new("7z")

            .arg("x")

            .arg(&zip_path)

            .arg(format!("-o{}", import_dir.to_string_lossy()))

            .arg("-y")

            .output()

            .map_err(|e| format!("Failed to run 7z command: {}", e))?;



        if !output.status.success() {

            return Err(format!("7z extraction failed: {}", String::from_utf8_lossy(&output.stderr)));

        }



        // Find the TXT file
        for entry in fs::read_dir(&import_dir).map_err(|e| format!("Failed to read import dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".txt") {
                // Read bytes and decode with proper encoding detection
                let txt_bytes = fs::read(entry.path()).map_err(|e| format!("Failed to read TXT bytes: {}", e))?;
                let content = if txt_bytes.starts_with(b"\xef\xbb\xbf") {
                    encoding_rs::UTF_8.decode_without_bom_handling(&txt_bytes[3..]).0.to_string()
                } else if txt_bytes.starts_with(b"\xff\xfe") {
                    encoding_rs::UTF_16LE.decode_without_bom_handling(&txt_bytes[2..]).0.to_string()
                } else if txt_bytes.starts_with(b"\xfe\xff") {
                    encoding_rs::UTF_16BE.decode_without_bom_handling(&txt_bytes[2..]).0.to_string()
                } else {
                    match String::from_utf8(txt_bytes.clone()) {
                        Ok(s) => s,
                        Err(_) => encoding_rs::UTF_16LE.decode_without_bom_handling(&txt_bytes).0.to_string()
                    }
                };
                txt_content = content;
            } else {
                media_files.push(name.clone());
            }
        }

    } else if is_rar {

        // Extract RAR using external unrar command

        let output = std::process::Command::new("unrar")

            .arg("x")

            .arg(&zip_path)

            .arg(&import_dir.to_string_lossy().to_string())

            .arg("-y")

            .output()

            .map_err(|e| format!("Failed to run unrar command: {}", e))?;



        if !output.status.success() {

            return Err(format!("unrar extraction failed: {}", String::from_utf8_lossy(&output.stderr)));

        }



        // Find the TXT file
        for entry in fs::read_dir(&import_dir).map_err(|e| format!("Failed to read import dir: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".txt") {
                // Read bytes and decode with proper encoding detection
                let txt_bytes = fs::read(entry.path()).map_err(|e| format!("Failed to read TXT bytes: {}", e))?;
                let content = if txt_bytes.starts_with(b"\xef\xbb\xbf") {
                    encoding_rs::UTF_8.decode_without_bom_handling(&txt_bytes[3..]).0.to_string()
                } else if txt_bytes.starts_with(b"\xff\xfe") {
                    encoding_rs::UTF_16LE.decode_without_bom_handling(&txt_bytes[2..]).0.to_string()
                } else if txt_bytes.starts_with(b"\xfe\xff") {
                    encoding_rs::UTF_16BE.decode_without_bom_handling(&txt_bytes[2..]).0.to_string()
                } else {
                    match String::from_utf8(txt_bytes.clone()) {
                        Ok(s) => s,
                        Err(_) => encoding_rs::UTF_16LE.decode_without_bom_handling(&txt_bytes).0.to_string()
                    }
                };
                txt_content = content;
            } else {
                media_files.push(name.clone());
            }
        }

    } else {
        // Extract ZIP

        let file = File::open(&zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;

        const MAX_ZIP_ENTRIES: usize = 10_000;
        const MAX_MEDIA_BYTES: u64 = 500 * 1024 * 1024; // 500 MB per file
        if archive.len() > MAX_ZIP_ENTRIES {
            return Err("Archive contains too many entries".to_string());
        }

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| format!("ZIP extraction error: {}", e))?;
            // Use enclosed_name() to prevent zip-slip — returns None for unsafe paths (e.g. containing ..)
            let safe_path = match file.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => continue, // skip unsafe entries
            };
            let name = safe_path.to_string_lossy().to_string();
            let out_path = import_dir.join(&safe_path);

            if name.ends_with('/') || name.ends_with('\\') {
                // Directory entry — skip, ensure_dir_exists handles creation on demand
                continue;
            } else if name.ends_with(".txt") {
                // Read as bytes and decode with proper encoding detection
                let mut txt_bytes = Vec::new();
                file.read_to_end(&mut txt_bytes).map_err(|e| format!("Failed to read TXT bytes: {}", e))?;
                let content = if txt_bytes.starts_with(b"\xef\xbb\xbf") {
                    encoding_rs::UTF_8.decode_without_bom_handling(&txt_bytes[3..]).0.to_string()
                } else if txt_bytes.starts_with(b"\xff\xfe") {
                    encoding_rs::UTF_16LE.decode_without_bom_handling(&txt_bytes[2..]).0.to_string()
                } else if txt_bytes.starts_with(b"\xfe\xff") {
                    encoding_rs::UTF_16BE.decode_without_bom_handling(&txt_bytes[2..]).0.to_string()
                } else {
                    match String::from_utf8(txt_bytes.clone()) {
                        Ok(s) => s,
                        Err(_) => encoding_rs::UTF_16LE.decode_without_bom_handling(&txt_bytes).0.to_string()
                    }
                };
                txt_content = content;
            } else {
                media_files.push(name.clone());
                if let Some(parent) = out_path.parent() {
                    ensure_dir_exists(parent);
                }
                if let Ok(mut out_file) = File::create(&out_path) {
                    let _ = std::io::copy(&mut (&mut file).take(MAX_MEDIA_BYTES), &mut out_file);
                }
            }

        }

    }



    // Strip BOM if present and parse chat

    let txt_content = txt_content.trim_start_matches('\u{FEFF}');

    let messages = parse_chat_text(&txt_content, &chat_dir, &import_dir)?;



    // Move media to chat media folder (flatten subdirectory structure)

    for media in &media_files {

        let src = import_dir.join(media);

        // Use only the filename (not full path) to flatten subdirectories like "WhatsApp Images/"

        let filename = Path::new(media).file_name()

            .and_then(|n| n.to_str())

            .unwrap_or(media);

        let dst = chat_dir.join("media").join(filename);

        if let Some(parent) = dst.parent() {

            ensure_dir_exists(parent);

        }

        match fs::copy(&src, &dst) {

            Ok(_) => { IMPORT_MEDIA_COUNT.fetch_add(1, Ordering::Relaxed); IMPORT_PHASE.store(1, Ordering::Relaxed); },

            Err(_) => {},

        }

    }

    

    IMPORT_PHASE.store(2, Ordering::Relaxed);

    // Save to SQLite database

    let mut conn = get_db();

    // backfill_epochs not needed here — new imports always set last_message_epoch correctly

    

    let is_group = detect_group_chat(&messages);



    // Find the chronologically newest message by epoch (file order is not reliable)

    let (last_content, last_timestamp, new_max_epoch) = {

        let mut best_content = String::new();

        let mut best_ts = String::new();

        let mut best_epoch: i64 = 0;

        for msg in &messages {

            let ep = timestamp_to_epoch(&msg.timestamp);

            if ep >= best_epoch {

                best_epoch = ep;

                best_ts = msg.timestamp.clone();

                best_content = msg.content.clone();

            }

        }

        (best_content, best_ts, best_epoch)

    };

    

    // Get chat name from ZIP filename
    let zip_name = Path::new(&zip_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Chat")
        .to_string();

    // original_name preserves the raw ZIP filename for merge deduplication
    let original_name = zip_name.clone();

    // Strip WhatsApp export prefixes for a cleaner display name
    let prefixes = [
        "WhatsApp-chat met ",
        "WhatsApp-gesprek met ",
        "WhatsApp Chat with ",
        "WhatsApp-Unterhaltung mit ",
    ];
    let display_name = {
        let mut name = zip_name.as_str();
        for prefix in &prefixes {
            if let Some(stripped) = name.strip_prefix(prefix) {
                name = stripped;
                break;
            }
            // Case-insensitive fallback
            if name.to_lowercase().starts_with(&prefix.to_lowercase()) {
                name = &name[prefix.len()..];
                break;
            }
        }
        name.to_string()
    };

    let chat_name = if is_group {
        format!("{} (Group)", display_name)
    } else {
        display_name.clone()
    };



    // Check if a chat with the same name already exists → merge instead of duplicate

    let existing_check = find_existing_chat_by_name(&conn, &zip_name, is_group);

    if let Some(existing_id) = existing_check {



        // Copy media into the existing chat's media folder (flatten subdirectory structure)

        let existing_chat_dir = get_app_data_dir().join("chats").join(&existing_id);

        for media in &media_files {

            let src = import_dir.join(media);

            // Use only the filename (not full path) to flatten subdirectories like "WhatsApp Images/"

            let filename = Path::new(media).file_name()

                .and_then(|n| n.to_str())

                .unwrap_or(media);

            let dst = existing_chat_dir.join("media").join(filename);

            if let Some(parent) = dst.parent() {

                ensure_dir_exists(parent);

            }

            let _ = fs::copy(&src, &dst);

        }



        merge_messages_into_chat(&mut conn, &existing_id, &messages)?;



        // After merge, compute true max epoch across ALL messages in the chat (existing + new)

        // db timestamps are strings, so iterate and convert in Rust

        let merged_max_epoch = {

            let mut stmt = conn.prepare(

                "SELECT timestamp FROM messages WHERE chat_id = ?1"

            ).map_err(|e| e.to_string())?;

            let rows = stmt.query_map(params![&existing_id], |row| row.get::<_, String>(0))

                .map_err(|e| e.to_string())?;

            let mut max_ep: i64 = 0;

            for row in rows.flatten() {

                let ep = timestamp_to_epoch(&row);

                if ep > max_ep { max_ep = ep; }

            }

            max_ep

        };

        // Use whichever is larger: existing DB max or incoming messages max

        let final_epoch = merged_max_epoch.max(new_max_epoch);

        conn.execute(

            "UPDATE chats SET last_message = ?1, timestamp = ?2, last_message_epoch = ?3 WHERE id = ?4",

            params![&last_content, &last_timestamp, final_epoch, &existing_id],

        ).map_err(|e| format!("Failed to update chat metadata: {}", e))?;



        // Clean up temp import dir

        let _ = fs::remove_dir_all(&import_dir);



        return Ok(existing_id);

    }

    

    // Insert chat metadata (fresh import) — use the already-computed new_max_epoch

    conn.execute(

        "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch, zip_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",

        params![

            &chat_id,

            &chat_name,

            &original_name,

            &last_content,

            &last_timestamp,

            is_group as i32,

            new_max_epoch,

            Some(zip_path.as_str())

        ],

    ).map_err(|e| format!("Failed to insert chat: {}", e))?;

    

    // Insert all messages

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    {

        let mut stmt = tx.prepare(

            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media, duration, tag_ext, display_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"

        ).map_err(|e| e.to_string())?;



        IMPORT_PHASE.store(3, Ordering::Relaxed);

        for msg in &messages {

            IMPORT_MSG_COUNT.fetch_add(1, Ordering::Relaxed);

            stmt.execute(params![

                &chat_id,

                &msg.timestamp,

                &msg.sender,

                &msg.msg_type,

                &msg.content,

                msg.media.as_ref().unwrap_or(&String::new()),

                msg.duration.as_ref().unwrap_or(&String::new()),

                &msg.tag_ext,

                &msg.display_name,

            ]).map_err(|e| e.to_string())?;

        }

    }

    tx.commit().map_err(|e| format!("Failed to commit transaction: {}", e))?;

    

    // Also save JSON backup (for now, until fully migrated)

    let chat_data = ChatData { messages };

    let messages_json = serde_json::to_string_pretty(&chat_data).map_err(|e| e.to_string())?;

    let messages_path = chat_dir.join("messages.json");

    let _ = fs::write(&messages_path, messages_json);

    

    let meta = ChatMeta {

        id: chat_id.clone(),

        name: chat_name.clone(),

        last_message: last_content.clone(),

        timestamp: last_timestamp.clone(),

        is_group,

        zip_path: Some(zip_path.clone()),

        photo_path: None,

    };

    let meta_json = serde_json::to_string_pretty(&meta).map_err(|e| e.to_string())?;

    let _ = fs::write(chat_dir.join("meta.json"), meta_json);

    

    IMPORT_PHASE.store(4, Ordering::Relaxed);
    IMPORT_ACTIVE.store(0, Ordering::Relaxed);

    Ok(chat_id)

}



fn timestamp_to_epoch(ts: &str) -> i64 {

    // Parses "dd/mm/yyyy HH:MM", "dd-mm-yyyy HH:MM", "dd.mm.yyyy HH:MM" and 2-digit year variants

    // Returns seconds since Unix epoch (UTC), or 0 on failure.

    let slash_full  = TS_RE_SLASH_FULL.get_or_init(|| regex::Regex::new(r"(\d{1,2})/(\d{1,2})/(\d{4})\s+(\d{1,2}):(\d{2})").unwrap());
    let slash_short = TS_RE_SLASH_SHORT.get_or_init(|| regex::Regex::new(r"(\d{1,2})/(\d{1,2})/(\d{2})\s+(\d{1,2}):(\d{2})").unwrap());
    let dash_full   = TS_RE_DASH_FULL.get_or_init(|| regex::Regex::new(r"(\d{2})-(\d{2})-(\d{4})\s+(\d{1,2}):(\d{2})").unwrap());
    let dot_full    = TS_RE_DOT_FULL.get_or_init(|| regex::Regex::new(r"(\d{2})\.(\d{2})\.(\d{4})\s+(\d{1,2}):(\d{2})").unwrap());



    let (d, m, y, h, min) = if let Some(c) = slash_full.captures(ts) {

        (c[1].parse::<i64>().unwrap_or(1), c[2].parse::<i64>().unwrap_or(1),

         c[3].parse::<i64>().unwrap_or(2000), c[4].parse::<i64>().unwrap_or(0), c[5].parse::<i64>().unwrap_or(0))

    } else if let Some(c) = slash_short.captures(ts) {

        (c[1].parse::<i64>().unwrap_or(1), c[2].parse::<i64>().unwrap_or(1),

         2000 + c[3].parse::<i64>().unwrap_or(0), c[4].parse::<i64>().unwrap_or(0), c[5].parse::<i64>().unwrap_or(0))

    } else if let Some(c) = dash_full.captures(ts) {

        (c[1].parse::<i64>().unwrap_or(1), c[2].parse::<i64>().unwrap_or(1),

         c[3].parse::<i64>().unwrap_or(2000), c[4].parse::<i64>().unwrap_or(0), c[5].parse::<i64>().unwrap_or(0))

    } else if let Some(c) = dot_full.captures(ts) {

        (c[1].parse::<i64>().unwrap_or(1), c[2].parse::<i64>().unwrap_or(1),

         c[3].parse::<i64>().unwrap_or(2000), c[4].parse::<i64>().unwrap_or(0), c[5].parse::<i64>().unwrap_or(0))

    } else {

        return 0;

    };



    // Simple days-since-epoch calculation (no external crate needed)

    // Using the algorithm: count days from 1970-01-01

    let months = [31i64,28,31,30,31,30,31,31,30,31,30,31];

    let is_leap = |yr: i64| (yr % 4 == 0 && yr % 100 != 0) || yr % 400 == 0;



    let mut days: i64 = 0;

    for yr in 1970..y {

        days += if is_leap(yr) { 366 } else { 365 };

    }

    for mo in 1..m {

        days += months[(mo - 1) as usize];

        if mo == 2 && is_leap(y) { days += 1; }

    }

    days += d - 1;



    days * 86400 + h * 3600 + min * 60

}



fn normalize_chat_name(name: &str) -> String {

    name.trim()

        .trim_end_matches(" (Group)")

        .to_lowercase()

}



fn find_existing_chat_by_name(conn: &Connection, zip_name: &str, is_group: bool) -> Option<String> {

    let candidate = if is_group {

        format!("{} (Group)", zip_name)

    } else {

        zip_name.to_string()

    };

    let normalized = normalize_chat_name(&candidate);




    // Match against original_name (immutable ZIP-derived name) so profile renames never break merging

    let mut stmt = conn.prepare(

        "SELECT id, original_name FROM chats"

    ).ok()?;



    let rows = stmt.query_map([], |row| {

        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))

    }).ok()?;



    for row in rows.flatten() {

        let row_normalized = normalize_chat_name(&row.1);

        if row_normalized == normalized {

            return Some(row.0);

        }

    }

    None

}



fn merge_messages_into_chat(conn: &mut Connection, chat_id: &str, new_messages: &[Message]) -> Result<usize, String> {

    // Build a fingerprint set of existing messages: (timestamp, sender, content_prefix) -> db_id

    // Include first 40 chars of content to distinguish same-sender same-timestamp messages

    let mut existing: std::collections::HashMap<(String, String, String), i64> = std::collections::HashMap::new();

    {

        let mut stmt = conn.prepare(

            "SELECT id, timestamp, sender, content FROM messages WHERE chat_id = ?1 ORDER BY id ASC"

        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([chat_id], |row| {

            Ok((

                row.get::<_, i64>(0)?,

                row.get::<_, String>(1)?,

                row.get::<_, String>(2)?,

                row.get::<_, String>(3)?,

            ))

        }).map_err(|e| e.to_string())?;

        for row in rows {

            let (db_id, ts, sender, content) = row.map_err(|e| e.to_string())?;

            let clean_content: String = content.chars().filter(|c| !matches!(*c,
                '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
                '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
            )).collect();

            let prefix: String = clean_content.chars().take(40).collect();

            existing.insert((ts, sender, prefix), db_id);

        }

    }



    let mut inserted = 0usize;



    let tx = conn.transaction().map_err(|e| e.to_string())?;

    for msg in new_messages {

        let clean_content: String = msg.content.chars().filter(|c| !matches!(*c,
            '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
            '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
        )).collect();

        let content_prefix: String = clean_content.chars().take(40).collect();

        let key = (msg.timestamp.clone(), msg.sender.clone(), content_prefix);

        if let Some(&db_id) = existing.get(&key) {

            // Message already exists — preserve manual changes, but upgrade msg_type
            // if the new parse produced a better type (e.g. "file" -> "image")

            let better_type = match msg.msg_type.as_str() {
                "image" | "video" | "audio" | "gif" | "sticker" => true,
                _ => false,
            };

            if better_type {
                let _ = tx.execute(
                    "UPDATE messages SET msg_type = ?1 WHERE id = ?2 AND msg_type = 'file'",
                    params![&msg.msg_type, db_id],
                );
            }

            continue;

        }



        // New message — append with tag_ext and display_name support

        tx.execute(

            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media, duration, tag_ext, display_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",

            params![

                chat_id,

                &msg.timestamp,

                &msg.sender,

                &msg.msg_type,

                &msg.content,

                msg.media.as_ref().unwrap_or(&String::new()),

                msg.duration.as_ref().unwrap_or(&String::new()),

                &msg.tag_ext,

                &msg.display_name,

            ],

        ).map_err(|e| e.to_string())?;

        inserted += 1;

    }

    tx.commit().map_err(|e| e.to_string())?;



    Ok(inserted)

}



fn detect_group_chat(messages: &[Message]) -> bool {

    let senders: std::collections::HashSet<_> = messages.iter()

        .filter(|m| m.sender != "System" && m.sender != "You")

        .map(|m| m.sender.clone())

        .collect();

    senders.len() > 1

}



fn parse_chat_text(content: &str, _chat_dir: &Path, import_dir: &Path) -> Result<Vec<Message>, String> {

    let mut messages = Vec::new();

    let lines: Vec<&str> = content.lines().collect();

    

    // Compile all patterns ONCE before iterating lines

    // WhatsApp format patterns - handles various export formats

    // iOS with brackets: [12/04/2024, 14:32] John: Hello

    // Android slash: 12/04/2024, 14:32 - John: Hello

    // Android dash: 17-06-2017 00:04 - John: Hello

    let date_patterns = [

        // iOS: [12/04/2024, 14:32] John: Hello (full year)

        regex::Regex::new(r"^\[(\d{2}/\d{2}/\d{4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+?):\s+(.+)$").unwrap(),

        // iOS: [12/04/24, 14:32] John: Hello (2-digit year)

        regex::Regex::new(r"^\[(\d{2}/\d{2}/\d{2}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+?):\s+(.+)$").unwrap(),

        // iOS US: [4/12/2024, 2:32 PM] John: Hello (full year with AM/PM)

        regex::Regex::new(r"^\[(\d{1,2}/\d{1,2}/\d{4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?\s*(?:AM|PM)?)\]\s+(.+?):\s+(.+)$").unwrap(),

        // iOS US: [4/12/24, 2:32 PM] John: Hello (2-digit year with AM/PM)

        regex::Regex::new(r"^\[(\d{1,2}/\d{1,2}/\d{2}),?\s+(\d{1,2}:\d{2}(?::\d{2})?\s*(?:AM|PM)?)\]\s+(.+?):\s+(.+)$").unwrap(),

        // Android slash: 12/04/2024, 14:32 - John: Hello

        regex::Regex::new(r"^(\d{2}/\d{2}/\d{4}),\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Android slash 2-digit: 12/04/24, 14:32 - John: Hello

        regex::Regex::new(r"^(\d{2}/\d{2}/\d{2}),\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Android slash US: 4/12/2024, 2:32 PM - John: Hello

        regex::Regex::new(r"^(\d{1,2}/\d{1,2}/\d{4}),\s+(\d{1,2}:\d{2}(?::\d{2})?\s*(?:AM|PM)?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Android slash US 2-digit: 4/12/24, 2:32 PM - John: Hello

        regex::Regex::new(r"^(\d{1,2}/\d{1,2}/\d{2}),\s+(\d{1,2}:\d{2}(?::\d{2})?\s*(?:AM|PM)?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Android dash: 17-06-2017 00:04 - John: Hello (NO COMMA)

        regex::Regex::new(r"^(\d{2}-\d{2}-\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Android dash 2-digit: 17-06-17 00:04 - John: Hello

        regex::Regex::new(r"^(\d{2}-\d{2}-\d{2})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Android dotted: 17.06.2017 00:04 - John: Hello

        regex::Regex::new(r"^(\d{2}\.\d{2}\.\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+?):\s+(.+)$").unwrap(),

        // Dotted with brackets: [12.04.2024, 14:32] John: Hello

        regex::Regex::new(r"^\[(\d{2}\.\d{2}\.\d{2,4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+?):\s+(.+)$").unwrap(),

        // Dashed with brackets: [12-04-2024, 14:32] John: Hello

        regex::Regex::new(r"^\[(\d{2}-\d{2}-\d{2,4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+?):\s+(.+)$").unwrap(),

    ];

    

    // Compile system patterns ONCE here, outside the per-line loop

    let system_patterns = [

        // iOS format with brackets

        regex::Regex::new(r"^\[(\d{2}/\d{2}/\d{4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+)$").unwrap(),

        regex::Regex::new(r"^\[(\d{2}/\d{2}/\d{2}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+)$").unwrap(),

        regex::Regex::new(r"^\[(\d{1,2}/\d{1,2}/\d{2,4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+)$").unwrap(),

        regex::Regex::new(r"^\[(\d{2}\.\d{2}\.\d{2,4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+)$").unwrap(),

        regex::Regex::new(r"^\[(\d{2}-\d{2}-\d{2,4}),?\s+(\d{1,2}:\d{2}(?::\d{2})?)\]\s+(.+)$").unwrap(),

        // Android with comma: "12/12/2025, 10:15 - Messages..."

        regex::Regex::new(r"^(\d{2}/\d{2}/\d{4}),\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+)$").unwrap(),

        regex::Regex::new(r"^(\d{2}/\d{2}/\d{2}),\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+)$").unwrap(),

        regex::Regex::new(r"^(\d{1,2}/\d{1,2}/\d{2,4}),\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+)$").unwrap(),

        // Android dash NO COMMA: "17-06-2017 00:04 - Messages..."

        regex::Regex::new(r"^(\d{2}-\d{2}-\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+)$").unwrap(),

        regex::Regex::new(r"^(\d{2}-\d{2}-\d{2})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+)$").unwrap(),

        // Android dotted NO COMMA: "17.06.2017 00:04 - Messages..."

        regex::Regex::new(r"^(\d{2}\.\d{2}\.\d{4})\s+(\d{1,2}:\d{2}(?::\d{2})?)\s+-\s+(.+)$").unwrap(),

    ];



    let mut current_msg: Option<Message> = None;

    let mut in_code_block = false;
    // Two-flag post-fence-close guard:
    // after_fence_close — a fence was just closed (reset by non-blank line or fence open)
    // blank_since_close — a blank line appeared since that close
    // We only reject a date-format candidate when BOTH are true, meaning the blank line
    // is what separated the fence close from the candidate (embedded example pattern).
    // A candidate immediately after the close (no blank line) is accepted as a real message.
    let mut after_fence_close = false;
    let mut blank_since_close = false;

    

    for line in lines {

        let line = line.trim_end_matches('\r'); // Remove Windows CRLF

        let mut matched = false;

        // Track triple-backtick code blocks — lines inside them are always continuation
        // Strip common invisible Unicode prefixes WhatsApp export adds (LTR/RTL marks etc.)
        let trimmed = line.trim_start_matches(|c: char| {
            c.is_whitespace() || c == '\u{200e}' || c == '\u{200f}' || c == '\u{feff}'
        });
        // Only treat lines that are EXACTLY ``` (or ```lang) as fences — not ```` or more
        let is_fence = trimmed.starts_with("```")
            && !trimmed.starts_with("````");
        if is_fence {
            let was_open = in_code_block;
            in_code_block = !in_code_block;
            after_fence_close = was_open; // true if we just closed a block
            blank_since_close = false;  // reset blank tracking on every fence transition
            if let Some(ref mut msg) = current_msg {
                msg.content.push('\n');
                msg.content.push_str(line);
            }
            matched = true;
        } else if !is_fence {
            // Reset fence-close guard once we've seen a non-fence line
            // (but don't reset yet — do it after the pattern check below)
        }

        if matched {
            continue;
        }

        if in_code_block {
            // Check if this line looks like a new message timestamp — if so, implicitly close
            // the unclosed code block and fall through to normal date matching below.
            let looks_like_date = line.len() > 10 && (
                line.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
            ) && line.contains(" - ");
            if !looks_like_date {
                if let Some(ref mut msg) = current_msg {
                    msg.content.push('\n');
                    msg.content.push_str(line);
                }
                continue;
            }
            // Implicitly close the unclosed fence
            in_code_block = false;
            after_fence_close = false;
            blank_since_close = false;
        }

        for pattern in date_patterns.iter() {

            if let Some(caps) = pattern.captures(line) {

                let sender_candidate = caps.get(3).map(|m| m.as_str()).unwrap_or("");

                // Reject matches where the "sender" looks like code/markdown content:
                // real WhatsApp sender names never contain backticks, hash, asterisk,
                // brackets, or exceed a reasonable length.
                let sender_looks_valid = sender_candidate.len() <= 80
                    && !sender_candidate.contains('`')
                    && !sender_candidate.contains('#')
                    && !sender_candidate.contains('[')
                    && !sender_candidate.contains(']')
                    && !sender_candidate.contains("**")
                    && !sender_candidate.contains("~~");

                let date_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let time_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                if !sender_looks_valid || !is_plausible_whatsapp_date(date_str, time_str) || (after_fence_close && blank_since_close) {
                    // Treat as continuation of previous message
                    if let Some(ref mut msg) = current_msg {
                        msg.content.push('\n');
                        msg.content.push_str(line);
                    }
                    matched = true;
                    break;
                }

                // Save previous message if exists — reset code block state per message
                in_code_block = false;
                after_fence_close = false;
                blank_since_close = false;

                if let Some(msg) = current_msg.take() {

                    messages.push(msg);

                }

                

                let sender = sender_candidate.to_string();

                let content_text = caps.get(4).map(|m| m.as_str()).unwrap_or("").to_string();

                

                let timestamp = format!("{} {}", date_str, time_str);

                

                // Check for media (English and Dutch)

                let (msg_type, media_path, final_content) = if content_text.contains("<Media omitted>") || 

                    content_text.contains("<Media weggelaten>") ||

                    content_text.contains("(file attached)") ||

                    content_text.contains("(bestand bijgevoegd)") {

                    let media_file = content_text

                        .replace("<Media omitted>", "")

                        .replace("<Media weggelaten>", "")

                        .replace("(file attached)", "")

                        .replace("(bestand bijgevoegd)", "")

                        .replace('\n', " ")

                        .replace('\r', "")

                        .split_whitespace()

                        .collect::<Vec<_>>()

                        .join(" ")

                        .trim()

                        .to_string();

                    // Strip Unicode directional/zero-width marks WhatsApp embeds in filenames

                    let media_file: String = media_file.chars().filter(|c| !matches!(*c,

                        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |

                        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'

                    )).collect();

                    let media_ext = Path::new(&media_file).extension()

                        .and_then(|e| e.to_str())

                        .unwrap_or("");

                    let media_basename = Path::new(&media_file)

                        .file_name().and_then(|n| n.to_str()).unwrap_or(&media_file);

                    let is_sticker = media_basename.to_uppercase().starts_with("STK-");

                    let msg_type = if is_sticker {

                        "sticker"

                    } else {

                        match media_ext.to_lowercase().as_str() {

                            "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "heic" | "heif" | "svg" | "tif" | "tiff" | "avif" => "image",

                            "mp4" | "mov" | "avi" | "mkv" | "webm" | "m4v" | "ts" => {

                                // Detect GIF: WhatsApp GIFs are typically very small (≤ 800 KB)

                                let fname = Path::new(&media_file).file_name().unwrap_or_default();

                                let media_full_path = import_dir.join(fname);

                                let size = fs::metadata(&media_full_path).map(|m| m.len()).unwrap_or(u64::MAX);

                                if size <= 800_000 { "gif" } else { "video" }

                            },

                            "mp3" | "ogg" | "opus" | "wav" | "m4a" | "aac" | "3gp" | "3gpp" | "amr" | "flac" => "audio",

                            _ => "file",

                        }

                    };

                    (msg_type.to_string(), Some(media_file), content_text.clone())

                } else if let Some(url) = extract_maps_url(&content_text) {

                    ("location".to_string(), Some(url), content_text)

                } else if content_text.trim().starts_with("PEILING:") || content_text.trim().starts_with("POLL:") {
                    // Handle poll messages
                    ("poll".to_string(), None, content_text)

                } else if content_text.trim().starts_with("EVENEMENT:") || content_text.trim().starts_with("EVENT:") {
                    // Handle calendar event messages (Dutch: EVENEMENT, English: EVENT)
                    ("event".to_string(), None, content_text)

                } else {
                    // Detect bare sticker filenames (no attachment marker, just the filename)

                    let trimmed = content_text.trim();

                    let clean_trimmed: String = trimmed.chars().filter(|c| !matches!(*c,
                        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
                        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
                    )).collect();

                    let basename = Path::new(clean_trimmed.as_str())
                        .file_name().and_then(|n| n.to_str()).unwrap_or(clean_trimmed.as_str());

                    if basename.to_uppercase().starts_with("STK-") && basename.to_uppercase().ends_with(".WEBP") {

                        ("sticker".to_string(), Some(clean_trimmed.clone()), content_text)

                    } else {

                        ("text".to_string(), None, content_text)

                    }

                };

                

                // Extract duration for audio files

                let duration = if msg_type == "audio" && media_path.is_some() {

                    let media_file_path = _chat_dir.join("media").join(media_path.as_ref().unwrap());

                    get_audio_duration(&media_file_path)

                } else {

                    None

                };

                

                current_msg = Some(Message {
                    id: None,
                    timestamp,

                    sender,

                    msg_type,

                    content: final_content,

                    media: media_path,

                    duration,

                    tag_ext: None,

                    display_name: None,

                    is_favorite: None,

                });

                

                matched = true;

                break;

            }

        }

        

        // Check for system messages (only if no regular message matched)

        if !matched {

            for sys_pattern in &system_patterns {

                if let Some(caps) = sys_pattern.captures(line) {

                    let date_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");

                    let time_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                    let content_text = caps.get(3).map(|m| m.as_str()).unwrap_or("").to_string();

                    

                    // Check if it looks like a system message (no colon before content, or starts with system text)

                    let is_system = !content_text.contains(": ") 

                        || content_text.starts_with("Messages and calls are end-to-end")

                        || content_text.starts_with("Berichten en oproepen")

                        || content_text.starts_with("Messages to this chat");

                    if is_system {

                        if let Some(msg) = current_msg.take() {

                            messages.push(msg);

                        }

                        current_msg = Some(Message {
                            id: None,
                            timestamp: format!("{} {}", date_str, time_str),

                            sender: "System".to_string(),

                            msg_type: "system".to_string(),

                            content: content_text,

                            media: None,

                            duration: None,

                            tag_ext: None,

                            display_name: None,

                            is_favorite: None,

                        });

                        matched = true;

                        break;

                    }

                    // pattern matched structurally but content looks like a regular message

                    // → keep trying other system patterns (don't break here)

                }

            }

        }

        

        // Continuation of previous message (multiline)

        if !matched && !line.is_empty() {

            if let Some(ref mut msg) = current_msg {

                msg.content.push('\n');

                msg.content.push_str(line);

            }

        }

        // Update the post-fence-close guard.
        // Non-blank, non-fence line: exit the post-close window entirely.
        // Blank line while inside the window: mark that a gap exists — this is what
        // distinguishes an embedded example (fence ``` → blank → fake-WA-line) from a
        // real next message (fence ``` → real-WA-line with no gap).
        if !is_fence {
            if line.trim().is_empty() {
                if after_fence_close {
                    blank_since_close = true;
                }
            } else {
                after_fence_close = false;
                blank_since_close = false;
            }
        }

    }

    

    // Don't forget the last message

    if let Some(msg) = current_msg {

        messages.push(msg);

    }

    

    Ok(messages)

}



#[tauri::command]

fn get_chat_list() -> Result<Vec<ChatMeta>, String> {

    let conn = get_db();

    backfill_epochs(&conn);

    

    let mut stmt = conn.prepare(

        "SELECT chats.id, COALESCE(profiles.name, chats.name), chats.last_message, chats.timestamp, chats.is_group, chats.zip_path, profiles.photo_path
         FROM chats LEFT JOIN profiles ON chats.id = profiles.chat_id
         ORDER BY chats.last_message_epoch DESC, chats.created_at DESC"

    ).map_err(|e| e.to_string())?;

    

    let chats = stmt.query_map([], |row| {

        Ok(ChatMeta {

            id: row.get(0)?,

            name: row.get(1)?,

            last_message: row.get(2)?,

            timestamp: row.get(3)?,

            is_group: row.get::<_, i32>(4)? != 0,

            zip_path: row.get(5)?,

            photo_path: row.get(6)?,

        })

    }).map_err(|e| e.to_string())?;

    

    let mut result = Vec::new();

    for chat in chats {

        result.push(chat.map_err(|e| e.to_string())?);

    }

    

    Ok(result)

}



#[tauri::command]

fn get_chat_messages(chat_id: String, limit: Option<i64>, offset: Option<i64>) -> Result<ChatData, String> {

    let conn = get_db();

    

    let query = match (limit, offset) {

        (Some(lim), Some(off)) => format!(

            "SELECT timestamp, sender, msg_type, content, media, duration, tag_ext, display_name, is_favorite FROM messages WHERE chat_id = ?1 ORDER BY id ASC LIMIT {} OFFSET {}",

            lim, off

        ),

        (Some(lim), None) => format!(

            "SELECT timestamp, sender, msg_type, content, media, duration, tag_ext, display_name, is_favorite FROM messages WHERE chat_id = ?1 ORDER BY id ASC LIMIT {}",

            lim

        ),

        _ => "SELECT timestamp, sender, msg_type, content, media, duration, tag_ext, display_name, is_favorite FROM messages WHERE chat_id = ?1 ORDER BY id ASC".to_string(),

    };

    

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    

    let messages = stmt.query_map([&chat_id], |row| {

        let media: String = row.get(4)?;

        let duration: String = row.get(5)?;

        let tag_ext: Option<String> = row.get(6)?;

        let display_name: Option<String> = row.get(7)?;

        let is_favorite: Option<i64> = row.get(8)?;

        Ok(Message {
            id: None,
            timestamp: row.get(0)?,

            sender: row.get(1)?,

            msg_type: row.get(2)?,

            content: row.get(3)?,

            media: if media.is_empty() { None } else { Some(media) },

            duration: if duration.is_empty() { None } else { Some(duration) },

            tag_ext,

            display_name,

            is_favorite: Some(is_favorite.unwrap_or(0) != 0),

        })

    }).map_err(|e| e.to_string())?;

    

    let mut result = Vec::new();

    for msg in messages {

        result.push(msg.map_err(|e| e.to_string())?);

    }

    

    Ok(ChatData { messages: result })

}



#[tauri::command]

fn get_chat_message_count(chat_id: String) -> Result<i64, String> {

    let conn = get_db();

    let count: i64 = conn.query_row(

        "SELECT COUNT(*) FROM messages WHERE chat_id = ?1",

        [&chat_id],

        |row| row.get(0)

    ).map_err(|e| e.to_string())?;

    Ok(count)

}



#[tauri::command]

fn get_media_base_dir(chat_id: String) -> String {

    let app_data = get_app_data_dir();

    app_data.join("chats").join(&chat_id).join("media")

        .to_string_lossy()

        .to_string()

}



fn mime_for_ext(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg"  => "image/jpeg",
        "png"           => "image/png",
        "gif"           => "image/gif",
        "webp"          => "image/webp",
        "bmp"           => "image/bmp",
        "heic" | "heif" => "image/heic",
        "svg"           => "image/svg+xml",
        "tif" | "tiff"  => "image/png",
        "avif"          => "image/avif",
        "mp4"           => "video/mp4",
        "mov"           => "video/quicktime",
        "avi"           => "video/x-msvideo",
        "mkv"           => "video/x-matroska",
        "webm"          => "video/webm",
        "3gp" | "3gpp"  => "audio/3gpp",
        "m4v"           => "video/x-m4v",
        "ts"            => "video/mp2t",
        "opus"          => "audio/ogg; codecs=opus",
        "ogg"           => "audio/ogg",
        "mp3"           => "audio/mpeg",
        "m4a"           => "audio/mp4",
        "aac"           => "audio/aac",
        "wav"           => "audio/wav",
        "amr"           => "audio/amr",
        "flac"          => "audio/flac",
        _               => "application/octet-stream",
    }
}

#[tauri::command]

fn get_media_as_base64(chat_id: String, filename: String, mime_hint: Option<String>) -> Result<String, String> {

    // Check preload cache first
    let cache_key = format!("{}:{}", chat_id, filename);
    {
        let mut cache = get_media_preload_cache().lock().map_err(|e| e.to_string())?;
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached);
        }
    }

    validate_chat_id(&chat_id)?;
    let app_data = get_app_data_dir();

    // Strip Unicode directional/zero-width marks WhatsApp embeds in filenames

    let filename_clean: String = filename.chars().filter(|c| !matches!(*c,

        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |

        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'

    )).collect();
    let filename_clean = sanitize_filename(&filename_clean)?;

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename_clean);



    let bytes = if media_path.exists() {
        fs::read(&media_path).map_err(|e| e.to_string())?
    } else {
        try_extract_from_zip(&chat_id, &filename_clean, &media_path)?
    };

    // Convert TIFF to PNG for browser compatibility (browsers can't render image/tiff)
    let bytes = {
        let ext_check = media_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        if ext_check == "tiff" || ext_check == "tif" {
            convert_tiff_to_png(&bytes).unwrap_or(bytes)
        } else {
            bytes
        }
    };

    let b64 = base64_encode(&bytes);



    // If caller supplies a mime_hint (e.g. for extensionless files), use it directly

    if let Some(hint) = mime_hint {

        if !hint.is_empty() {

            return Ok(format!("data:{};base64,{}", hint, b64));

        }

    }



    let ext = media_path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = mime_for_ext(ext.as_str());



    let result = format!("data:{};base64,{}", mime, b64);

    // Store in preload cache
    {
        let mut cache = get_media_preload_cache().lock().map_err(|e| e.to_string())?;
        cache.set(cache_key, result.clone());
    }

    Ok(result)

}

/// If a media file is missing from the chat's media folder, try to extract it
/// from the original source ZIP (stored in the DB) and save it permanently.
/// Returns the bytes if found, Err if not available from any source.
fn try_extract_from_zip(chat_id: &str, filename_clean: &str, media_path: &std::path::Path) -> Result<Vec<u8>, String> {
    let conn = get_db();
    let zip_path: Option<String> = conn.query_row(
        "SELECT zip_path FROM chats WHERE id = ?1",
        [chat_id],
        |row| row.get(0),
    ).ok().flatten();

    let zip_path = zip_path.ok_or_else(|| format!("File not found and no source ZIP: {:?}", media_path))?;
    if !std::path::Path::new(&zip_path).exists() {
        return Err(format!("File not found and source ZIP missing: {}", zip_path));
    }

    let ext = get_archive_extension(&zip_path);
    let bytes = if ext == "7z" {
        let tmp_dir = media_path.parent().unwrap();
        let out = std::process::Command::new("7z")
            .args(["x", &zip_path, &format!("-o{}", tmp_dir.to_string_lossy()), filename_clean, "-y"])
            .output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err("7z extract failed".into()); }
        fs::read(media_path).map_err(|e| e.to_string())?
    } else if ext == "rar" {
        let tmp_dir = media_path.parent().unwrap().to_string_lossy().to_string();
        let out = std::process::Command::new("unrar")
            .args(["x", &zip_path, filename_clean, &tmp_dir, "-y"])
            .output().map_err(|e| e.to_string())?;
        if !out.status.success() { return Err("unrar extract failed".into()); }
        fs::read(media_path).map_err(|e| e.to_string())?
    } else {
        let file = File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
        let mut found_bytes: Option<Vec<u8>> = None;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let entry_filename = std::path::Path::new(&name)
                .file_name().and_then(|n| n.to_str()).unwrap_or("");
            if entry_filename == filename_clean || name.ends_with(filename_clean) {
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut entry, &mut buf).map_err(|e| e.to_string())?;
                // Save permanently so future loads don't need the ZIP
                if let Ok(mut out) = File::create(media_path) {
                    let _ = std::io::Write::write_all(&mut out, &buf);
                }
                found_bytes = Some(buf);
                break;
            }
        }
        found_bytes.ok_or_else(|| format!("'{}' not found in ZIP", filename_clean))?
    };
    Ok(bytes)
}

#[derive(serde::Serialize)]
struct MediaWithDims {
    data: String,
    width: u32,
    height: u32,
}

#[tauri::command]
fn get_media_with_dims(chat_id: String, filename: String, mime_hint: Option<String>) -> Result<MediaWithDims, String> {
    // Check preload cache first
    let cache_key = format!("{}:{}", chat_id, filename);
    {
        let mut cache = get_media_preload_cache().lock().map_err(|e| e.to_string())?;
        if let Some(cached) = cache.get(&cache_key) {
            if let Some(data_start) = cached.find("base64,") {
                let b64_data = &cached[data_start + 7..];
                if let Ok(bytes) = base64_decode(b64_data) {
                    let (width, height) = get_image_dimensions(&bytes).unwrap_or((0, 0));
                    return Ok(MediaWithDims { data: cached.clone(), width, height });
                }
            }
            return Ok(MediaWithDims { data: cached.clone(), width: 0, height: 0 });
        }
    }

    validate_chat_id(&chat_id)?;
    let app_data = get_app_data_dir();

    // Strip Unicode directional/zero-width marks WhatsApp embeds in filenames
    let filename_clean: String = filename.chars().filter(|c| !matches!(*c,
        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
    )).collect();
    let filename_clean = sanitize_filename(&filename_clean)?;

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename_clean);

    let bytes = if media_path.exists() {
        fs::read(&media_path).map_err(|e| e.to_string())?
    } else {
        try_extract_from_zip(&chat_id, &filename_clean, &media_path)?
    };

    // Get dimensions - try video first for video formats, then fall back to image
    let ext = media_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let is_video = matches!(ext.as_str(), "mp4" | "mov" | "avi" | "mkv" | "webm" | "m4v" | "ts");

    let (width, height) = if is_video {
        get_video_dimensions(&bytes).unwrap_or((0, 0))
    } else {
        get_image_dimensions(&bytes).unwrap_or((0, 0))
    };

    // Convert TIFF to PNG for browser compatibility (browsers can't render image/tiff)
    let bytes = if ext == "tiff" || ext == "tif" {
        convert_tiff_to_png(&bytes).unwrap_or(bytes)
    } else {
        bytes
    };

    let b64 = base64_encode(&bytes);

    // If caller supplies a mime_hint (e.g. for extensionless files), use it directly
    if let Some(hint) = mime_hint {
        if !hint.is_empty() {
            let result = format!("data:{};base64,{}", hint, b64);
            let mut cache = get_media_preload_cache().lock().map_err(|e| e.to_string())?;
            cache.set(cache_key, result.clone());
            return Ok(MediaWithDims { data: result, width, height });
        }
    }

    let mime = mime_for_ext(ext.as_str());
    let result = format!("data:{};base64,{}", mime, b64);

    {
        let mut cache = get_media_preload_cache().lock().map_err(|e| e.to_string())?;
        cache.set(cache_key, result.clone());
    }

    Ok(MediaWithDims { data: result, width, height })
}

fn get_video_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    // Simple MP4/MOV dimension parser - looks for 'tkhd' (track header) box
    // This is a basic implementation that works for most MP4/MOV files
    let tkhd = b"tkhd";
    
    for i in 0..bytes.len().saturating_sub(100) {
        if &bytes[i..i+4] == tkhd {
            // tkhd box found, dimensions are at offset +76 and +80 (version 0)
            // or +88 and +92 (version 1) - try both
            let version = bytes.get(i + 4)?;
            
            let (width_offset, height_offset) = if *version == 0 {
                (i + 76, i + 80)
            } else {
                (i + 88, i + 92)
            };
            
            if height_offset + 4 <= bytes.len() {
                // Dimensions are stored as 32-bit fixed-point (16.16)
                let width_raw = u32::from_be_bytes([
                    bytes[width_offset],
                    bytes[width_offset + 1],
                    bytes[width_offset + 2],
                    bytes[width_offset + 3],
                ]);
                let height_raw = u32::from_be_bytes([
                    bytes[height_offset],
                    bytes[height_offset + 1],
                    bytes[height_offset + 2],
                    bytes[height_offset + 3],
                ]);
                
                // Convert from 16.16 fixed point to integer
                let width = width_raw >> 16;
                let height = height_raw >> 16;
                
                if width > 0 && height > 0 && width < 10000 && height < 10000 {
                    return Some((width, height));
                }
            }
        }
    }
    
    None
}

fn get_image_dimensions(bytes: &[u8]) -> Result<(u32, u32), String> {
    // Use into_dimensions() — reads only the image header, no full decode, avoids OOM bomb
    let cursor = std::io::BufReader::new(std::io::Cursor::new(bytes));
    match image::ImageReader::new(cursor).with_guessed_format() {
        Ok(reader) => {
            if let Ok((w, h)) = reader.into_dimensions() {
                return Ok((w, h));
            }
        }
        Err(_) => {}
    }
    // SVG fallback
    if let Ok(svg_str) = std::str::from_utf8(bytes) {
        if let Some((w, h)) = parse_svg_dimensions(svg_str) {
            return Ok((w, h));
        }
    }
    Ok((0, 0))
}

fn parse_svg_dimensions(svg: &str) -> Option<(u32, u32)> {
    // Simple SVG dimension parser - looks for width and height attributes
    let width_re = SVG_RE_WIDTH.get_or_init(|| regex::Regex::new(r#"width\s*=\s*["']?(\d+)"#).unwrap());
    let height_re = SVG_RE_HEIGHT.get_or_init(|| regex::Regex::new(r#"height\s*=\s*["']?(\d+)"#).unwrap());
    
    let width = width_re.captures(svg)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())?;
    
    let height = height_re.captures(svg)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok())?;
    
    Some((width, height))
}

fn convert_tiff_to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    use image::{ImageFormat, Limits};
    let mut limits = Limits::default();
    limits.max_image_width = Some(16_000);
    limits.max_image_height = Some(16_000);
    limits.max_alloc = Some(256 * 1024 * 1024);
    let cursor = std::io::BufReader::new(std::io::Cursor::new(bytes));
    let mut reader = image::ImageReader::with_format(cursor, ImageFormat::Tiff);
    reader.limits(limits); // limits() is &mut self → (), cannot be chained
    let img = reader.decode()
        .map_err(|e| format!("Failed to decode TIFF: {}", e))?;
    let mut png_buf = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_buf), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode PNG: {}", e))?;
    Ok(png_buf)
}

fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    const CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for ch in data.chars() {
        if ch == '=' {
            break;
        }
        if let Some(val) = CHARS.find(ch) {
            buffer = (buffer << 6) | (val as u32);
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                result.push((buffer >> bits) as u8);
                buffer &= (1 << bits) - 1;
            }
        }
    }

    Ok(result)
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);

    for chunk in data.chunks(3) {

        let b0 = chunk[0] as usize;

        let b1 = if chunk.len() > 1 { chunk[1] as usize } else { 0 };

        let b2 = if chunk.len() > 2 { chunk[2] as usize } else { 0 };

        out.push(CHARS[b0 >> 2] as char);

        out.push(CHARS[((b0 & 3) << 4) | (b1 >> 4)] as char);

        out.push(if chunk.len() > 1 { CHARS[((b1 & 15) << 2) | (b2 >> 6)] as char } else { '=' });

        out.push(if chunk.len() > 2 { CHARS[b2 & 63] as char } else { '=' });

    }

    out

}



fn extract_maps_url(text: &str) -> Option<String> {

    // Match both google maps and apple maps URLs with coordinates

    let patterns = [

        "maps.google.com",

        "maps.apple.com",

        "goo.gl/maps",

        "maps.app.goo.gl",

    ];

    let clean: String = text.chars().filter(|c| !matches!(*c,
        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
    )).collect();

    for part in clean.split_whitespace() {

        // Only extract if it's actually a maps URL (not youtube, etc.)
        if patterns.iter().any(|p| part.contains(p)) && !part.contains("youtube.com") && !part.contains("youtu.be") {

            let trimmed = part.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/' && c != '?' && c != '=' && c != '.' && c != ':' && c != ',' && c != '-' && c != '_' && c != '%' && c != '&' && c != '+' && c != '#').to_string();
            // Only return http/https URLs to prevent file:// or custom-scheme launches
            if validate_url_scheme(&trimmed).is_ok() {
                return Some(trimmed);
            }

        }

    }

    None

}



#[tauri::command]

fn set_display_name(chat_id: String, message_idx: i64, display_name: String) -> Result<(), String> {

    let conn = get_db();

    let name = if display_name.trim().is_empty() { None::<String> } else { Some(display_name.trim().to_string()) };

    conn.execute(

        "UPDATE messages SET display_name = ?1, display_name_modified = 1 WHERE chat_id = ?2 AND id = (

            SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3

        )",

        rusqlite::params![name, chat_id, message_idx],

    ).map_err(|e| e.to_string())?;

    Ok(())

}



#[tauri::command]

fn set_file_tag(chat_id: String, message_idx: i64, tag_ext: String) -> Result<(), String> {

    let conn = get_db();

    let tag = if tag_ext.trim().is_empty() { None::<String> } else {

        Some(tag_ext.trim().trim_start_matches('.').to_lowercase())

    };

    conn.execute(

        "UPDATE messages SET tag_ext = ?1, tag_ext_modified = 1 WHERE chat_id = ?2 AND id = (

            SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3

        )",

        rusqlite::params![tag, chat_id, message_idx],

    ).map_err(|e| e.to_string())?;

    Ok(())

}



#[tauri::command]

fn rename_media_file(chat_id: String, message_idx: i64, old_filename: String, new_filename: String) -> Result<(), String> {

    validate_chat_id(&chat_id)?;
    let old_filename = sanitize_filename(&old_filename)?;
    let new_filename = sanitize_filename(&new_filename)?;
    let app_data = get_app_data_dir();

    let media_dir = app_data.join("chats").join(&chat_id).join("media");

    let old_path = media_dir.join(&old_filename);

    let new_path = media_dir.join(&new_filename);



    // Check if old file exists

    if !old_path.exists() {

        return Err(format!("Original file not found: {:?}", old_path));

    }



    // Check if new file already exists (would overwrite)

    if new_path.exists() {

        return Err(format!("Cannot rename - file already exists: {:?}", new_path));

    }



    // Rename the actual file

    fs::rename(&old_path, &new_path).map_err(|e| format!("Failed to rename file: {}", e))?;



    // Update the database media column

    let conn = get_db();

    conn.execute(

        "UPDATE messages SET media = ?1 WHERE chat_id = ?2 AND id = (

            SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3

        )",

        rusqlite::params![new_filename, chat_id, message_idx],

    ).map_err(|e| e.to_string())?;



    Ok(())

}



#[tauri::command]

fn set_message_type(chat_id: String, message_idx: i64, msg_type: String) -> Result<(), String> {

    let conn = get_db();

    conn.execute(

        "UPDATE messages SET msg_type = ?1, msg_type_modified = 1 WHERE chat_id = ?2 AND id = (

            SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3

        )",

        rusqlite::params![msg_type, chat_id, message_idx],

    ).map_err(|e| e.to_string())?;

    Ok(())

}



#[derive(serde::Serialize)]

struct VCardContact {

    name: Option<String>,

    phones: Vec<String>,

    emails: Vec<String>,

}



#[tauri::command]

fn parse_vcard(chat_id: String, filename: String) -> Result<VCardContact, String> {

    validate_chat_id(&chat_id)?;
    let app_data = get_app_data_dir();

    let filename: String = filename.chars().filter(|c| !matches!(*c,

        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |

        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'

    )).collect();
    let filename = sanitize_filename(&filename)?;

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);

    if !media_path.exists() {

        return Err(format!("File not found: {:?}", media_path));

    }



    // Read raw bytes and handle encoding properly (including emoji)
    let bytes = std::fs::read(&media_path)
        .map_err(|e| format!("Failed to read vCard: {}", e))?;

    // Check for BOM and strip it, detect encoding
    let vcard_content = if bytes.starts_with(b"\xef\xbb\xbf") {
        encoding_rs::UTF_8.decode_without_bom_handling(&bytes[3..]).0.to_string()
    } else if bytes.starts_with(b"\xff\xfe") {
        encoding_rs::UTF_16LE.decode_without_bom_handling(&bytes[2..]).0.to_string()
    } else if bytes.starts_with(b"\xfe\xff") {
        encoding_rs::UTF_16BE.decode_without_bom_handling(&bytes[2..]).0.to_string()
    } else {
        // Try UTF-8 first
        match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => encoding_rs::UTF_16LE.decode_without_bom_handling(&bytes).0.to_string()
        }
    };

    let mut name = None;

    let mut phones = Vec::new();

    let mut emails = Vec::new();



    for line in vcard_content.lines() {

        let line = line.trim();

        let line_lower = line.to_lowercase();



        if line_lower.starts_with("fn:") || line_lower.starts_with("fn;") {
            let extracted = line.split(':').nth(1).map(|s| s.trim().to_string());
            name = extracted;
        } else if line_lower.starts_with("tel") {
            if let Some(phone) = line.split(':').nth(1) {
                let phone = phone.trim()
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '+' || *c == '-' || *c == ' ')
                    .collect::<String>();
                if !phone.is_empty() {
                    phones.push(phone);
                }
            }
        } else if line_lower.starts_with("email") {

            if let Some(email) = line.split(':').nth(1) {

                let email = email.trim().to_string();

                if !email.is_empty() {

                    emails.push(email);

                }

            }

        }

    }

    Ok(VCardContact { name, phones, emails })

}



#[tauri::command]

fn open_vcard_whatsapp(chat_id: String, filename: String, method: String) -> Result<(), String> {

    validate_chat_id(&chat_id)?;
    let app_data = get_app_data_dir();

    let filename: String = filename.chars().filter(|c| !matches!(*c,

        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |

        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'

    )).collect();
    let filename = sanitize_filename(&filename)?;

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);

    if !media_path.exists() {

        return Err(format!("File not found: {:?}", media_path));

    }



    // Read raw bytes and handle encoding properly (including emoji)
    let bytes = std::fs::read(&media_path)
        .map_err(|e| format!("Failed to read vCard: {}", e))?;

    // Check for BOM and strip it, detect encoding
    let vcard_content = if bytes.starts_with(b"\xef\xbb\xbf") {
        // UTF-8 BOM
        encoding_rs::UTF_8.decode_without_bom_handling(&bytes[3..]).0.to_string()
    } else if bytes.starts_with(b"\xff\xfe") {
        // UTF-16 LE BOM
        encoding_rs::UTF_16LE.decode_without_bom_handling(&bytes[2..]).0.to_string()
    } else if bytes.starts_with(b"\xfe\xff") {
        // UTF-16 BE BOM
        encoding_rs::UTF_16BE.decode_without_bom_handling(&bytes[2..]).0.to_string()
    } else {
        // Try UTF-8 first
        match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(_) => {
                // Fall back to UTF-16LE (common on Windows)
                encoding_rs::UTF_16LE.decode_without_bom_handling(&bytes).0.to_string()
            }
        }
    };

    // Extract phone number from vCard (TEL field)

    let phone = vcard_content.lines()

        .find_map(|line| {

            let line = line.trim();

            if line.to_lowercase().starts_with("tel") {

                let phone = line

                    .split(':')

                    .nth(1)

                    .unwrap_or("")

                    .trim()

                    .chars()

                    .filter(|c| c.is_ascii_digit() || *c == '+' || *c == '-' || *c == ' ')

                    .collect::<String>();

                if !phone.is_empty() { Some(phone) } else { None }

            } else {

                None

            }

        })

        .ok_or("No phone number found in vCard")?;



    let url = match method.as_str() {

        "app" => format!("whatsapp://send?phone={}", phone),

        "web" => format!("https://web.whatsapp.com/send?phone={}", phone),

        _ => return Err("Invalid method. Use 'app' or 'web'.".to_string()),

    };



    tauri_plugin_opener::open_url(&url, None::<&str>)

        .map_err(|e| format!("Failed to open WhatsApp: {}", e))

}



#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    validate_url_scheme(&url)?;
    tauri_plugin_opener::open_url(&url, None::<&str>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]

fn open_media_file(chat_id: String, filename: String) -> Result<(), String> {

    validate_chat_id(&chat_id)?;
    let app_data = get_app_data_dir();

    let filename: String = filename.chars().filter(|c| !matches!(*c,

        '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |

        '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'

    )).collect();
    let filename = sanitize_filename(&filename)?;

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);

    if !media_path.exists() {

        return Err(format!("File not found: {:?}", media_path));

    }

    // Only allow known-safe extensions through the OS handler to prevent click-to-execute RCE
    let ext = media_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if !is_safe_open_extension(&ext) {
        return Err(format!("File type not permitted to open: {:?}", ext));
    }

    tauri_plugin_opener::open_path(media_path.to_string_lossy().as_ref(), None::<&str>)

        .map_err(|e| e.to_string())

}



#[tauri::command]

fn get_media_path(chat_id: String, filename: String) -> Result<String, String> {

    validate_chat_id(&chat_id)?;
    let filename = sanitize_filename(&filename)?;
    let app_data = get_app_data_dir();

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);

    

    if media_path.exists() {

        Ok(media_path.to_string_lossy().to_string())

    } else {

        Err("Media file not found".to_string())

    }

}



#[tauri::command]

fn check_file_in_zip(chat_id: String, filename: String) -> Result<bool, String> {

    let conn = get_db();

    let zip_path: Option<String> = conn.query_row(

        "SELECT zip_path FROM chats WHERE id = ?1",

        [&chat_id],

        |row| row.get(0)

    ).map_err(|e| e.to_string())?;



    let zip_path = zip_path.ok_or("No archive path stored for this chat")?;

    let ext = get_archive_extension(&zip_path);

    let is_7z = ext == "7z";

    let is_rar = ext == "rar";



    if is_7z {

        let output = std::process::Command::new("7z")

            .arg("l")

            .arg(&zip_path)

            .output()

            .map_err(|e| format!("Failed to run 7z command: {}", e))?;



        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout.contains(&filename) || stdout.lines().any(|line| line.contains(&filename)))

    } else if is_rar {

        let output = std::process::Command::new("unrar")

            .arg("l")

            .arg(&zip_path)

            .output()

            .map_err(|e| format!("Failed to run unrar command: {}", e))?;



        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout.contains(&filename) || stdout.lines().any(|line| line.contains(&filename)))

    } else {

        let file = File::open(&zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;



        // Search for the file in the ZIP

        for i in 0..archive.len() {

            let file = archive.by_index(i).map_err(|e| format!("ZIP read error: {}", e))?;

            let name = file.name();

            // Check if filename matches (could be in subdirectories)

            if name.contains(&filename) || name.ends_with(&filename) {

                return Ok(true);

            }

        }

        Ok(false)

    }

}



#[tauri::command]

fn preload_media(chat_id: String, filenames: Vec<String>) -> Result<usize, String> {

    let app_data = get_app_data_dir();

    let mut loaded = 0;

    for filename in &filenames {

        let cache_key = format!("{}:{}", chat_id, filename);

        // Skip if already in cache
        {
            let mut cache = get_media_preload_cache().lock().map_err(|e| e.to_string())?;
            if cache.get(&cache_key).is_some() {
                continue;
            }
        }

        // Strip Unicode directional/zero-width marks
        let filename_clean: String = filename.chars().filter(|c| !matches!(*c,
            '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' |
            '\u{2066}'..='\u{2069}' | '\u{FEFF}' | '\u{200B}'
        )).collect();

        let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename_clean);

        if !media_path.exists() {
            continue;
        }

        if let Ok(bytes) = fs::read(&media_path) {
            // Convert TIFF to PNG for browser compatibility
            let bytes = {
                let ext_check = media_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
                if ext_check == "tiff" || ext_check == "tif" {
                    convert_tiff_to_png(&bytes).unwrap_or(bytes)
                } else {
                    bytes
                }
            };

            let b64 = base64_encode(&bytes);

            let ext = media_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let mime = match ext.as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "png"          => "image/png",
                "gif"          => "image/gif",
                "webp"         => "image/webp",
                "bmp"          => "image/bmp",
                "tiff" | "tif" => "image/png",
                "heic" | "heif" => "image/heic",
                "svg"           => "image/svg+xml",
                "mp4"          => "video/mp4",
                "mov"          => "video/quicktime",
                "avi"          => "video/x-msvideo",
                "mkv"          => "video/x-matroska",
                "webm"         => "video/webm",
                "3gp" | "3gpp" => "audio/3gpp",
                "m4v"          => "video/x-m4v",
                "ts"           => "video/mp2t",
                "opus"         => "audio/ogg; codecs=opus",
                "ogg"          => "audio/ogg",
                "mp3"          => "audio/mpeg",
                "m4a"          => "audio/mp4",
                "aac"          => "audio/aac",
                "wav"          => "audio/wav",
                "amr"          => "audio/amr",
                "flac"         => "audio/flac",
                _              => "application/octet-stream",
            };

            let result = format!("data:{};base64,{}", mime, b64);

            // Store in preload cache
            if let Ok(mut cache) = get_media_preload_cache().lock() {
                cache.set(cache_key, result);
                loaded += 1;
            }
        }
    }

    Ok(loaded)

}



#[tauri::command]

fn extract_file_from_zip(chat_id: String, filename: String) -> Result<String, String> {

    let conn = get_db();

    let zip_path: Option<String> = conn.query_row(

        "SELECT zip_path FROM chats WHERE id = ?1",

        [&chat_id],

        |row| row.get(0)

    ).map_err(|e| e.to_string())?;



    let zip_path = zip_path.ok_or("No archive path stored for this chat")?;

    validate_chat_id(&chat_id)?;
    let filename = sanitize_filename(&filename)?;
    let app_data = get_app_data_dir();

    let media_path = app_data.join("chats").join(&chat_id).join("media").join(&filename);

    let ext = get_archive_extension(&zip_path);

    let is_7z = ext == "7z";

    let is_rar = ext == "rar";



    if is_7z {

        let output = std::process::Command::new("7z")

            .arg("x")

            .arg(&zip_path)

            .arg(format!("-o{}", media_path.parent().unwrap().to_string_lossy()))

            .arg(&filename)

            .arg("-y")

            .output()

            .map_err(|e| format!("Failed to run 7z command: {}", e))?;



        if !output.status.success() {

            return Err(format!("7z extraction failed: {}", String::from_utf8_lossy(&output.stderr)));

        }



        Ok(media_path.to_string_lossy().to_string())

    } else if is_rar {

        let output = std::process::Command::new("unrar")

            .arg("x")

            .arg(&zip_path)

            .arg(&filename)

            .arg(&media_path.parent().unwrap().to_string_lossy().to_string())

            .arg("-y")

            .output()

            .map_err(|e| format!("Failed to run unrar command: {}", e))?;



        if !output.status.success() {

            return Err(format!("unrar extraction failed: {}", String::from_utf8_lossy(&output.stderr)));

        }



        Ok(media_path.to_string_lossy().to_string())

    } else {

        let file = File::open(&zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;

        let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;



        // Find and extract the file

        for i in 0..archive.len() {

            let mut zipfile = archive.by_index(i).map_err(|e| format!("ZIP read error: {}", e))?;

            let name = zipfile.name();

            // Check if filename matches (could be in subdirectories)

            if name.contains(&filename) || name.ends_with(&filename) {

                let mut out_file = File::create(&media_path).map_err(|e| format!("Failed to create file: {}", e))?;

                std::io::copy(&mut zipfile, &mut out_file).map_err(|e| format!("Failed to extract file: {}", e))?;

                return Ok(media_path.to_string_lossy().to_string());

            }

        }

        Err("File not found in archive".to_string())

    }

}



#[tauri::command]

fn debug_chat_media(chat_id: String) -> Result<String, String> {

    let app_data = get_app_data_dir();

    let media_dir = app_data.join("chats").join(&chat_id).join("media");

    

    let mut result = format!("Debug for chat: {}\n", chat_id);

    result.push_str(&format!("Media directory: {:?}\n", media_dir));

    result.push_str(&format!("Directory exists: {}\n\n", media_dir.exists()));

    

    // List files on disk

    result.push_str("Files on disk:\n");

    if media_dir.exists() {

        fn list_files_recursive(dir: &Path, prefix: &str, result: &mut String) {

            if let Ok(entries) = std::fs::read_dir(dir) {

                for entry in entries.flatten() {

                    let path = entry.path();

                    let name = path.file_name().unwrap_or_default().to_string_lossy();

                    if path.is_file() {

                        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                        result.push_str(&format!("{} - {} ({} bytes)\n", prefix, name, size));

                    } else if path.is_dir() {

                        result.push_str(&format!("{} [DIR] {}/\n", prefix, name));

                        list_files_recursive(&path, &format!("{}  ", prefix), result);

                    }

                }

            }

        }

        list_files_recursive(&media_dir, "  ", &mut result);

    }

    

    // List media from database

    result.push_str("\nMedia paths in database:\n");

    let conn = get_db();

    let mut stmt = conn.prepare(

        "SELECT media FROM messages WHERE chat_id = ?1 AND media != ''"

    ).map_err(|e| e.to_string())?;

    let media_paths: Vec<String> = stmt.query_map([&chat_id], |row| row.get(0))

        .map_err(|e| e.to_string())?

        .flatten()

        .collect();

    for path in media_paths.iter().take(20) {

        let exists = media_dir.join(path).exists();

        result.push_str(&format!("  {} (exists: {})\n", path, exists));

    }

    if media_paths.len() > 20 {

        result.push_str(&format!("  ... and {} more\n", media_paths.len() - 20));

    }

    

    Ok(result)

}



fn migrate_from_json() -> SqliteResult<()> {

    let app_data = get_app_data_dir();

    let chats_dir = app_data.join("chats");

    

    if !chats_dir.exists() {

        return Ok(());

    }

    

    let mut conn = get_db();

    

    for entry in fs::read_dir(&chats_dir).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))? {

        let entry = entry.map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

        let chat_id = entry.file_name().to_string_lossy().to_string();

        

        // Check if chat already in database

        let exists: bool = conn.query_row(

            "SELECT 1 FROM chats WHERE id = ?1",

            [&chat_id],

            |_| Ok(true)

        ).unwrap_or(false);

        

        if exists {

            continue; // Already migrated

        }

        

        // Read meta.json

        let meta_path = entry.path().join("meta.json");

        let messages_path = entry.path().join("messages.json");

        

        if let Ok(meta_content) = fs::read_to_string(&meta_path) {

            if let Ok(meta) = serde_json::from_str::<ChatMeta>(&meta_content) {

                // Insert chat

                conn.execute(

                    "INSERT INTO chats (id, name, last_message, timestamp, is_group, zip_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",

                    params![&chat_id, &meta.name, &meta.last_message, &meta.timestamp, meta.is_group as i32, meta.zip_path.as_deref()],

                )?;

                

                // Migrate messages

                if let Ok(msg_content) = fs::read_to_string(&messages_path) {

                    if let Ok(chat_data) = serde_json::from_str::<ChatData>(&msg_content) {

                        let tx = conn.transaction()?;

                        {

                            let mut stmt = tx.prepare(

                                "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media, duration) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"

                            )?;

                            

                            for msg in &chat_data.messages {

                                stmt.execute(params![

                                    &chat_id,

                                    &msg.timestamp,

                                    &msg.sender,

                                    &msg.msg_type,

                                    &msg.content,

                                    msg.media.as_ref().unwrap_or(&String::new()),

                                    msg.duration.as_ref().unwrap_or(&String::new())

                                ])?;

                            }

                        }

                        tx.commit()?;

                    }

                }

            }

        }

    }

    

    Ok(())

}



#[tauri::command]

fn delete_chat(chat_id: String) -> Result<(), String> {

    validate_chat_id(&chat_id)?;
    let app_data = get_app_data_dir();

    let chat_dir = app_data.join("chats").join(&chat_id);

    let import_dir = app_data.join("imports").join(&chat_id);

    

    // Delete from SQLite (cascade will delete messages)

    let conn = get_db();

    conn.execute("DELETE FROM chats WHERE id = ?1", params![&chat_id])

        .map_err(|e| e.to_string())?;

    

    if chat_dir.exists() {

        fs::remove_dir_all(&chat_dir).map_err(|e| e.to_string())?;

    }

    

    if import_dir.exists() {

        fs::remove_dir_all(&import_dir).map_err(|e| e.to_string())?;

    }

    

    Ok(())

}



#[tauri::command]
fn clear_all_chats() -> Result<u32, String> {
    let conn = get_db();
    let app_data = get_app_data_dir();

    // Get all chat IDs first
    let mut stmt = conn.prepare("SELECT id FROM chats").map_err(|e| e.to_string())?;
    let ids: Vec<String> = stmt.query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let count = ids.len() as u32;

    // Delete all rows (messages cascade via FK)
    conn.execute("DELETE FROM chats", []).map_err(|e| e.to_string())?;

    // Remove all chat directories
    for id in &ids {
        let chat_dir = app_data.join("chats").join(id);
        let import_dir = app_data.join("imports").join(id);
        if chat_dir.exists() { let _ = fs::remove_dir_all(&chat_dir); }
        if import_dir.exists() { let _ = fs::remove_dir_all(&import_dir); }
    }

    // Clear preload cache
    if let Ok(mut cache) = get_media_preload_cache().lock() {
        *cache = MediaPreloadCache::new(50);
    }

    Ok(count)
}

#[tauri::command]

fn migrate_chats() -> Result<i32, String> {

    migrate_from_json().map_err(|e| e.to_string())?;

    

    // Count migrated chats

    let conn = get_db();

    let count: i32 = conn.query_row("SELECT COUNT(*) FROM chats", [], |row| row.get(0))

        .map_err(|e| e.to_string())?;

    

    Ok(count)

}



#[derive(Serialize, Deserialize, Debug)]

pub struct SearchResult {

    pub message_index: usize,

    pub timestamp: String,

    pub sender: String,

    pub content: String,

    pub msg_type: String,

}



#[derive(Serialize, Deserialize, Debug)]

pub struct SearchFilters {

    pub query: String,

    pub date_from: Option<String>,

    pub date_to: Option<String>,

    pub sender: Option<String>,

    pub msg_type: Option<String>,

}



#[tauri::command]

fn search_messages(chat_id: String, query: String) -> Result<Vec<SearchResult>, String> {

    let conn = get_db();

    

    // Fetch all messages for this chat to get correct indices

    let mut stmt = conn.prepare(

        "SELECT id, timestamp, sender, msg_type, content FROM messages 

         WHERE chat_id = ?1 ORDER BY id"

    ).map_err(|e| e.to_string())?;

    

    let search_lower = query.to_lowercase();

    let mut results = Vec::new();

    let mut index: usize = 0;

    

    let rows = stmt.query_map([&chat_id], |row| {

        Ok((

            row.get::<_, i64>(0)?, // id

            row.get::<_, String>(1)?, // timestamp

            row.get::<_, String>(2)?, // sender

            row.get::<_, String>(3)?, // msg_type

            row.get::<_, String>(4)?, // content

        ))

    }).map_err(|e| e.to_string())?;

    

    for row in rows {

        let (_id, timestamp, sender, msg_type, content) = row.map_err(|e| e.to_string())?;

        

        // Check if content, sender, or timestamp matches

        if content.to_lowercase().contains(&search_lower) || 

           sender.to_lowercase().contains(&search_lower) ||

           timestamp.to_lowercase().contains(&search_lower) {

            results.push(SearchResult {

                message_index: index,

                timestamp,

                sender,

                msg_type,

                content,

            });

        }

        index += 1;

    }

    

    Ok(results)

}



#[tauri::command]

fn search_messages_filtered(chat_id: String, filters: SearchFilters) -> Result<Vec<SearchResult>, String> {

    let conn = get_db();

    

    // Build WHERE clause dynamically based on filters

    let mut where_conditions = vec!["chat_id = ?1".to_string()];

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(chat_id.clone())];

    let mut param_index: usize = 2;

    

    if let Some(date_from) = &filters.date_from {

        where_conditions.push(format!("timestamp >= ?{}", param_index));

        params.push(Box::new(date_from.clone()));

        param_index += 1;

    }

    

    if let Some(date_to) = &filters.date_to {

        where_conditions.push(format!("timestamp <= ?{}", param_index));

        params.push(Box::new(date_to.clone()));

        param_index += 1;

    }

    

    if let Some(sender_filter) = &filters.sender {

        where_conditions.push(format!("LOWER(sender) LIKE ?{}", param_index));

        params.push(Box::new(format!("%{}%", sender_filter.to_lowercase())));

        param_index += 1;

    }

    

    if let Some(msg_type_filter) = &filters.msg_type {

        where_conditions.push(format!("msg_type = ?{}", param_index));

        params.push(Box::new(msg_type_filter.clone()));

        param_index += 1;

    }

    let _ = param_index;

    

    let where_clause = where_conditions.join(" AND ");

    

    // Fetch filtered messages

    let mut stmt = conn.prepare(&format!(

        "SELECT id, timestamp, sender, msg_type, content FROM messages 

         WHERE {} ORDER BY id",

        where_clause

    )).map_err(|e| e.to_string())?;

    

    let search_lower = filters.query.to_lowercase();

    let mut results = Vec::new();

    let mut index: usize = 0;

    

    // Convert params for query_map

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    

    let rows = stmt.query_map(&param_refs[..], |row| {

        Ok((

            row.get::<_, i64>(0)?, // id

            row.get::<_, String>(1)?, // timestamp

            row.get::<_, String>(2)?, // sender

            row.get::<_, String>(3)?, // msg_type

            row.get::<_, String>(4)?, // content

        ))

    }).map_err(|e| e.to_string())?;

    

    for row in rows {

        let (_id, timestamp, sender, msg_type, content) = row.map_err(|e| e.to_string())?;

        

        // Apply text search filter if query is not empty

        if filters.query.is_empty() || 

           content.to_lowercase().contains(&search_lower) || 

           sender.to_lowercase().contains(&search_lower) ||

           timestamp.to_lowercase().contains(&search_lower) {

            results.push(SearchResult {

                message_index: index,

                timestamp,

                sender,

                msg_type,

                content,

            });

        }

        index += 1;

    }

    

    Ok(results)

}



#[tauri::command]

fn search_chats(query: String) -> Result<Vec<ChatMeta>, String> {

    let conn = get_db();

    

    let search_lower = format!("%{}%", query.to_lowercase());

    

    let mut stmt = conn.prepare(

        "SELECT chats.id, COALESCE(profiles.name, chats.name), chats.last_message, chats.timestamp, chats.is_group, chats.zip_path, profiles.photo_path
         FROM chats LEFT JOIN profiles ON chats.id = profiles.chat_id
         WHERE LOWER(COALESCE(profiles.name, chats.name)) LIKE ?1 OR LOWER(chats.last_message) LIKE ?1
         ORDER BY chats.last_message_epoch DESC, chats.created_at DESC"

    ).map_err(|e| e.to_string())?;

    

    let chats = stmt.query_map([&search_lower], |row| {

        Ok(ChatMeta {

            id: row.get(0)?,

            name: row.get(1)?,

            last_message: row.get(2)?,

            timestamp: row.get(3)?,

            is_group: row.get::<_, i32>(4)? != 0,

            zip_path: row.get(5)?,

            photo_path: row.get(6)?,

        })

    }).map_err(|e| e.to_string())?;

    

    let mut result = Vec::new();

    for chat in chats {

        result.push(chat.map_err(|e| e.to_string())?);

    }

    

    Ok(result)

}



#[derive(Serialize, Deserialize, Debug)]

pub struct Profile {

    pub chat_id: String,

    pub name: Option<String>,

    pub notes: Option<String>,

    pub photo_path: Option<String>,

    pub phone_number: Option<String>,

}



#[tauri::command]

fn get_profile(chat_id: String) -> Result<Profile, String> {

    let conn = get_db();

    

    let mut stmt = conn.prepare(

        "SELECT chat_id, name, notes, photo_path, phone_number FROM profiles WHERE chat_id = ?1"

    ).map_err(|e| e.to_string())?;



    let profile = stmt.query_row([&chat_id], |row| {

        Ok(Profile {

            chat_id: row.get(0)?,

            name: row.get(1)?,

            notes: row.get(2)?,

            photo_path: row.get(3)?,

            phone_number: row.get(4)?,

        })

    });



    match profile {

        Ok(p) => Ok(p),

        Err(_) => Ok(Profile {

            chat_id,

            name: None,

            notes: None,

            photo_path: None,

            phone_number: None,

        })

    }

}



#[tauri::command]

fn update_profile(chat_id: String, name: Option<String>, notes: Option<String>, photo_path: Option<String>, phone_number: Option<String>) -> Result<(), String> {

    let conn = get_db();



    // Fetch current name so we only record history when it actually changes

    let current_name: Option<String> = conn.query_row(

        "SELECT name FROM profiles WHERE chat_id = ?1",

        params![&chat_id],

        |row| row.get(0),

    ).ok().flatten();



    conn.execute(

        "INSERT INTO profiles (chat_id, name, notes, photo_path, phone_number, profile_modified)

         VALUES (?1, ?2, ?3, ?4, ?5, 1)

         ON CONFLICT(chat_id) DO UPDATE SET

         name = COALESCE(?2, name),

         notes = COALESCE(?3, notes),

         photo_path = COALESCE(?4, photo_path),

         phone_number = COALESCE(?5, phone_number),

         profile_modified = 1",

        params![&chat_id, &name, &notes, &photo_path, &phone_number],

    ).map_err(|e| e.to_string())?;



    // Record name change in history if the name actually changed

    if let Some(new_name) = &name {

        let changed = match &current_name {

            Some(old) => old != new_name,

            None => true,

        };

        if changed {

            conn.execute(

                "INSERT INTO chat_name_history (chat_id, name) VALUES (?1, ?2)",

                params![&chat_id, new_name],

            ).map_err(|e| e.to_string())?;

        }

    }



    Ok(())

}



#[tauri::command]

fn remove_profile_photo(chat_id: String) -> Result<(), String> {

    let conn = get_db();



    conn.execute(

        "UPDATE profiles SET photo_path = NULL WHERE chat_id = ?1",

        params![&chat_id],

    ).map_err(|e| e.to_string())?;



    Ok(())

}



#[derive(Serialize, Deserialize, Debug)]

pub struct NameHistoryEntry {

    pub id: i64,

    pub name: String,

    pub changed_at: String,

}



#[tauri::command]

fn get_name_history(chat_id: String) -> Result<Vec<NameHistoryEntry>, String> {

    let conn = get_db();

    let mut stmt = conn.prepare(

        "SELECT id, name, changed_at FROM chat_name_history WHERE chat_id = ?1 ORDER BY changed_at DESC"

    ).map_err(|e| e.to_string())?;

    let entries = stmt.query_map(params![&chat_id], |row| {

        Ok(NameHistoryEntry {

            id: row.get(0)?,

            name: row.get(1)?,

            changed_at: row.get(2)?,

        })

    }).map_err(|e| e.to_string())?;

    Ok(entries.flatten().collect())

}



#[tauri::command]

fn revert_profile_name(chat_id: String, name: Option<String>) -> Result<(), String> {

    // name = Some(x) restores to x, name = None resets to original (clears profiles.name)

    let conn = get_db();

    conn.execute(

        "INSERT INTO profiles (chat_id, name, notes, photo_path)

         VALUES (?1, ?2, NULL, NULL)

         ON CONFLICT(chat_id) DO UPDATE SET name = ?2",

        params![&chat_id, &name],

    ).map_err(|e| e.to_string())?;

    // Only log restores to a specific name, not resets to original

    if let Some(restored_name) = &name {

        conn.execute(

            "INSERT INTO chat_name_history (chat_id, name) VALUES (?1, ?2)",

            params![&chat_id, restored_name],

        ).map_err(|e| e.to_string())?;

    }

    Ok(())

}

#[tauri::command]
fn export_chat_modifications(chat_id: String) -> Result<String, String> {

    let conn = get_db();
    
    // Build list of all modifications

    let mut all_modifications: Vec<serde_json::Value> = Vec::new();
    
    // 1. Export message modifications (display_name, tag_ext, msg_type changes)

    // Use ROW_NUMBER() to compute message index since we don't have a message_index column

    let mut stmt = conn.prepare(

        "SELECT 

            m1.id,

            (SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = m1.chat_id AND m2.id <= m1.id) - 1 as msg_idx,

            m1.display_name, 

            m1.tag_ext, 

            m1.msg_type 

         FROM messages m1

         WHERE m1.chat_id = ?1 AND (m1.display_name_modified = 1 OR m1.tag_ext_modified = 1 OR m1.msg_type_modified = 1)"

    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([&chat_id], |row| {

        Ok((

            row.get::<_, i64>(0)?,  // id

            row.get::<_, i64>(1)?,  // msg_idx

            row.get::<_, Option<String>>(2)?,  // display_name

            row.get::<_, Option<String>>(3)?,  // tag_ext

            row.get::<_, String>(4)?,  // msg_type

        ))

    }).map_err(|e| e.to_string())?;
    
    for row in rows {

        let (_id, msg_idx, display_name, tag_ext, msg_type) = row.map_err(|e| e.to_string())?;

        all_modifications.push(serde_json::json!({

            "type": "message",

            "message_index": msg_idx,

            "display_name": display_name,

            "tag_ext": tag_ext,

            "msg_type": msg_type

        }));

    }

    // 2. Check for profile modifications (only if profile_modified flag is set)

    let profile_modified: bool = conn.query_row(

        "SELECT 1 FROM profiles WHERE chat_id = ?1 AND profile_modified = 1",

        [&chat_id],

        |_| Ok(true)

    ).unwrap_or(false);

    

    if profile_modified {

        let profile_data: (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) = conn.query_row(

            "SELECT name, notes, photo_path, phone_number, background_path FROM profiles WHERE chat_id = ?1",

            [&chat_id],

            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))

        ).map_err(|e| e.to_string())?;

        

        all_modifications.push(serde_json::json!({

            "type": "profile",

            "name": profile_data.0,

            "notes": profile_data.1,

            "photo_path": profile_data.2,

            "phone_number": profile_data.3,

            "background_path": profile_data.4

        }));

    }

    serde_json::to_string(&all_modifications).map_err(|e| e.to_string())

}



#[tauri::command]

fn apply_chat_modifications(chat_id: String, modifications_json: String) -> Result<(), String> {

    let conn = get_db();
    
    let modifications: Vec<serde_json::Value> = 

        serde_json::from_str(&modifications_json).map_err(|e| e.to_string())?;
    
    for mod_entry in modifications {

        let mod_type = mod_entry.get("type").and_then(|v| v.as_str()).unwrap_or("message");
        

        if mod_type == "message" {

            let message_index = mod_entry.get("message_index").and_then(|v| v.as_i64()).unwrap_or(0);

            let display_name = mod_entry.get("display_name").and_then(|v| v.as_str()).map(|s| s.to_string());

            let tag_ext = mod_entry.get("tag_ext").and_then(|v| v.as_str()).map(|s| s.to_string());

            let msg_type = mod_entry.get("msg_type").and_then(|v| v.as_str()).unwrap_or("text");
            

            // Apply updates one by one using parameterized queries to avoid SQL injection

            if let Some(name) = display_name {

                conn.execute(

                    "UPDATE messages SET display_name = ?4, display_name_modified = 1 WHERE chat_id = ?1 AND id = (SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3)",

                    [&chat_id, &chat_id, &message_index.to_string(), &name]

                ).map_err(|e| e.to_string())?;

            }

            if let Some(tag) = tag_ext {

                conn.execute(

                    "UPDATE messages SET tag_ext = ?4, tag_ext_modified = 1 WHERE chat_id = ?1 AND id = (SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3)",

                    [&chat_id, &chat_id, &message_index.to_string(), &tag]

                ).map_err(|e| e.to_string())?;

            }

            if msg_type != "text" {

                let msg_type_owned = msg_type.to_string();

                conn.execute(

                    "UPDATE messages SET msg_type = ?4, msg_type_modified = 1 WHERE chat_id = ?1 AND id = (SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3)",

                    [&chat_id, &chat_id, &message_index.to_string(), &msg_type_owned]

                ).map_err(|e| e.to_string())?;

            }

        } else if mod_type == "profile" {

            // Restore profile data

            let name = mod_entry.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());

            let notes = mod_entry.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string());

            let photo_path = mod_entry.get("photo_path").and_then(|v| v.as_str()).map(|s| s.to_string());

            let phone_number = mod_entry.get("phone_number").and_then(|v| v.as_str()).map(|s| s.to_string());

            let background_path = mod_entry.get("background_path").and_then(|v| v.as_str()).map(|s| s.to_string());

            

            conn.execute(

                "INSERT INTO profiles (chat_id, name, notes, photo_path, phone_number, background_path, profile_modified)

                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)

                 ON CONFLICT(chat_id) DO UPDATE SET

                 name = COALESCE(?2, name),

                 notes = COALESCE(?3, notes),

                 photo_path = COALESCE(?4, photo_path),

                 phone_number = COALESCE(?5, phone_number),

                 background_path = COALESCE(?6, background_path),

                 profile_modified = 1",

                rusqlite::params![&chat_id, &name, &notes, &photo_path, &phone_number, &background_path],

            ).map_err(|e| e.to_string())?;

        }

    }

    Ok(())

}



#[tauri::command]

fn clear_modification_flags(chat_id: String) -> Result<(), String> {

    let conn = get_db();

    

    // Clear all message modification flags for this chat

    conn.execute(

        "UPDATE messages SET display_name_modified = 0, tag_ext_modified = 0, msg_type_modified = 0 WHERE chat_id = ?1",

        [&chat_id],

    ).map_err(|e| e.to_string())?;

    

    // Clear profile_modified flag for this chat

    conn.execute(

        "UPDATE profiles SET profile_modified = 0 WHERE chat_id = ?1",

        [&chat_id],

    ).map_err(|e| e.to_string())?;

    

    Ok(())

}



#[tauri::command]

async fn pick_profile_photo(app: tauri::AppHandle) -> Result<Option<String>, String> {

    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog().file()

        .add_filter("Images", &["png", "jpg", "jpeg", "jfif", "gif", "webp", "bmp", "tiff", "tif", "avif", "svg", "ico"])

        .pick_file(move |file_path| {

            let _ = tx.send(file_path);

        });

    let result = rx.await.map_err(|e| e.to_string())?;

    let picked = match result {

        Some(p) => p.to_string(),

        None => return Ok(None),

    };

    // Copy image into profile_photos dir so it's within allowed scope
    let app_data = get_app_data_dir();
    let profile_dir = app_data.join("profile_photos");
    fs::create_dir_all(&profile_dir).map_err(|e| format!("Failed to create profile photos dir: {}", e))?;

    let src_path = std::path::Path::new(&picked);
    let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dest_filename = format!("profile_{}.{}", timestamp, ext);
    let dest_path = profile_dir.join(&dest_filename);
    fs::copy(&src_path, &dest_path).map_err(|e| format!("Failed to copy profile photo: {}", e))?;

    Ok(Some(dest_path.to_string_lossy().to_string()))

}



#[tauri::command]
async fn pick_global_background(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file()
        .add_filter("Images", &["png", "jpg", "jpeg", "jfif", "gif", "webp", "bmp", "tiff", "tif", "avif", "svg", "ico"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });
    let result = rx.await.map_err(|e| e.to_string())?;
    Ok(result.map(|file| file.to_string()))
}

#[tauri::command]
fn save_background_from_b64(b64: String, ext: String) -> Result<String, String> {
    let bytes = base64_decode(&b64).map_err(|e| format!("Failed to decode base64: {}", e))?;
    let bg_dir = get_app_data_dir().join("user_backgrounds");
    fs::create_dir_all(&bg_dir).map_err(|e| e.to_string())?;
    let safe_ext = match ext.to_lowercase().as_str() {
        "jpg" | "jpeg" | "jfif" => "jpg",
        "png" => "png",
        "gif" => "gif",
        "webp" => "webp",
        "bmp" => "bmp",
        "svg" => "svg",
        _ => "jpg",
    };
    let filename = format!("{}.{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(), safe_ext);
    let out_path = bg_dir.join(&filename);
    fs::write(&out_path, &bytes).map_err(|e| e.to_string())?;
    Ok(out_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn read_file_as_base64(path: String) -> Result<String, String> {
    let app_data = get_app_data_dir();
    if !std::path::Path::new(&path).starts_with(&app_data) {
        return Err("Path is outside permitted directory".to_string());
    }
    let bytes = fs::read(&path).map_err(|e| e.to_string())?;
    let b64 = base64_encode(&bytes);
    let ext = Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        "jfif" => "image/jpeg",
        "tiff" | "tif" => "image/tiff",
        "avif" => "image/avif",
        "ico" => "image/x-icon",
        _ => "image/jpeg",
    };
    Ok(format!("data:{};base64,{}", mime, b64))
}



#[derive(Serialize, Deserialize, Debug)]
pub struct BackgroundHistoryEntry {
    pub id: i64,
    pub background_path: String,
    pub changed_at: String,
}

#[tauri::command]
async fn set_chat_background(app: tauri::AppHandle, chat_id: String) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog().file()
        .add_filter("Images", &["png", "jpg", "jpeg", "jfif", "gif", "webp", "bmp", "tiff", "tif", "avif", "svg", "ico"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    let result = rx.await.map_err(|e| e.to_string())?;
    let picked = match result {
        Some(p) => p.to_string(),
        None => return Ok(None),
    };

    // Copy image into custom/ dir so it travels with exports
    let app_data = get_app_data_dir();
    let custom_dir = app_data.join("chats").join(&chat_id).join("custom");
    ensure_dir_exists(&custom_dir);

    let src_path = std::path::Path::new(&picked);
    let ext = src_path.extension().and_then(|e| e.to_str()).unwrap_or("jpg");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let dest_filename = format!("bg_{}.{}", timestamp, ext);
    let dest_path = custom_dir.join(&dest_filename);
    fs::copy(&src_path, &dest_path).map_err(|e| format!("Failed to copy background: {}", e))?;
    let dest_str = dest_path.to_string_lossy().to_string();

    let conn = get_db();

    conn.execute(
        "INSERT INTO profiles (chat_id, background_path, profile_modified)
         VALUES (?1, ?2, 1)
         ON CONFLICT(chat_id) DO UPDATE SET background_path = ?2, profile_modified = 1",
        params![&chat_id, &dest_str],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO chat_background_history (chat_id, background_path) VALUES (?1, ?2)",
        params![&chat_id, &dest_str],
    ).map_err(|e| e.to_string())?;

    Ok(Some(dest_str))
}

#[tauri::command]
fn get_background_history(chat_id: String) -> Result<Vec<BackgroundHistoryEntry>, String> {
    let conn = get_db();
    let mut stmt = conn.prepare(
        "SELECT id, background_path, changed_at FROM chat_background_history WHERE chat_id = ?1 ORDER BY changed_at DESC"
    ).map_err(|e| e.to_string())?;
    let entries = stmt.query_map(params![&chat_id], |row| {
        Ok(BackgroundHistoryEntry {
            id: row.get(0)?,
            background_path: row.get(1)?,
            changed_at: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(entries.flatten().collect())
}

#[tauri::command]
fn get_chat_background(chat_id: String) -> Result<Option<String>, String> {
    let conn = get_db();
    let result: Option<String> = conn.query_row(
        "SELECT background_path FROM profiles WHERE chat_id = ?1",
        params![&chat_id],
        |row| row.get(0),
    ).ok().flatten();
    Ok(result)
}

#[tauri::command]
fn restore_chat_background(chat_id: String, background_path: String) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "INSERT INTO profiles (chat_id, background_path, profile_modified)
         VALUES (?1, ?2, 1)
         ON CONFLICT(chat_id) DO UPDATE SET background_path = ?2, profile_modified = 1",
        params![&chat_id, &background_path],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO chat_background_history (chat_id, background_path) VALUES (?1, ?2)",
        params![&chat_id, &background_path],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn clear_chat_background(chat_id: String) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "UPDATE profiles SET background_path = NULL, profile_modified = 1 WHERE chat_id = ?1",
        params![&chat_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn toggle_message_favorite(chat_id: String, message_idx: i64, is_favorite: bool) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "UPDATE messages SET is_favorite = ?1 WHERE chat_id = ?2 AND id = (
            SELECT id FROM messages WHERE chat_id = ?3 ORDER BY id LIMIT 1 OFFSET ?4
        )",
        params![if is_favorite { 1 } else { 0 }, &chat_id, &chat_id, message_idx],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_favorite_messages(chat_id: String) -> Result<ChatData, String> {
    let conn = get_db();

    // Get total message count first
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE chat_id = ?1",
        [&chat_id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT (SELECT COUNT(*) FROM messages WHERE chat_id = ?1 AND id <= m.id) - 1 as row_index, timestamp, sender, msg_type, content, media, duration, tag_ext, display_name, is_favorite FROM messages m WHERE chat_id = ?1 AND is_favorite = 1 ORDER BY id DESC"
    ).map_err(|e| e.to_string())?;

    let messages = stmt.query_map([&chat_id], |row| {
        let row_index: i64 = row.get(0)?;
        let media: String = row.get(5)?;
        let duration: String = row.get(6)?;
        let tag_ext: Option<String> = row.get(7)?;
        let display_name: Option<String> = row.get(8)?;
        let is_favorite: Option<i64> = row.get(9)?;

        Ok(Message {
            id: Some(row_index),
            timestamp: row.get(1)?,
            sender: row.get(2)?,
            msg_type: row.get(3)?,
            content: row.get(4)?,
            media: if media.is_empty() { None } else { Some(media) },
            duration: if duration.is_empty() { None } else { Some(duration) },
            tag_ext,
            display_name,
            is_favorite: Some(is_favorite.unwrap_or(0) != 0),
        })
    }).map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for msg in messages {
        result.push(msg.map_err(|e| e.to_string())?);
    }

    Ok(ChatData { messages: result })
}

#[derive(Serialize)]
struct ImportProgress {
    active: bool,
    messages: usize,
    media: usize,
    phase: String,
}

#[tauri::command]
fn get_import_progress() -> ImportProgress {
    let phase = match IMPORT_PHASE.load(Ordering::Relaxed) {
        1 => "Extracting files...",
        2 => "Preparing database...",
        3 => "Saving messages...",
        4 => "Done",
        _ => "",
    };
    ImportProgress {
        active: IMPORT_ACTIVE.load(Ordering::Relaxed) == 1,
        messages: IMPORT_MSG_COUNT.load(Ordering::Relaxed),
        media: IMPORT_MEDIA_COUNT.load(Ordering::Relaxed),
        phase: phase.to_string(),
    }
}

// --- Export functions ---

fn build_chat_export_data(conn: &Connection, chat_id: &str) -> Result<(String, Vec<u8>, Vec<u8>, Vec<u8>), String> {
    // Get chat metadata
    let (name, original_name, is_group, zip_path): (String, Option<String>, i32, Option<String>) = conn.query_row(
        "SELECT name, original_name, is_group, zip_path FROM chats WHERE id = ?1",
        params![chat_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|e| format!("Chat not found: {}", e))?;

    // Use original_name for folder name if available, else chat name
    let folder_name = original_name.unwrap_or_else(|| name.clone());
    // Sanitize folder name for filesystem
    let folder_name: String = folder_name.chars().map(|c| match c {
        '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
        _ => c,
    }).collect();

    // Build meta.json
    let meta = serde_json::json!({
        "id": chat_id,
        "name": name,
        "is_group": is_group != 0,
        "zip_path": zip_path,
        "export_version": 1
    });
    let meta_bytes = serde_json::to_vec_pretty(&meta).map_err(|e| e.to_string())?;

    // Build messages.json from DB
    let mut stmt = conn.prepare(
        "SELECT timestamp, sender, msg_type, content, media, duration, tag_ext, display_name FROM messages WHERE chat_id = ?1 ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    let messages: Vec<Message> = stmt.query_map(params![chat_id], |row| {
        let media: String = row.get(4)?;
        let duration: String = row.get(5)?;
        Ok(Message {
            id: None,
            timestamp: row.get(0)?,
            sender: row.get(1)?,
            msg_type: row.get(2)?,
            content: row.get(3)?,
            media: if media.is_empty() { None } else { Some(media) },
            duration: if duration.is_empty() { None } else { Some(duration) },
            tag_ext: row.get(6)?,
            display_name: row.get(7)?,
            is_favorite: None,
        })
    }).map_err(|e| e.to_string())?
    .collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;

    let chat_data = ChatData { messages };
    let messages_bytes = serde_json::to_vec_pretty(&chat_data).map_err(|e| e.to_string())?;

    // Build modifications.json
    let modifications: String = export_chat_modifications_internal(conn, chat_id)?;
    let modifications_bytes = modifications.into_bytes();

    Ok((folder_name, meta_bytes, messages_bytes, modifications_bytes))
}

fn export_chat_modifications_internal(conn: &Connection, chat_id: &str) -> Result<String, String> {
    let mut all_modifications: Vec<serde_json::Value> = Vec::new();

    let mut stmt = conn.prepare(
        "SELECT 
            m1.id,
            (SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = m1.chat_id AND m2.id <= m1.id) - 1 as msg_idx,
            m1.display_name, 
            m1.tag_ext, 
            m1.msg_type 
         FROM messages m1
         WHERE m1.chat_id = ?1 AND (m1.display_name_modified = 1 OR m1.tag_ext_modified = 1 OR m1.msg_type_modified = 1)"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([chat_id], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Option<String>>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, String>(4)?,
        ))
    }).map_err(|e| e.to_string())?;

    for row in rows {
        let (_id, msg_idx, display_name, tag_ext, msg_type) = row.map_err(|e| e.to_string())?;
        all_modifications.push(serde_json::json!({
            "type": "message",
            "message_index": msg_idx,
            "display_name": display_name,
            "tag_ext": tag_ext,
            "msg_type": msg_type
        }));
    }

    let profile_modified: bool = conn.query_row(
        "SELECT 1 FROM profiles WHERE chat_id = ?1 AND profile_modified = 1",
        [chat_id],
        |_| Ok(true)
    ).unwrap_or(false);

    if profile_modified {
        let profile_data: (Option<String>, Option<String>, Option<String>, Option<String>) = conn.query_row(
            "SELECT name, notes, photo_path, phone_number FROM profiles WHERE chat_id = ?1",
            [chat_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        ).map_err(|e| e.to_string())?;

        all_modifications.push(serde_json::json!({
            "type": "profile",
            "name": profile_data.0,
            "notes": profile_data.1,
            "photo_path": profile_data.2,
            "phone_number": profile_data.3
        }));
    }

    serde_json::to_string(&all_modifications).map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_chat_zip(app: tauri::AppHandle, chat_id: String) -> Result<String, String> {
    let (folder_name, meta_bytes, messages_bytes, modifications_bytes) = {
        let conn = get_db();
        build_chat_export_data(&conn, &chat_id)?
    };

    // Pick save location
    let (tx, rx) = tokio::sync::oneshot::channel();
    let default_name = format!("{}.zip", folder_name);
    tauri_plugin_dialog::FileDialogBuilder::new(app.dialog().clone())
        .add_filter("ZIP Archive", &["zip"])
        .set_file_name(&default_name)
        .save_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    let save_path = match rx.await.map_err(|e| e.to_string())? {
        Some(path) => path.as_path().map(|p| p.to_path_buf()).ok_or("Invalid path")?,
        None => return Ok("cancelled".to_string()),
    };

    // Create ZIP
    let file = File::create(&save_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Write meta.json
    zip.start_file(format!("{}/meta.json", folder_name), options).map_err(|e| e.to_string())?;
    zip.write_all(&meta_bytes).map_err(|e| e.to_string())?;

    // Write messages.json
    zip.start_file(format!("{}/messages.json", folder_name), options).map_err(|e| e.to_string())?;
    zip.write_all(&messages_bytes).map_err(|e| e.to_string())?;

    // Write modifications.json
    zip.start_file(format!("{}/modifications.json", folder_name), options).map_err(|e| e.to_string())?;
    zip.write_all(&modifications_bytes).map_err(|e| e.to_string())?;

    // Write media files
    let media_dir = get_app_data_dir().join("chats").join(&chat_id).join("media");
    if media_dir.exists() {
        if let Ok(entries) = fs::read_dir(&media_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    if let Ok(data) = fs::read(&path) {
                        zip.start_file(format!("{}/media/{}", folder_name, filename), options).map_err(|e| e.to_string())?;
                        zip.write_all(&data).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    // Write custom files (profile photos etc)
    let custom_dir = get_app_data_dir().join("chats").join(&chat_id).join("custom");
    if custom_dir.exists() {
        if let Ok(entries) = fs::read_dir(&custom_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let filename = path.file_name().unwrap().to_string_lossy().to_string();
                    if let Ok(data) = fs::read(&path) {
                        zip.start_file(format!("{}/custom/{}", folder_name, filename), options).map_err(|e| e.to_string())?;
                        zip.write_all(&data).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(save_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn export_all_chats_zip(app: tauri::AppHandle) -> Result<String, String> {
    // Get all chat IDs
    let chat_ids: Vec<String> = {
        let conn = get_db();
        let mut stmt = conn.prepare("SELECT id FROM chats ORDER BY last_message_epoch DESC")
            .map_err(|e| e.to_string())?;
        let result = stmt.query_map([], |row| row.get(0))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        result
    };

    if chat_ids.is_empty() {
        return Err("No chats to export".to_string());
    }

    // Pick save location
    let (tx, rx) = tokio::sync::oneshot::channel();
    tauri_plugin_dialog::FileDialogBuilder::new(app.dialog().clone())
        .add_filter("ZIP Archive", &["zip"])
        .set_file_name("WhatsApp_Backup_Export.zip")
        .save_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    let save_path = match rx.await.map_err(|e| e.to_string())? {
        Some(path) => path.as_path().map(|p| p.to_path_buf()).ok_or("Invalid path")?,
        None => return Ok("cancelled".to_string()),
    };

    // Create ZIP
    let file = File::create(&save_path).map_err(|e| format!("Failed to create file: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Track folder names to avoid collisions
    let mut used_names: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for chat_id in &chat_ids {
        let (mut folder_name, meta_bytes, messages_bytes, modifications_bytes) = {
            let conn = get_db();
            build_chat_export_data(&conn, chat_id)?
        };

        // Handle duplicate folder names
        let count = used_names.entry(folder_name.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            folder_name = format!("{}_{}", folder_name, count);
        }

        // Write meta.json
        zip.start_file(format!("{}/meta.json", folder_name), options).map_err(|e| e.to_string())?;
        zip.write_all(&meta_bytes).map_err(|e| e.to_string())?;

        // Write messages.json
        zip.start_file(format!("{}/messages.json", folder_name), options).map_err(|e| e.to_string())?;
        zip.write_all(&messages_bytes).map_err(|e| e.to_string())?;

        // Write modifications.json
        zip.start_file(format!("{}/modifications.json", folder_name), options).map_err(|e| e.to_string())?;
        zip.write_all(&modifications_bytes).map_err(|e| e.to_string())?;

        // Write media files
        let media_dir = get_app_data_dir().join("chats").join(chat_id).join("media");
        if media_dir.exists() {
            if let Ok(entries) = fs::read_dir(&media_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();
                        if let Ok(data) = fs::read(&path) {
                            zip.start_file(format!("{}/media/{}", folder_name, filename), options).map_err(|e| e.to_string())?;
                            zip.write_all(&data).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
        }

        // Write custom files
        let custom_dir = get_app_data_dir().join("chats").join(chat_id).join("custom");
        if custom_dir.exists() {
            if let Ok(entries) = fs::read_dir(&custom_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let filename = path.file_name().unwrap().to_string_lossy().to_string();
                        if let Ok(data) = fs::read(&path) {
                            zip.start_file(format!("{}/custom/{}", folder_name, filename), options).map_err(|e| e.to_string())?;
                            zip.write_all(&data).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(save_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn import_from_export(zip_path: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || import_from_export_inner(zip_path)).await.map_err(|e| e.to_string())?
}

/// Android-only helper: the web frontend reads a ZIP file via <input type="file">
/// and sends the raw bytes here (as a Vec<u8>). We write them to a temp cache file
/// and then run the normal import pipeline. This avoids needing the file-dialog plugin
/// which does not support Android.
#[tauri::command]
async fn import_zip_from_bytes(b64: String, filename: String) -> Result<Vec<String>, String> {
    let filename = sanitize_filename(&filename)?;
    let bytes = base64_decode(&b64).map_err(|e| format!("Failed to decode base64: {}", e))?;
    let cache_path = get_app_data_dir().join("import_cache").join(&filename);
    if let Some(parent) = cache_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(&cache_path, &bytes).map_err(|e| format!("Failed to write import cache: {}", e))?;
    let path_str = cache_path.to_string_lossy().to_string();
    let result = tauri::async_runtime::spawn_blocking(move || import_from_export_inner(path_str))
        .await
        .map_err(|e| e.to_string())??;
    let _ = fs::remove_file(&cache_path);
    Ok(result)
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn check_vc_redist() -> bool {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let versions = ["14.0", "15.0", "16.0", "17.0"];
    
    for version in versions {
        if let Ok(key) = hklm.open_subkey(&format!("SOFTWARE\\Microsoft\\VisualStudio\\{}\\VC\\Runtimes\\x64", version)) {
            if let Ok(installed) = key.get_value::<u32, _>("Installed") {
                if installed == 1 {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn install_vc_redist() -> Result<(), String> {
    
    let vc_redist_path = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .parent()
        .map(|p| p.join("vc_redist.x64.exe"))
        .ok_or_else(|| "Could not determine app directory".to_string())?;
    
    if !vc_redist_path.exists() {
        return Err("vc_redist.x64.exe not found. Please download it from Microsoft.".to_string());
    }
    
    std::process::Command::new(&vc_redist_path)
        .spawn()
        .map_err(|e| format!("Failed to start installer: {}", e))?;
    
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn check_vc_redist() -> bool {
    true // Not applicable on non-Windows
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn install_vc_redist() -> Result<(), String> {
    Ok(()) // Not applicable on non-Windows
}

/// Android only: called by the frontend on mount to check if the app was opened
/// via a share intent (e.g. WhatsApp "Export Chat"). Returns the zip path and
/// clears the flag so it only fires once per share.
#[tauri::command]
fn get_pending_share() -> Option<String> {
    None
}

fn import_from_export_inner(zip_path: String) -> Result<Vec<String>, String> {
    IMPORT_MSG_COUNT.store(0, Ordering::Relaxed);
    IMPORT_MEDIA_COUNT.store(0, Ordering::Relaxed);
    IMPORT_ACTIVE.store(1, Ordering::Relaxed);
    IMPORT_PHASE.store(1, Ordering::Relaxed);

    let file = File::open(&zip_path).map_err(|e| format!("Failed to open ZIP: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP: {}", e))?;

    // Detect if this is an export ZIP by checking for meta.json files
    let mut chat_folders: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let name = archive.by_index(i).map_err(|e| e.to_string())?.name().to_string();
        if name.ends_with("/meta.json") {
            let folder = name.trim_end_matches("/meta.json").to_string();
            if !folder.contains('/') {
                chat_folders.push(folder);
            }
        }
    }

    if chat_folders.is_empty() {
        // Not an export ZIP — fall back to regular import
        drop(archive);
        let id = import_chat_inner(zip_path)?;
        IMPORT_ACTIVE.store(0, Ordering::Relaxed);
        return Ok(vec![id]);
    }

    let mut imported_ids: Vec<String> = Vec::new();
    let app_data = get_app_data_dir();
    let mut conn = get_db();
    backfill_epochs(&conn);

    for folder in &chat_folders {
        IMPORT_PHASE.store(1, Ordering::Relaxed); // Extracting
        
        // Read meta.json first to check for duplicates
        let meta_path = format!("{}/meta.json", folder);
        let meta_json: serde_json::Value = {
            let mut file = archive.by_name(&meta_path).map_err(|e| format!("Missing meta.json for {}: {}", folder, e))?;
            let mut buf = String::new();
            file.read_to_string(&mut buf).map_err(|e| e.to_string())?;
            serde_json::from_str(&buf).map_err(|e| e.to_string())?
        };

        let chat_name = meta_json["name"].as_str().unwrap_or(folder).to_string();
        let is_group = meta_json["is_group"].as_bool().unwrap_or(false);

        // Read messages.json
        let messages_path = format!("{}/messages.json", folder);
        let chat_data: ChatData = {
            let mut file = archive.by_name(&messages_path).map_err(|e| format!("Missing messages.json for {}: {}", folder, e))?;
            let mut buf = String::new();
            file.read_to_string(&mut buf).map_err(|e| e.to_string())?;
            serde_json::from_str(&buf).map_err(|e| e.to_string())?
        };

        // Compute last message info
        let (last_content, last_timestamp, new_max_epoch) = {
            let mut best_content = String::new();
            let mut best_ts = String::new();
            let mut best_epoch: i64 = 0;
            for msg in &chat_data.messages {
                let ep = timestamp_to_epoch(&msg.timestamp);
                if ep >= best_epoch {
                    best_epoch = ep;
                    best_ts = msg.timestamp.clone();
                    best_content = msg.content.clone();
                }
            }
            (best_content, best_ts, best_epoch)
        };

        // Check if a chat with same original_name already exists → merge instead of duplicate
        let existing_chat_id: Option<String> = conn.query_row(
            "SELECT id FROM chats WHERE original_name = ?1",
            params![folder],
            |row| row.get(0)
        ).ok();

        if let Some(existing_id) = existing_chat_id {
            // Merge: copy any new media into existing chat dir then merge messages
            let existing_chat_dir = app_data.join("chats").join(&existing_id);
            let media_prefix = format!("{}/media/", folder);
            let custom_prefix = format!("{}/custom/", folder);
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
                let entry_name = entry.name().to_string();
                if entry_name.starts_with(&media_prefix) && entry_name.len() > media_prefix.len() {
                    let filename = &entry_name[media_prefix.len()..];
                    if !filename.contains('/') && !filename.contains('\\') && !filename.is_empty() {
                        let dst = existing_chat_dir.join("media").join(filename);
                        if !dst.exists() {
                            if let Ok(mut out) = File::create(&dst) {
                                IMPORT_MEDIA_COUNT.fetch_add(1, Ordering::Relaxed);
                                let _ = std::io::copy(&mut entry, &mut out);
                            }
                        }
                    }
                } else if entry_name.starts_with(&custom_prefix) && entry_name.len() > custom_prefix.len() {
                    let filename = &entry_name[custom_prefix.len()..];
                    if !filename.contains('/') && !filename.contains('\\') && !filename.is_empty() {
                        let dst = existing_chat_dir.join("custom").join(filename);
                        if !dst.exists() {
                            if let Ok(mut out) = File::create(&dst) {
                                IMPORT_MEDIA_COUNT.fetch_add(1, Ordering::Relaxed);
                                let _ = std::io::copy(&mut entry, &mut out);
                            }
                        }
                    }
                }
            }

            IMPORT_PHASE.store(3, Ordering::Relaxed);
            merge_messages_into_chat(&mut conn, &existing_id, &chat_data.messages)?;

            // Update last message metadata to the newest across all messages
            let merged_max_epoch = {
                let mut stmt = conn.prepare("SELECT timestamp FROM messages WHERE chat_id = ?1")
                    .map_err(|e| e.to_string())?;
                let rows = stmt.query_map(params![&existing_id], |row| row.get::<_, String>(0))
                    .map_err(|e| e.to_string())?;
                let mut max_ep: i64 = 0;
                for row in rows.flatten() {
                    let ep = timestamp_to_epoch(&row);
                    if ep > max_ep { max_ep = ep; }
                }
                max_ep
            };
            let final_epoch = merged_max_epoch.max(new_max_epoch);
            conn.execute(
                "UPDATE chats SET last_message = ?1, timestamp = ?2, last_message_epoch = ?3 WHERE id = ?4",
                params![&last_content, &last_timestamp, final_epoch, &existing_id],
            ).map_err(|e| format!("Failed to update chat metadata: {}", e))?;

            imported_ids.push(existing_id);
            continue;
        }

        // Fresh import — create new chat
        let chat_id = Uuid::new_v4().to_string();
        let chat_dir = app_data.join("chats").join(&chat_id);
        ensure_dir_exists(&chat_dir);
        ensure_dir_exists(&chat_dir.join("media"));
        ensure_dir_exists(&chat_dir.join("custom"));

        // Extract media files
        let media_prefix = format!("{}/media/", folder);
        let custom_prefix = format!("{}/custom/", folder);
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let entry_name = entry.name().to_string();

            if entry_name.starts_with(&media_prefix) && entry_name.len() > media_prefix.len() {
                let filename = &entry_name[media_prefix.len()..];
                if !filename.contains('/') && !filename.contains('\\') && !filename.is_empty() {
                    let dst = chat_dir.join("media").join(filename);
                    let mut out = File::create(&dst).map_err(|e| e.to_string())?;
                    IMPORT_MEDIA_COUNT.fetch_add(1, Ordering::Relaxed);
                    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                }
            } else if entry_name.starts_with(&custom_prefix) && entry_name.len() > custom_prefix.len() {
                let filename = &entry_name[custom_prefix.len()..];
                if !filename.contains('/') && !filename.contains('\\') && !filename.is_empty() {
                    let dst = chat_dir.join("custom").join(filename);
                    let mut out = File::create(&dst).map_err(|e| e.to_string())?;
                    IMPORT_MEDIA_COUNT.fetch_add(1, Ordering::Relaxed);
                    std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
                }
            }
        }

        // Insert chat into DB
        conn.execute(
            "INSERT INTO chats (id, name, original_name, last_message, timestamp, is_group, last_message_epoch, zip_path) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &chat_id,
                &chat_name,
                folder,
                &last_content,
                &last_timestamp,
                is_group as i32,
                new_max_epoch,
                Option::<String>::None
            ],
        ).map_err(|e| format!("Failed to insert chat: {}", e))?;

        // Insert messages
        IMPORT_PHASE.store(3, Ordering::Relaxed); // Saving to DB
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media, duration, tag_ext, display_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            ).map_err(|e| e.to_string())?;
            for msg in &chat_data.messages {
                IMPORT_MSG_COUNT.fetch_add(1, Ordering::Relaxed);
                stmt.execute(params![
                    &chat_id,
                    &msg.timestamp,
                    &msg.sender,
                    &msg.msg_type,
                    &msg.content,
                    msg.media.as_ref().unwrap_or(&String::new()),
                    msg.duration.as_ref().unwrap_or(&String::new()),
                    &msg.tag_ext,
                    &msg.display_name,
                ]).map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;

        // Apply modifications if present
        let mods_path = format!("{}/modifications.json", folder);
        if let Ok(mut file) = archive.by_name(&mods_path) {
            let mut buf = String::new();
            if file.read_to_string(&mut buf).is_ok() && buf != "[]" {
                let _ = apply_chat_modifications_internal(&mut conn, &chat_id, &buf);
            }
        }

        imported_ids.push(chat_id);
        IMPORT_PHASE.store(4, Ordering::Relaxed); // Done this chat
    }

    IMPORT_ACTIVE.store(0, Ordering::Relaxed);
    IMPORT_PHASE.store(0, Ordering::Relaxed);

    Ok(imported_ids)
}

fn apply_chat_modifications_internal(conn: &mut Connection, chat_id: &str, modifications_json: &str) -> Result<(), String> {
    let modifications: Vec<serde_json::Value> = serde_json::from_str(modifications_json).map_err(|e| e.to_string())?;

    for modification in &modifications {
        let mod_type = modification["type"].as_str().unwrap_or("");
        match mod_type {
            "message" => {
                let msg_idx = modification["message_index"].as_i64().unwrap_or(-1);
                if msg_idx < 0 { continue; }

                // Get message ID by index
                let msg_id: Option<i64> = conn.prepare(
                    "SELECT id FROM messages WHERE chat_id = ?1 ORDER BY id ASC LIMIT 1 OFFSET ?2"
                ).ok().and_then(|mut stmt| {
                    stmt.query_row(params![chat_id, msg_idx], |row| row.get(0)).ok()
                });

                if let Some(id) = msg_id {
                    if let Some(dn) = modification["display_name"].as_str() {
                        let _ = conn.execute(
                            "UPDATE messages SET display_name = ?1, display_name_modified = 1 WHERE id = ?2",
                            params![dn, id]
                        );
                    }
                    if let Some(te) = modification["tag_ext"].as_str() {
                        let _ = conn.execute(
                            "UPDATE messages SET tag_ext = ?1, tag_ext_modified = 1 WHERE id = ?2",
                            params![te, id]
                        );
                    }
                    if let Some(mt) = modification["msg_type"].as_str() {
                        let _ = conn.execute(
                            "UPDATE messages SET msg_type = ?1, msg_type_modified = 1 WHERE id = ?2",
                            params![mt, id]
                        );
                    }
                }
            }
            "profile" => {
                let name = modification["name"].as_str();
                let notes = modification["notes"].as_str();
                let photo = modification["photo_path"].as_str();
                let phone = modification["phone_number"].as_str();

                // Remap background_path: only keep the filename, resolve to new chat's custom/ dir
                let raw_bg = modification["background_path"].as_str();
                let remapped_bg: Option<String> = raw_bg.and_then(|p| {
                    let fname = std::path::Path::new(p).file_name()?.to_string_lossy().to_string();
                    let new_path = get_app_data_dir().join("chats").join(chat_id).join("custom").join(&fname);
                    if new_path.exists() { Some(new_path.to_string_lossy().to_string()) } else { None }
                });

                conn.execute(
                    "INSERT OR REPLACE INTO profiles (chat_id, name, notes, photo_path, phone_number, background_path, profile_modified) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1)",
                    params![chat_id, name, notes, photo, phone, remapped_bg]
                ).map_err(|e| e.to_string())?;

                // Record background in history if present
                if let Some(ref bg) = remapped_bg {
                    let _ = conn.execute(
                        "INSERT INTO chat_background_history (chat_id, background_path) VALUES (?1, ?2)",
                        params![chat_id, bg]
                    );
                }
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]

pub fn run() {
    // Single-instance enforcement: kill any existing running instance
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let current_pid = std::process::id();
        // Use tasklist to find other instances of the same exe
        if let Ok(output) = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq whatsapp-archive-viewer-pc.exe", "/FO", "CSV", "/NH"])
            .output()
        {
            if let Ok(s) = String::from_utf8(output.stdout) {
                for line in s.lines() {
                    // CSV format: "name","pid","session","#","mem"
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        let pid_str = parts[1].trim().trim_matches('"');
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            if pid != current_pid {
                                let _ = Command::new("taskkill")
                                    .args(["/PID", &pid.to_string(), "/F"])
                                    .output();
                            }
                        }
                    }
                }
            }
        }
    }

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    builder = builder.register_uri_scheme_protocol("media", |_app, request| {

            let uri = request.uri().to_string();

            // URI format varies by platform/Tauri version:

            // - media://localhost/chat_id/filename  (standard)

            // - http://media.localhost/chat_id/filename  (Windows WebView2)

            // - https://media.localhost/chat_id/filename  (Windows WebView2 secure)

            // - media://chat_id/filename  (some builds)

            let path_str = uri

                .strip_prefix("http://media.localhost/")

                .or_else(|| uri.strip_prefix("https://media.localhost/"))

                .or_else(|| uri.strip_prefix("media://localhost/"))

                .or_else(|| uri.strip_prefix("media://"))

                .unwrap_or("");

            let decoded = percent_decode_str(path_str).decode_utf8_lossy().to_string();

            let parts: Vec<&str> = decoded.splitn(2, '/').collect();

            if parts.len() < 2 {

                return tauri::http::Response::builder()

                    .status(404)

                    .body(b"Not found".to_vec())

                    .unwrap();

            }

            let chat_id = parts[0];

            let filename = parts[1];

            let app_data = get_app_data_dir();

            

            // Try multiple possible locations for media files

            let media_dir = app_data.join("chats").join(chat_id).join("media");

            let direct_path = media_dir.join(filename);

            

            // Try direct path first, then search recursively in subdirectories

            let file_data = if let Ok(data) = std::fs::read(&direct_path) {

                Some(data)

            } else {

                // Fallback: search recursively in media folder (for existing imports with subdirs)

                fn find_file_recursive(dir: &Path, target: &str) -> Option<PathBuf> {

                    if let Ok(entries) = std::fs::read_dir(dir) {

                        for entry in entries.flatten() {

                            let path = entry.path();

                            if path.is_file() && path.file_name()?.to_str()? == target {

                                return Some(path);

                            }

                            if path.is_dir() {

                                if let Some(found) = find_file_recursive(&path, target) {

                                    return Some(found);

                                }

                            }

                        }

                    }

                    None

                }

                let found = find_file_recursive(&media_dir, filename);

                found.and_then(|p| std::fs::read(p).ok())

            };

            

            match file_data {

                Some(data) => {

                    let mime = match Path::new(filename).extension().and_then(|e| e.to_str()).unwrap_or("") {

                        "jpg" | "jpeg" => "image/jpeg",

                        "png" => "image/png",

                        "gif" => "image/gif",

                        "webp" => "image/webp",

                        "bmp" => "image/bmp",

                        "mp4" => "video/mp4",

                        "mov" => "video/quicktime",

                        "avi" => "video/x-msvideo",

                        "mkv" => "video/x-matroska",

                        "webm" => "video/webm",

                        "mp3" => "audio/mpeg",

                        "ogg" => "audio/ogg",

                        "wav" => "audio/wav",

                        "m4a" => "audio/mp4",

                        "aac" => "audio/aac",

                        "opus" => "audio/opus",

                        "pdf" => "application/pdf",

                        _ => "application/octet-stream",

                    };

                    tauri::http::Response::builder()

                        .status(200)

                        .header("Content-Type", mime)

                        .header("Access-Control-Allow-Origin", "*")

                        .body(data)

                        .unwrap()

                }

                None => {

                    tauri::http::Response::builder()

                        .status(404)

                        .body(b"File not found".to_vec())

                        .unwrap()

                }

            }
        });

    builder

        .plugin(tauri_plugin_opener::init())

        .plugin(tauri_plugin_dialog::init())

        .invoke_handler(tauri::generate_handler![

            check_file_exists,

            pick_zip_file,

            pick_zip_files,

            import_chat,

            import_chats_batch,

            get_chat_list,

            get_chat_messages,

            get_chat_message_count,

            get_media_base_dir,

            get_media_as_base64,

            get_media_with_dims,

            preload_media,

            get_media_path,

            delete_chat,

            clear_all_chats,

            migrate_chats,

            search_messages,

            search_messages_filtered,

            search_chats,

            get_profile,

            update_profile,

            remove_profile_photo,

            get_name_history,

            revert_profile_name,

            pick_profile_photo,

            pick_global_background,
            save_background_from_b64,
            read_file_as_base64,

            list_default_backgrounds,

            debug_chat_media,

            set_message_type,

            open_media_file,

            open_url,

            open_vcard_whatsapp,

            parse_vcard,

            check_file_in_zip,

            extract_file_from_zip,

            set_file_tag,

            set_display_name,

            rename_media_file,

            export_chat_modifications,

            apply_chat_modifications,

            clear_modification_flags,

            export_chat_zip,

            export_all_chats_zip,

            import_from_export,

            import_zip_from_bytes,

            get_import_progress,

            set_chat_background,

            get_background_history,

            get_chat_background,

            restore_chat_background,

            clear_chat_background,

            toggle_message_favorite,

            get_favorite_messages,

            check_vc_redist,

            install_vc_redist,

            get_pending_share

        ])

        .run(tauri::generate_context!())

        .expect("error while running tauri application");

}



// =============================================================================

// TESTS

//

// IMPORTANT FOR AI ASSISTANTS — READ BEFORE TOUCHING ANY CODE:

//   These tests document the exact behavior this application relies on.

//   Before changing ANY function covered by a test you MUST:

//     1. Read every test whose name mentions that function.

//     2. Ask the owner (Ramon) for explicit confirmation that the change is safe.

//     3. Update the test(s) to reflect the new behavior AFTER getting approval.

//   Run tests with: cargo test --manifest-path whatsapp-archive-viewer-pc/project-code/src-tauri/Cargo.toml

// =============================================================================

#[cfg(test)]

mod tests {

    use super::*;



    // ------------------------------------------------------------------

    // Helper: parse a chat string through an empty temp import dir

    // ------------------------------------------------------------------

    fn parse(content: &str) -> Vec<Message> {

        let tmp = std::env::temp_dir().join(format!("wa_test_{}", std::time::SystemTime::now()

            .duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));

        std::fs::create_dir_all(&tmp).unwrap();

        let result = parse_chat_text(content, &tmp, &tmp).unwrap();

        let _ = std::fs::remove_dir_all(&tmp);

        result

    }



    // ------------------------------------------------------------------

    // Helper: in-memory SQLite connection for merge/dedup tests

    // ------------------------------------------------------------------

    fn in_memory_db_with_chat(chat_id: &str) -> rusqlite::Connection {

        let conn = rusqlite::Connection::open_in_memory().unwrap();

        conn.execute_batch("

            CREATE TABLE chats (

                id TEXT PRIMARY KEY, name TEXT NOT NULL,

                last_message TEXT, timestamp TEXT,

                is_group INTEGER NOT NULL DEFAULT 0,

                zip_path TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

                last_message_epoch INTEGER NOT NULL DEFAULT 0,

                original_name TEXT

            );

            CREATE TABLE messages (

                id INTEGER PRIMARY KEY AUTOINCREMENT,

                chat_id TEXT NOT NULL,

                timestamp TEXT NOT NULL,

                sender TEXT NOT NULL,

                msg_type TEXT NOT NULL DEFAULT 'text',

                content TEXT NOT NULL,

                media TEXT,

                duration TEXT,

                tag_ext TEXT,

                display_name TEXT

            );

        ").unwrap();

        conn.execute(

            "INSERT INTO chats (id, name, original_name) VALUES (?1, 'Test', 'Test')",

            rusqlite::params![chat_id],

        ).unwrap();

        conn

    }



    // ==================== timestamp_to_epoch ====================



    #[test]

    fn timestamp_to_epoch__android_slash_format_dd_mm_yyyy_returns_correct_unix_seconds() {

        // 01/01/1970 00:00 → epoch 0

        assert_eq!(timestamp_to_epoch("01/01/1970 00:00"), 0);

    }



    #[test]

    fn timestamp_to_epoch__android_slash_format_known_date_returns_correct_unix_seconds() {

        // 17/06/2017 00:04 → should be a positive epoch value consistent across runs

        let epoch = timestamp_to_epoch("17/06/2017 00:04");

        assert!(epoch > 0, "expected positive epoch for 17/06/2017 00:04");

        // 2017-06-17 00:04 UTC = 1497657840

        assert_eq!(epoch, 1497657840);

    }



    #[test]

    fn timestamp_to_epoch__android_dash_format_dd_mm_yyyy_returns_correct_unix_seconds() {

        let epoch = timestamp_to_epoch("17-06-2017 00:04");

        assert_eq!(epoch, 1497657840);

    }



    #[test]

    fn timestamp_to_epoch__android_dotted_format_dd_mm_yyyy_returns_correct_unix_seconds() {

        let epoch = timestamp_to_epoch("17.06.2017 00:04");

        assert_eq!(epoch, 1497657840);

    }



    #[test]

    fn timestamp_to_epoch__two_digit_year_is_interpreted_as_2000_plus() {

        // 01/01/24 00:00 → same as 01/01/2024 00:00

        let two_digit = timestamp_to_epoch("01/01/24 00:00");

        let four_digit = timestamp_to_epoch("01/01/2024 00:00");

        assert_eq!(two_digit, four_digit);

    }



    #[test]

    fn timestamp_to_epoch__unparseable_string_returns_zero() {

        assert_eq!(timestamp_to_epoch("not a date"), 0);

        assert_eq!(timestamp_to_epoch(""), 0);

    }



    // ==================== normalize_chat_name ====================



    #[test]

    fn normalize_chat_name__strips_group_suffix_and_lowercases() {

        assert_eq!(normalize_chat_name("My Friends (Group)"), "my friends");

    }



    #[test]

    fn normalize_chat_name__plain_name_is_lowercased_only() {

        assert_eq!(normalize_chat_name("Ramon"), "ramon");

    }



    #[test]

    fn normalize_chat_name__trims_leading_and_trailing_whitespace() {

        assert_eq!(normalize_chat_name("  Chat Name  "), "chat name");

    }



    // ==================== detect_group_chat ====================



    #[test]

    fn detect_group_chat__three_distinct_non_system_senders_returns_true() {

        let msgs = vec![

            Message { sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

            Message { sender: "Bob".into(),   msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

            Message { sender: "Charlie".into(),msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

        ];

        assert!(detect_group_chat(&msgs));

    }



    #[test]

    fn detect_group_chat__two_senders_returns_false_because_it_is_a_private_chat() {

        let msgs = vec![

            Message { sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

            Message { sender: "Bob".into(),   msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

        ];

        assert!(!detect_group_chat(&msgs));

    }



    #[test]

    fn detect_group_chat__system_and_you_senders_are_excluded_from_count() {

        let msgs = vec![

            Message { sender: "System".into(), msg_type: "system".into(), content: "end-to-end".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

            Message { sender: "You".into(),    msg_type: "text".into(),   content: "hi".into(),         timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

            Message { sender: "Alice".into(),  msg_type: "text".into(),   content: "hi".into(),         timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },

        ];

        assert!(!detect_group_chat(&msgs));

    }



    // ==================== parse_chat_text — message types ====================



    #[test]

    fn parse_chat_text__android_slash_format_plain_text_message_is_parsed_as_type_text() {

        let chat = "12/04/2024, 14:32 - Alice: Hello World\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 1);

        assert_eq!(msgs[0].sender, "Alice");

        assert_eq!(msgs[0].content, "Hello World");

        assert_eq!(msgs[0].msg_type, "text");

        assert!(msgs[0].media.is_none());

    }



    #[test]

    fn parse_chat_text__ios_bracket_format_plain_text_message_is_parsed_correctly() {

        let chat = "[12/04/2024, 14:32] Alice: Hello World\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 1);

        assert_eq!(msgs[0].sender, "Alice");

        assert_eq!(msgs[0].msg_type, "text");

    }



    #[test]

    fn parse_chat_text__android_dash_format_no_comma_is_parsed_correctly() {

        let chat = "17-06-2017 00:04 - Alice: Hello\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 1);

        assert_eq!(msgs[0].sender, "Alice");

        assert_eq!(msgs[0].msg_type, "text");

    }



    #[test]

    fn parse_chat_text__jpg_attachment_with_file_attached_marker_is_typed_as_image() {

        let chat = "12/04/2024, 14:32 - Alice: IMG-20240412-WA0001.jpg (file attached)\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "image");

        assert_eq!(msgs[0].media.as_deref(), Some("IMG-20240412-WA0001.jpg"));

    }



    #[test]

    fn parse_chat_text__jpg_attachment_with_dutch_marker_bestand_bijgevoegd_is_typed_as_image() {

        let chat = "12/04/2024, 14:32 - Alice: IMG-20240412-WA0001.jpg (bestand bijgevoegd)\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "image");

        assert_eq!(msgs[0].media.as_deref(), Some("IMG-20240412-WA0001.jpg"));

    }



    #[test]

    fn parse_chat_text__webp_non_sticker_with_media_omitted_is_typed_as_image_not_sticker() {

        let chat = "12/04/2024, 14:32 - Alice: IMG-20240412-WA0001.webp <Media omitted>\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "image");

        assert!(!msgs[0].media.as_deref().unwrap_or("").starts_with("STK-"));

    }



    #[test]

    fn parse_chat_text__stk_prefix_webp_with_file_attached_marker_is_typed_as_sticker() {

        let chat = "12/04/2024, 14:32 - Alice: STK-20240412-WA0001.webp (file attached)\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "sticker");

        assert_eq!(msgs[0].media.as_deref(), Some("STK-20240412-WA0001.webp"));

    }



    #[test]

    fn parse_chat_text__stk_prefix_webp_with_media_omitted_marker_is_typed_as_sticker() {

        let chat = "12/04/2024, 14:32 - Alice: STK-20240412-WA0001.webp <Media omitted>\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "sticker");

    }



    #[test]

    fn parse_chat_text__stk_prefix_webp_bare_filename_no_attachment_marker_is_typed_as_sticker() {

        // WhatsApp sometimes exports stickers as bare filename with no "(file attached)" suffix

        let chat = "12/04/2024, 14:32 - Alice: STK-20240412-WA0001.webp\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "sticker");

        assert_eq!(msgs[0].media.as_deref(), Some("STK-20240412-WA0001.webp"));

    }



    #[test]

    fn parse_chat_text__mp4_attachment_is_typed_as_video_when_no_size_info_available() {

        let chat = "12/04/2024, 14:32 - Alice: VID-20240412-WA0001.mp4 (file attached)\n";

        let msgs = parse(chat);

        // Without a real file on disk the size fallback is u64::MAX → "video"

        assert_eq!(msgs[0].msg_type, "video");

    }



    #[test]

    fn parse_chat_text__opus_attachment_is_typed_as_audio() {

        let chat = "12/04/2024, 14:32 - Alice: AUD-20240412-WA0001.opus (file attached)\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "audio");

    }



    #[test]

    fn parse_chat_text__pdf_attachment_is_typed_as_file() {

        let chat = "12/04/2024, 14:32 - Alice: document.pdf (file attached)\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "file");

    }



    #[test]

    fn parse_chat_text__google_maps_url_in_text_is_typed_as_location() {

        let chat = "12/04/2024, 14:32 - Alice: Location: https://maps.google.com/?q=52.3,4.9\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "location");

        assert!(msgs[0].media.as_deref().unwrap_or("").contains("maps.google.com"));

    }



    #[test]

    fn parse_chat_text__system_message_without_sender_colon_is_typed_as_system() {

        let chat = "12/04/2024, 14:32 - Messages and calls are end-to-end encrypted.\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "system");

        assert_eq!(msgs[0].sender, "System");

    }



    #[test]

    fn parse_chat_text__multiline_message_continuation_is_appended_to_previous_message() {

        let chat = "12/04/2024, 14:32 - Alice: First line\nSecond line\nThird line\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 1);

        assert!(msgs[0].content.contains("Second line"));

        assert!(msgs[0].content.contains("Third line"));

    }



    #[test]

    fn parse_chat_text__bom_prefix_in_content_is_stripped_from_media_filename() {

        // WhatsApp embeds U+200E (LRM) and similar marks in filenames

        let lrm = '\u{200E}';

        let chat = format!("12/04/2024, 14:32 - Alice: {}STK-20240412-WA0001.webp (file attached)\n", lrm);

        let msgs = parse(&chat);

        assert_eq!(msgs[0].msg_type, "sticker");

        // Media should NOT start with the LRM character

        let media = msgs[0].media.as_deref().unwrap_or("");

        assert!(!media.starts_with(lrm), "LRM mark not stripped from media filename");

    }



    #[test]

    fn parse_chat_text__two_sequential_messages_are_both_captured() {

        let chat = "12/04/2024, 14:32 - Alice: Hello\n12/04/2024, 14:33 - Bob: Hi\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 2);

        assert_eq!(msgs[0].sender, "Alice");

        assert_eq!(msgs[1].sender, "Bob");

    }



    #[test]

    fn parse_chat_text__dutch_media_weggelaten_marker_extracts_filename_and_types_correctly() {

        let chat = "12/04/2024, 14:32 - Alice: IMG-20240412-WA0001.jpg <Media weggelaten>\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "image");

        assert_eq!(msgs[0].media.as_deref(), Some("IMG-20240412-WA0001.jpg"));

    }



    // ==================== merge_messages_into_chat (dedup) ====================



    #[test]

    fn merge_messages_into_chat__duplicate_timestamp_and_sender_pair_is_skipped_not_inserted() {

        let chat_id = "test-chat-1";

        let mut conn = in_memory_db_with_chat(chat_id);



        let existing = Message {

            timestamp: "12/04/2024 14:32".into(),

            sender: "Alice".into(),

            msg_type: "text".into(),

            content: "Hello".into(),

            media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None,

        };

        conn.execute(

            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media, duration, tag_ext, display_name) VALUES (?1,?2,?3,?4,?5,'','',NULL,NULL)",

            rusqlite::params![chat_id, existing.timestamp, existing.sender, existing.msg_type, existing.content],

        ).unwrap();



        let duplicate = existing.clone();

        let inserted = merge_messages_into_chat(&mut conn, chat_id, &[duplicate]).unwrap();

        assert_eq!(inserted, 0, "duplicate message must not be inserted again");

    }



    #[test]

    fn merge_messages_into_chat__new_message_with_different_timestamp_is_inserted() {

        let chat_id = "test-chat-2";

        let mut conn = in_memory_db_with_chat(chat_id);



        let new_msg = Message {

            timestamp: "12/04/2024 15:00".into(),

            sender: "Alice".into(),

            msg_type: "text".into(),

            content: "New message".into(),

            media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None,

        };

        let inserted = merge_messages_into_chat(&mut conn, chat_id, &[new_msg]).unwrap();

        assert_eq!(inserted, 1, "new message must be inserted");

    }



    #[test]

    fn merge_messages_into_chat__existing_message_content_is_preserved_not_overwritten() {

        let chat_id = "test-chat-3";

        let mut conn = in_memory_db_with_chat(chat_id);



        conn.execute(

            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content, media, duration, tag_ext, display_name) VALUES (?1,'ts','Alice','text','Original content','','',NULL,NULL)",

            rusqlite::params![chat_id],

        ).unwrap();



        let reimport = Message {

            timestamp: "ts".into(),

            sender: "Alice".into(),

            msg_type: "text".into(),

            content: "Changed content".into(),

            media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None,

        };

        merge_messages_into_chat(&mut conn, chat_id, &[reimport]).unwrap();



        let stored: String = conn.query_row(

            "SELECT content FROM messages WHERE chat_id = ?1",

            rusqlite::params![chat_id],

            |row| row.get(0),

        ).unwrap();

        assert_eq!(stored, "Original content", "manual edits must not be overwritten on re-import");

    }



    #[test]

    fn parse_chat_text__wa_format_example_after_fence_close_blank_line_not_split() {

        // After a fence close, a blank line appears before an embedded WA-format example.
        // Without the two-flag guard, the blank line used to silently clear the guard,
        // causing the example line to be accepted as a new message.
        let chat = concat!(
            "12/04/2024, 10:00 - Alice: Here's the format:\n",
            "```\n",
            "\n",
            "### Input Format Example:\n",
            "```\n",
            "\n",
            "[12/04/2024, 14:32] John: Hello\n",
            "```\n",
            "\n",
            "### Tasks:\n",
            "```\n",
            "12/04/2024, 14:33 - Bob: Got it\n",
        );

        let msgs = parse(chat);

        assert!(
            msgs.iter().all(|m| m.sender != "John"),
            "Example WA-format line was incorrectly split into its own message"
        );

        assert_eq!(msgs.len(), 2, "Should be exactly Alice + Bob");

        assert!(msgs[0].content.contains("John: Hello"), "Example line should be in Alice's content");

    }



    // ==================== is_plausible_whatsapp_date ====================



    #[test]

    fn is_plausible_whatsapp_date__valid_slash_date_and_time_returns_true() {

        assert!(is_plausible_whatsapp_date("12/04/2024", "14:32"));

    }



    #[test]

    fn is_plausible_whatsapp_date__valid_dash_date_returns_true() {

        assert!(is_plausible_whatsapp_date("17-06-2017", "00:04"));

    }



    #[test]

    fn is_plausible_whatsapp_date__valid_dot_date_returns_true() {

        assert!(is_plausible_whatsapp_date("17.06.2017", "00:04"));

    }



    #[test]

    fn is_plausible_whatsapp_date__both_fields_out_of_range_returns_false() {

        // day=99, month=99 — neither field is plausible
        assert!(!is_plausible_whatsapp_date("99/99/2024", "14:32"));

    }



    #[test]

    fn is_plausible_whatsapp_date__invalid_hour_returns_false() {

        assert!(!is_plausible_whatsapp_date("12/04/2024", "25:00"));

    }



    #[test]

    fn is_plausible_whatsapp_date__year_before_whatsapp_era_returns_false() {

        assert!(!is_plausible_whatsapp_date("12/04/2008", "14:32"));

    }



    // ==================== extract_maps_url ====================



    #[test]

    fn extract_maps_url__google_maps_link_is_extracted() {

        let url = "https://maps.google.com/?q=52.3,4.9";

        assert_eq!(extract_maps_url(url), Some(url.to_string()));

    }



    #[test]

    fn extract_maps_url__apple_maps_link_is_extracted() {

        let url = "https://maps.apple.com/?ll=52.3,4.9";

        assert_eq!(extract_maps_url(url), Some(url.to_string()));

    }



    #[test]

    fn extract_maps_url__goo_gl_maps_link_is_extracted() {

        let result = extract_maps_url("https://goo.gl/maps/abc123");

        assert!(result.is_some());

        assert!(result.unwrap().contains("goo.gl/maps"));

    }



    #[test]

    fn extract_maps_url__non_maps_url_returns_none() {

        assert_eq!(extract_maps_url("https://www.example.com"), None);

    }



    #[test]

    fn extract_maps_url__plain_text_returns_none() {

        assert_eq!(extract_maps_url("Hello World"), None);

    }



    // ==================== parse_chat_text — additional types ====================



    #[test]

    fn parse_chat_text__gif_extension_attachment_is_typed_as_image() {

        // .gif files are categorised as "image"; only small .mp4 files get type "gif"
        let chat = "12/04/2024, 14:32 - Alice: GIF-20240412-WA0001.gif (file attached)\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "image");

        assert_eq!(msgs[0].media.as_deref(), Some("GIF-20240412-WA0001.gif"));

    }



    #[test]

    fn parse_chat_text__ios_bracket_format_with_12h_time_is_parsed_correctly() {

        let chat = "[12/04/2024, 2:32:00 PM] Alice: Hello\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 1);

        assert_eq!(msgs[0].sender, "Alice");

        assert_eq!(msgs[0].msg_type, "text");

    }



    #[test]

    fn parse_chat_text__apple_maps_url_in_text_is_typed_as_location() {

        let chat = "12/04/2024, 14:32 - Alice: Location: https://maps.apple.com/?ll=52.3,4.9\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].msg_type, "location");

        assert!(msgs[0].media.as_deref().unwrap_or("").contains("maps.apple.com"));

    }



    #[test]

    fn parse_chat_text__parsed_messages_have_is_favorite_none_by_default() {

        let chat = "12/04/2024, 14:32 - Alice: Hello\n";

        let msgs = parse(chat);

        assert_eq!(msgs[0].is_favorite, None);

    }



    #[test]

    fn parse_chat_text__empty_input_returns_no_messages() {

        let msgs = parse("");

        assert_eq!(msgs.len(), 0);

    }



    #[test]

    fn parse_chat_text__dotted_date_format_plain_text_is_parsed_correctly() {

        // Dotted format uses no comma: "dd.mm.yyyy HH:MM - Sender: msg"
        let chat = "17.06.2017 00:04 - Alice: Hello dot format\n";

        let msgs = parse(chat);

        assert_eq!(msgs.len(), 1);

        assert_eq!(msgs[0].sender, "Alice");

        assert_eq!(msgs[0].msg_type, "text");

    }

}


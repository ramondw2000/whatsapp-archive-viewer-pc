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
static PHONE_LIKE_RE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_EN_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_EN_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_EN_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_EN_WAS_ADDED: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_EN_WAS_REMOVED: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FR_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FR_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FR_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FR_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FR_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
// The following are all best-effort, sourced from a decompiled WhatsApp APK's
// values-XX/strings.xml (https://github.com/GigaDroid/Decompiled-Whatsapp), not verified
// against real exports. See extract_membership_events doc comment for details.
static MEMBERSHIP_RE_AZ_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_AZ_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_AZ_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_AZ_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_AZ_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CA_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CA_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CA_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CA_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CA_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CS_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CS_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CS_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CS_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_CS_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DA_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DA_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DA_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DA_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DA_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DE_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DE_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DE_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DE_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_DE_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ES_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ES_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ES_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ES_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ES_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ET_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ET_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ET_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ET_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ET_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FI_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FI_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FI_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FI_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_FI_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HR_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HR_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HR_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HR_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HR_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HU_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HU_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HU_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HU_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_HU_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ID_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ID_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ID_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ID_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_ID_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_IT_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_IT_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_IT_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_IT_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_IT_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LT_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LT_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LT_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LT_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LT_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LV_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LV_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LV_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LV_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_LV_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_MS_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_MS_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_MS_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_MS_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_MS_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_NB_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_NB_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_NB_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_NB_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_NB_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PL_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PL_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PL_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PL_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PL_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PT_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PT_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PT_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PT_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PT_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PTBR_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PTBR_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PTBR_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PTBR_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_PTBR_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_RO_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_RO_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_RO_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_RO_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_RO_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SK_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SK_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SK_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SK_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SK_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SL_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SL_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SL_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SL_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SL_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SQ_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SQ_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SQ_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SQ_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SQ_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SV_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SV_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SV_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SV_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SV_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SW_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SW_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SW_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SW_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SW_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TL_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TL_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TL_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TL_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TL_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TR_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TR_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TR_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TR_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_TR_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_UZ_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_UZ_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_UZ_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_UZ_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_UZ_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_VI_SELF_LEFT: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_VI_THIRD_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_VI_THIRD_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_VI_YOU_ADD: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_VI_YOU_REMOVE: OnceLock<regex::Regex> = OnceLock::new();
static MEMBERSHIP_RE_SPLIT_TARGETS: OnceLock<regex::Regex> = OnceLock::new();
// Auto-links established by reconcile_contacts_and_chats since the frontend last drained them
// (via take_pending_auto_links). Reconciliation runs from places with no direct request/response
// path back to a listening frontend (startup, and deep inside spawn_blocking after import), so
// results are queued here instead of returned directly.
static PENDING_AUTO_LINKS: OnceLock<Mutex<Vec<AutoLinkEvent>>> = OnceLock::new();

fn queue_auto_link_events(events: Vec<AutoLinkEvent>) {
    if events.is_empty() { return; }
    let mutex = PENDING_AUTO_LINKS.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut pending) = mutex.lock() {
        pending.extend(events);
    }
}

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
use rusqlite::{Connection, Result as SqliteResult, params, OptionalExtension};
use percent_encoding::percent_decode_str;
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
        // Images
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp" | "tiff" | "tif" | "avif" |
        // Videos
        "mp4" | "mkv" | "mov" | "avi" | "webm" | "m4v" | "3gp" |
        // Audio
        "mp3" | "m4a" | "aac" | "ogg" | "opus" | "flac" | "wav" |
        // Documents
        "pdf" | "vcf" | "ico" | "txt" | "csv" | "json" | "xml" | "html" | "htm" |
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" |
        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" |
        // Other common
        "rtf" | "md" | "log"
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
    f.read_exact(&mut buf).ok()?;
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
    // Generic key-value table for single-value local app settings (e.g. the PIN lock hash)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (key TEXT PRIMARY KEY, value TEXT)",
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
    // Create contacts table — a shared identity independent of any one chat, used to link
    // group participants to a 1-on-1 chat's profile (and to each other across groups).
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contacts (
            id TEXT PRIMARY KEY,
            normalized_key TEXT NOT NULL UNIQUE,
            display_key TEXT NOT NULL,
            name TEXT,
            notes TEXT,
            photo_path TEXT,
            phone_number TEXT
        )",
        [],
    )?;
    // Add contact_id column to chats if it doesn't exist yet (links a 1-on-1 chat to a contact)
    let _ = conn.execute(
        "ALTER TABLE chats ADD COLUMN contact_id TEXT REFERENCES contacts(id)",
        [],
    );
    // Create contact_groups table — tracks which group chats a contact has been resolved in.
    // `excluded` is a soft-remove flag set by the "Remove group" action in the profile dialog;
    // it's only cleared again by explicitly re-editing that participant from that same group.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS contact_groups (
            contact_id TEXT NOT NULL REFERENCES contacts(id),
            chat_id TEXT NOT NULL REFERENCES chats(id),
            excluded INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (contact_id, chat_id)
        )",
        [],
    )?;
    // Create file renames table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chat_file_renames (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            chat_id TEXT NOT NULL,
            message_index INTEGER NOT NULL,
            original_filename TEXT NOT NULL,
            new_filename TEXT NOT NULL,
            changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (chat_id) REFERENCES chats(id) ON DELETE CASCADE
        )",
        [],
    )?;
    reclassify_misdetected_groups(&conn);
    queue_auto_link_events(reconcile_contacts_and_chats(&conn));
    Ok(conn)
}

// One-time correction for chats mis-detected as groups by an earlier, less accurate version
// of detect_group_chat (which treated any 2 distinct senders as a group — see the comment on
// that function for why that's wrong). Re-runs the corrected detection against already-stored
// messages and fixes both `is_group` and the "(Group)" suffix baked into the name at import.
fn reclassify_misdetected_groups(conn: &Connection) {
    let group_chats: Vec<(String, String)> = {
        let mut stmt = match conn.prepare("SELECT id, name FROM chats WHERE is_group = 1") {
            Ok(s) => s,
            Err(_) => return,
        };
        let rows = match stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
            Ok(rows) => rows,
            Err(_) => return,
        };
        rows.flatten().collect()
    };
    for (chat_id, name) in group_chats {
        let mut stmt = match conn.prepare(
            "SELECT sender, msg_type, content FROM messages WHERE chat_id = ?1"
        ) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let rows = match stmt.query_map(params![&chat_id], |row| {
            Ok(Message {
                id: None,
                timestamp: String::new(),
                sender: row.get(0)?,
                msg_type: row.get(1)?,
                content: row.get(2)?,
                media: None,
                duration: None,
                tag_ext: None,
                display_name: None,
                is_favorite: None,
            })
        }) {
            Ok(rows) => rows,
            Err(_) => continue,
        };
        let messages: Vec<Message> = rows.flatten().collect();
        if !detect_group_chat(&messages) {
            let corrected_name = name.strip_suffix(" (Group)").unwrap_or(&name).to_string();
            let _ = conn.execute(
                "UPDATE chats SET is_group = 0, name = ?1 WHERE id = ?2",
                params![&corrected_name, &chat_id],
            );
        }
    }
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
        link_chat_to_contact_if_match(&conn, &existing_id, is_group);
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
    link_chat_to_contact_if_match(&conn, &chat_id, is_group);
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

fn iso_date_to_epoch(date_str: &str, end_of_day: bool) -> Option<i64> {
    // Parses "YYYY-MM-DD" (as produced by <input type="date">) into seconds since Unix epoch (UTC).
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 { return None; }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;
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
    Some(days * 86400 + if end_of_day { 86399 } else { 0 })
}

fn find_message_index_for_date_impl(conn: &Connection, chat_id: &str, target_epoch: i64) -> Result<Option<i64>, String> {
    let mut stmt = conn.prepare(
        "SELECT timestamp FROM messages WHERE chat_id = ?1 ORDER BY id ASC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([chat_id], |row| row.get::<_, String>(0)).map_err(|e| e.to_string())?;
    for (idx, row) in rows.enumerate() {
        let ts = row.map_err(|e| e.to_string())?;
        if timestamp_to_epoch(&ts) >= target_epoch {
            return Ok(Some(idx as i64));
        }
    }
    Ok(None)
}

#[tauri::command]
fn find_message_index_for_date(chat_id: String, date: String) -> Result<Option<i64>, String> {
    let conn = get_db();
    let target_epoch = iso_date_to_epoch(&date, false)
        .ok_or_else(|| "Invalid date format".to_string())?;
    find_message_index_for_date_impl(&conn, &chat_id, target_epoch)
}

fn normalize_chat_name(name: &str) -> String {
    name.trim()
        .trim_end_matches(" (Group)")
        .to_lowercase()
}

fn looks_like_phone_number(s: &str) -> bool {
    let stripped: String = s.chars().filter(|c| !matches!(c, ' ' | '-' | '(' | ')')).collect();
    let re = PHONE_LIKE_RE.get_or_init(|| regex::Regex::new(r"^\+?\d{7,15}$").unwrap());
    re.is_match(&stripped)
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
    // An explicit system message about group creation/membership is the only fully reliable
    // signal here. Sender count alone can't distinguish a group from an ordinary 1-on-1
    // conversation: real WhatsApp exports never actually label the account owner "You" (that
    // filter above is a defensive no-op for the rare export that might), so a normal 2-person
    // chat where both people sent messages under their real names also has exactly 2 distinct
    // senders — the same as it would if this were a 2-person group. Check indicators first.
    let group_indicators = [
        // English
        "created group",
        "added you",
        "added ",
        "changed the group",
        "changed the subject",
        "changed this group",
        "group icon",
        "joined using this group",
        "was added",
        "were added",
        "left",
        // Dutch — third-person (someone else did something)
        "heeft de groep aangemaakt",
        "heeft de groepsnaam",
        "heeft het groepspictogram",
        "heeft de groepsafbeelding",
        "toegevoegd aan de groep",
        "heeft je toegevoegd",
        "heeft de groepsbeschrijving",
        "is toegevoegd",
        "zijn toegevoegd",
        "heeft de groep verlaten",
        "verwijderd",
        // Dutch — second-person "Je hebt ..." (you did something in the group)
        "je hebt deze groep gemaakt",
        "je hebt de groep gemaakt",
        "je hebt de groepsnaam",
        "je hebt de groepsafbeelding",
        "je hebt de groepsbeschrijving",
        "je hebt de groep verlaten",
        "hebt de groep verlaten",
        "aan de groep toegevoegd",
        "uit de groep verwijderd",
        "hebt de groepsafbeelding",
        // German
        "hat die Gruppe erstellt",
        "zur Gruppe hinzugefügt",
        "hat das Gruppenthema",
        // Spanish
        "creó el grupo",
        "te añadió",
        // French
        "a créé le groupe",
        "vous a ajouté",
        "a ajouté",
        "a retiré",
        "a changé le sujet",
        "icône de groupe",
        "est parti",
        // Azerbaijani
        "qrupunu yaratdı",
        "əlavə etdi",
        "tərəfindən çıxarıldı",
        "tərk etdi",
        "mövzusu ilə əvəzlənmişdir",
        "qrup təsviri",
        // Catalan
        "ha creat el grup",
        "ha afegit",
        "ha expulsat",
        "ha canviat el tema",
        "icona del grup",
        "icona de grup",
        // Czech
        "vytvořil",
        "přidal",
        "odebral",
        "odešel",
        "změnil",
        "ikona skupiny",
        // Danish
        "oprettede gruppen",
        "tilføjede",
        "fjernede",
        "forlod",
        "ændrede emnet",
        "gruppe ikon",
        // German (additional)
        "hat die gruppe verlassen",
        "gruppenbild",
        // Spanish (additional)
        "eliminó",
        "salió",
        "cambió el asunto",
        "icono del grupo",
        // Estonian
        "lõi grupi",
        "lisas",
        "eemaldas",
        "lahkus",
        "seadis teemaks",
        "grupi ikoon",
        // Finnish
        "loi ryhmän",
        "lisäsi henkilön",
        "poisti henkilön",
        "poistui",
        "vaihtoi aiheeksi",
        "ryhmän kuvake",
        // Croatian
        "stvorio",
        "dodao",
        "uklonio",
        "izašao",
        "promijenio",
        "ikona grupe",
        // Hungarian
        "létrehozta",
        "hozzáadta",
        "eltávolította",
        "kilépett",
        "lecserélte a témát",
        "csoportikon",
        // Indonesian
        "membuat grup",
        "menambahkan",
        "mengeluarkan",
        "mengubah subjek",
        "ikon grup telah",
        // Italian
        "ha creato il gruppo",
        "ha aggiunto",
        "ha rimosso",
        "ha abbandonato",
        "ha cambiato l'oggetto",
        "immagine del gruppo",
        // Lithuanian
        "sukūrė grupę",
        "pridėjo",
        "pašalino",
        "išėjo",
        "pakeitė temą",
        "grupės piktograma",
        // Latvian
        "izveidoja grupu",
        "pievienoja",
        "noņēma",
        "aizgāja",
        "nomainīja tematu",
        "grupas ikona",
        // Malay
        "mencipta grup",
        "telah menambah",
        "telah membuang",
        "telah keluar",
        "ikon grup telah dikemaskini",
        // Norwegian Bokmal
        "laget gruppen",
        "la til",
        "fjernet",
        "forlot gruppen",
        "endret emnet",
        "gruppeikon",
        // Polish
        "utworzył",
        "dodał",
        "usunął",
        "opuścił",
        "zmienił",
        "ikonę grupy",
        "ikona grupy",
        // Portuguese (Portugal)
        "criou o grupo",
        "adicionou",
        "removeu",
        "saiu do grupo",
        "alterou o assunto",
        "ícone do grupo",
        // Portuguese (Brazil)
        "alterou o nome do grupo para",
        "imagem do grupo",
        // Romanian
        "a creat grupul",
        "a adăugat",
        "a eliminat",
        "a ieșit",
        "a schimbat subiectul",
        "imaginea grupului",
        // Slovak
        "vytvoril",
        "odobral",
        "odišiel",
        // Slovenian
        "je ustvaril",
        "je dodal",
        "je odstranil",
        "je odšel",
        "je spremenil temo",
        "slika skupine",
        // Albanian
        "krijoi grupin",
        "shtoi",
        "hoqi",
        "u largua",
        "ndryshoi titullin",
        "ikona e grupit",
        // Swedish
        "har skapat grupp",
        "lade till",
        "tog bort",
        "lämnade",
        "ändrade ämnet",
        "gruppikon",
        // Swahili
        "ametengeneza kikundi",
        "amemuongeza",
        "amemuondoa",
        "katoka",
        "amebadilisha mada",
        "ikoni ya kikundi",
        // Tagalog
        "binuo ang grupong",
        "idinagdag",
        "inalis",
        "umalis",
        // Turkish
        "grubunu oluşturdu",
        "kişisini ekledi",
        "kişisini çıkardı",
        "ayrıldı",
        "olarak değiştirdi",
        "grup simgesi",
        // Uzbek
        "guruhini yaratdi",
        "qo‘shdi",
        "o‘chirdi",
        "tark etdi",
        "o‘zgartirdi",
        "guruh rasmi",
        // Vietnamese
        "đã tạo nhóm",
        "đã thêm",
        "đã bỏ",
        "đã rời nhóm",
        "đã đổi tên nhóm thành",
        "biểu tượng nhóm",
    ];
    let has_group_indicator = messages.iter()
        .filter(|m| m.sender == "System" || m.msg_type == "system")
        .any(|m| {
            let content_lower = m.content.to_lowercase();
            group_indicators.iter().any(|indicator| content_lower.contains(&indicator.to_lowercase()))
        });
    if has_group_indicator {
        return true;
    }
    // Fallback for exports where the group-creation message wasn't captured: 3+ distinct
    // senders can only happen in a group, since a 1-on-1 chat has exactly 2 participants.
    senders.len() > 2
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
                // Azerbaijani's "X əlavə etdi: Y" (added) system message legitimately contains
                // ": ", so it structurally looks like "Sender: text" to this same pattern and
                // would otherwise be split into a fake sender/message pair below. Detect it here,
                // before that split assumption applies, and store it as a system message instead.
                if sender_candidate.contains(" əlavə etdi") {
                    in_code_block = false;
                    after_fence_close = false;
                    blank_since_close = false;
                    if let Some(msg) = current_msg.take() {
                        messages.push(msg);
                    }
                    let content_text_candidate = caps.get(4).map(|m| m.as_str()).unwrap_or("");
                    current_msg = Some(Message {
                        id: None,
                        timestamp: format!("{} {}", date_str, time_str),
                        sender: "System".to_string(),
                        msg_type: "system".to_string(),
                        content: format!("{}: {}", sender_candidate, content_text_candidate),
                        media: None,
                        duration: None,
                        tag_ext: None,
                        display_name: None,
                        is_favorite: None,
                    });
                    matched = true;
                    break;
                }
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
    // Record the rename in history
    conn.execute(
        "INSERT INTO chat_file_renames (chat_id, message_index, original_filename, new_filename) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![chat_id, message_idx, old_filename, new_filename],
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
                link_chat_to_contact_if_match(&conn, &chat_id, meta.is_group);
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
        // Check if content, sender, or timestamp matches (skip system messages)
        if msg_type != "system" && (content.to_lowercase().contains(&search_lower) || 
           sender.to_lowercase().contains(&search_lower) ||
           timestamp.to_lowercase().contains(&search_lower)) {
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
    // Note: date_from/date_to are "YYYY-MM-DD" strings from an <input type="date">, while the
    // stored timestamp column holds WhatsApp's native export format (e.g. "17/06/2024 00:04").
    // The two aren't lexicographically comparable, so date bounds are applied after fetching
    // via timestamp_to_epoch instead of in SQL.
    let epoch_from = filters.date_from.as_deref().and_then(|d| iso_date_to_epoch(d, false));
    let epoch_to = filters.date_to.as_deref().and_then(|d| iso_date_to_epoch(d, true));
    if let Some(sender_filter) = &filters.sender {
        where_conditions.push(format!("LOWER(sender) LIKE ?{}", param_index));
        params.push(Box::new(format!("%{}%", sender_filter.to_lowercase())));
        param_index += 1;
    }
    if let Some(msg_type_filter) = &filters.msg_type {
        // Media files that predate an extension being added to the classifier (or that were
        // never reclassified out of "file") still render client-side as image/video/audio via
        // extension sniffing (see IMAGE_EXTS/VIDEO_EXTS/AUDIO_EXTS in types.ts). Mirror that
        // fallback here so the filter matches what the user actually sees in the chat.
        let exts: Option<&[&str]> = match msg_type_filter.as_str() {
            "video" => Some(&["mp4", "mov", "avi", "mkv", "webm", "m4v", "ts", "flv", "wmv"]),
            "image" => Some(&["jpg", "jpeg", "png", "gif", "webp", "bmp", "heic", "heif", "svg", "tif", "tiff", "avif", "jfif", "ico"]),
            "audio" => Some(&["mp3", "ogg", "opus", "wav", "m4a", "aac", "3gp", "3gpp", "amr", "flac", "wma"]),
            _ => None,
        };
        if let Some(exts) = exts {
            let mut sub_conditions = vec![format!("msg_type = ?{}", param_index)];
            params.push(Box::new(msg_type_filter.clone()));
            param_index += 1;
            let mut ext_conditions = Vec::new();
            for ext in exts {
                ext_conditions.push(format!("LOWER(media) LIKE ?{}", param_index));
                params.push(Box::new(format!("%.{}", ext)));
                param_index += 1;
            }
            sub_conditions.push(format!("(msg_type = 'file' AND ({}))", ext_conditions.join(" OR ")));
            where_conditions.push(format!("({})", sub_conditions.join(" OR ")));
        } else {
            where_conditions.push(format!("msg_type = ?{}", param_index));
            params.push(Box::new(msg_type_filter.clone()));
            param_index += 1;
        }
    }
    let _ = param_index;
    // Exclude system messages unless the user explicitly asked to filter for them
    if filters.msg_type.as_deref() != Some("system") {
        where_conditions.push("msg_type != 'system'".to_string());
    }
    let where_clause = where_conditions.join(" AND ");
    // Fetch filtered messages with their true global row index via correlated subquery.
    // This ensures message_index matches the frontend messages[] array position regardless
    // of what filters are active.
    let mut stmt = conn.prepare(&format!(
        "SELECT (SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = m1.chat_id AND m2.id < m1.id) as row_idx,
                m1.timestamp, m1.sender, m1.msg_type, m1.content
         FROM messages m1
         WHERE {} ORDER BY m1.id",
        where_clause
    )).map_err(|e| e.to_string())?;
    let search_lower = filters.query.to_lowercase();
    let mut results = Vec::new();
    // Convert params for query_map
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(&param_refs[..], |row| {
        Ok((
            row.get::<_, i64>(0)?, // row_idx (global position)
            row.get::<_, String>(1)?, // timestamp
            row.get::<_, String>(2)?, // sender
            row.get::<_, String>(3)?, // msg_type
            row.get::<_, String>(4)?, // content
        ))
    }).map_err(|e| e.to_string())?;
    for row in rows {
        let (row_idx, timestamp, sender, msg_type, content) = row.map_err(|e| e.to_string())?;
        // Apply date range filter (compared as epoch seconds, see note above)
        if epoch_from.is_some() || epoch_to.is_some() {
            let ts_epoch = timestamp_to_epoch(&timestamp);
            if let Some(from) = epoch_from {
                if ts_epoch < from { continue; }
            }
            if let Some(to) = epoch_to {
                if ts_epoch > to { continue; }
            }
        }
        // Apply text search filter if query is not empty
        if filters.query.is_empty() ||
           content.to_lowercase().contains(&search_lower) ||
           sender.to_lowercase().contains(&search_lower) ||
           timestamp.to_lowercase().contains(&search_lower) {
            results.push(SearchResult {
                message_index: row_idx as usize,
                timestamp,
                sender,
                msg_type,
                content,
            });
        }
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
         WHERE LOWER(COALESCE(profiles.name, chats.name)) LIKE ?1
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
pub struct CrossChatSearchResult {
    pub chat_id: String,
    pub chat_name: String,
    pub message_index: usize,
    pub timestamp: String,
    pub sender: String,
    pub content: String,
    pub msg_type: String,
}

const CROSS_CHAT_SEARCH_LIMIT: usize = 200;

fn search_messages_across_chats_impl(conn: &Connection, search_lower: &str, limit: usize) -> Result<Vec<CrossChatSearchResult>, String> {
    let mut stmt = conn.prepare(
        "SELECT (SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = m1.chat_id AND m2.id < m1.id) as row_idx,
                m1.chat_id, COALESCE(profiles.name, chats.name) as chat_name,
                m1.timestamp, m1.sender, m1.msg_type, m1.content
         FROM messages m1
         JOIN chats ON chats.id = m1.chat_id
         LEFT JOIN profiles ON profiles.chat_id = m1.chat_id
         WHERE m1.msg_type != 'system' AND LOWER(m1.content) LIKE ?1
         ORDER BY chats.last_message_epoch DESC, m1.id DESC
         LIMIT ?2"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(rusqlite::params![search_lower, limit as i64], |row| {
        Ok(CrossChatSearchResult {
            message_index: row.get::<_, i64>(0)? as usize,
            chat_id: row.get(1)?,
            chat_name: row.get(2)?,
            timestamp: row.get(3)?,
            sender: row.get(4)?,
            msg_type: row.get(5)?,
            content: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for r in rows {
        result.push(r.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
fn search_messages_across_chats(query: String) -> Result<Vec<CrossChatSearchResult>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let conn = get_db();
    let search_lower = format!("%{}%", query.to_lowercase());
    search_messages_across_chats_impl(&conn, &search_lower, CROSS_CHAT_SEARCH_LIMIT)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub chat_id: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub photo_path: Option<String>,
    pub phone_number: Option<String>,
    pub original_name: Option<String>,
    pub contact_id: Option<String>,
}

#[tauri::command]
fn get_profile(chat_id: String) -> Result<Profile, String> {
    let conn = get_db();
    let original_name: Option<String> = conn.query_row(
        "SELECT COALESCE(original_name, name) FROM chats WHERE id = ?1",
        [&chat_id],
        |row| row.get(0),
    ).ok();
    let contact_id: Option<String> = conn.query_row(
        "SELECT contact_id FROM chats WHERE id = ?1",
        [&chat_id],
        |row| row.get(0),
    ).ok().flatten();
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
            original_name: None,
            contact_id: None,
        })
    });
    match profile {
        Ok(mut p) => { p.original_name = original_name; p.contact_id = contact_id; Ok(p) },
        Err(_) => Ok(Profile {
            chat_id,
            name: None,
            notes: None,
            photo_path: None,
            phone_number: None,
            original_name,
            contact_id,
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
    // Mirror into the linked contact, if this chat is tied to one, so a group participant's
    // resolved profile (and any other chat sharing that contact) sees the same update.
    let linked_contact_id: Option<String> = conn.query_row(
        "SELECT contact_id FROM chats WHERE id = ?1",
        params![&chat_id],
        |row| row.get(0),
    ).ok().flatten();
    if let Some(contact_id) = linked_contact_id {
        conn.execute(
            "UPDATE contacts SET
             name = COALESCE(?2, name),
             notes = COALESCE(?3, notes),
             photo_path = COALESCE(?4, photo_path),
             phone_number = COALESCE(?5, phone_number)
             WHERE id = ?1",
            params![&contact_id, &name, &notes, &photo_path, &phone_number],
        ).map_err(|e| e.to_string())?;
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
    let linked_contact_id: Option<String> = conn.query_row(
        "SELECT contact_id FROM chats WHERE id = ?1",
        params![&chat_id],
        |row| row.get(0),
    ).ok().flatten();
    if let Some(contact_id) = linked_contact_id {
        conn.execute(
            "UPDATE contacts SET photo_path = NULL WHERE id = ?1",
            params![&contact_id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// ==== PIN Lock ====
// A local UI gate only — it does NOT encrypt the SQLite database, which remains plaintext
// on disk regardless of whether a PIN is set. Recovery is via a security question set
// alongside the PIN (see reset_pin_with_recovery_answer below), so forgetting the PIN can't
// permanently lock you out of a years-long archive, but resetting still requires knowing an
// answer only you'd know — not just a click.
//
// The recovery answer is stored in plaintext (like the question), not hashed — unlike the PIN
// itself. That's deliberate: the whole point of this feature is to let the settings dialog show
// the current question/answer back to the user, pre-filled, for editing — which is impossible
// with a one-way hash. This doesn't weaken anything the lock actually protects, since the lock
// was never encryption in the first place; the SQLite database holding both this value and the
// entire chat archive is already plaintext on disk regardless.

fn hash_secret(secret: &str) -> Result<String, String> {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::{SaltString, rand_core::OsRng};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| e.to_string())
}

fn verify_secret_hash(secret: &str, stored_hash: &str) -> Result<bool, String> {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};
    let parsed = PasswordHash::new(stored_hash).map_err(|e| e.to_string())?;
    Ok(Argon2::default().verify_password(secret.as_bytes(), &parsed).is_ok())
}

// Recovery answers are matched case/whitespace-insensitively so a legitimate user isn't
// tripped up by capitalization or trailing spaces they didn't type consistently the first time.
fn normalize_recovery_answer(answer: &str) -> String {
    answer.trim().to_lowercase()
}

fn get_app_setting(conn: &Connection, key: &str) -> Result<Option<String>, String> {
    conn.query_row("SELECT value FROM app_settings WHERE key = ?1", [key], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())
}

fn set_app_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

fn delete_app_setting(conn: &Connection, key: &str) -> Result<(), String> {
    conn.execute("DELETE FROM app_settings WHERE key = ?1", [key]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn has_pin() -> Result<bool, String> {
    Ok(get_app_setting(&get_db(), "pin_hash")?.is_some())
}

// A recovery question + answer must be set alongside every PIN (initial set or change) —
// it's what reset_pin_with_recovery_answer checks instead of a bare unauthenticated reset.
#[tauri::command]
fn set_pin(pin: String, recovery_question: String, recovery_answer: String) -> Result<(), String> {
    if pin.trim().is_empty() {
        return Err("PIN cannot be empty".to_string());
    }
    if recovery_question.trim().is_empty() || recovery_answer.trim().is_empty() {
        return Err("A recovery question and answer are required".to_string());
    }
    let pin_hash = hash_secret(&pin)?;
    let conn = get_db();
    set_app_setting(&conn, "pin_hash", &pin_hash)?;
    set_app_setting(&conn, "recovery_question", recovery_question.trim())?;
    set_app_setting(&conn, "recovery_answer", recovery_answer.trim())
}

#[tauri::command]
fn verify_pin(pin: String) -> Result<bool, String> {
    match get_app_setting(&get_db(), "pin_hash")? {
        Some(hash) => verify_secret_hash(&pin, &hash),
        None => Ok(true), // nothing set to verify against; caller must gate via has_pin first
    }
}

// Updates only the PIN, leaving the existing recovery question/answer untouched — the
// counterpart to change_recovery_question below, since the user can change either
// independently. There's only ever one active PIN and one active recovery question stored
// (single-row app_settings keys), so updating one in place can't desync from the other.
#[tauri::command]
fn change_pin(current_pin: String, new_pin: String) -> Result<(), String> {
    if new_pin.trim().is_empty() {
        return Err("PIN cannot be empty".to_string());
    }
    let conn = get_db();
    match get_app_setting(&conn, "pin_hash")? {
        Some(hash) if verify_secret_hash(&current_pin, &hash)? => {
            let new_hash = hash_secret(&new_pin)?;
            set_app_setting(&conn, "pin_hash", &new_hash)
        }
        Some(_) => Err("Incorrect current PIN".to_string()),
        None => Err("No PIN is currently set".to_string()),
    }
}

// Updates only the recovery question/answer, leaving the existing PIN untouched — the
// counterpart to change_pin above.
#[tauri::command]
fn change_recovery_question(current_pin: String, recovery_question: String, recovery_answer: String) -> Result<(), String> {
    if recovery_question.trim().is_empty() || recovery_answer.trim().is_empty() {
        return Err("A recovery question and answer are required".to_string());
    }
    let conn = get_db();
    match get_app_setting(&conn, "pin_hash")? {
        Some(hash) if verify_secret_hash(&current_pin, &hash)? => {
            set_app_setting(&conn, "recovery_question", recovery_question.trim())?;
            set_app_setting(&conn, "recovery_answer", recovery_answer.trim())
        }
        Some(_) => Err("Incorrect current PIN".to_string()),
        None => Err("No PIN is currently set".to_string()),
    }
}

// Only clears the PIN itself — the recovery question/answer is left in place, since it's a
// separate piece of info from the PIN and there's no reason to lose it just because the PIN
// was removed. It'll simply sit unused until a new PIN is set (set_pin always asks for a
// fresh recovery question/answer at that point, overwriting whatever's left over).
#[tauri::command]
fn clear_pin(current_pin: String) -> Result<(), String> {
    let conn = get_db();
    match get_app_setting(&conn, "pin_hash")? {
        Some(hash) if verify_secret_hash(&current_pin, &hash)? => delete_app_setting(&conn, "pin_hash"),
        Some(_) => Err("Incorrect PIN".to_string()),
        None => Ok(()),
    }
}

#[tauri::command]
fn get_recovery_question() -> Result<Option<String>, String> {
    get_app_setting(&get_db(), "recovery_question")
}

#[tauri::command]
fn get_recovery_answer() -> Result<Option<String>, String> {
    get_app_setting(&get_db(), "recovery_answer")
}

// The "forgot PIN" escape hatch, only reachable from the lock screen's "Forgot PIN?" flow
// (never from the already-unlocked settings dialog, which uses clear_pin above and requires
// the current PIN). Requires the recovery answer set alongside the PIN rather than being a
// bare unauthenticated reset — real friction against a casual bypass, while still guaranteed
// recoverable by the legitimate user, since this lock never encrypted the data in the first
// place and refusing to ever reset would only risk losing access to the archive, not protect it.
// Only clears the PIN itself, same as clear_pin above — the recovery question/answer that was
// just used to verify this reset stays in place rather than being thrown away.
#[tauri::command]
fn reset_pin_with_recovery_answer(answer: String) -> Result<(), String> {
    let conn = get_db();
    match get_app_setting(&conn, "recovery_answer")? {
        Some(stored) if normalize_recovery_answer(&answer) == normalize_recovery_answer(&stored) => delete_app_setting(&conn, "pin_hash"),
        Some(_) => Err("Incorrect answer".to_string()),
        None => Err("No recovery question is set for this PIN".to_string()),
    }
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
    Ok(())
}

// =============================================================================
// Contacts — shared profile identity for group participants
// =============================================================================

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoLinkEvent {
    pub contact_id: String,
    pub contact_name: String,
    pub chat_id: String,
    pub chat_name: String,
    pub via_group: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Contact {
    pub id: String,
    pub normalized_key: String,
    pub display_key: String,
    pub name: Option<String>,
    pub notes: Option<String>,
    pub photo_path: Option<String>,
    pub phone_number: Option<String>,
}

fn get_contact_by_id(conn: &Connection, contact_id: &str) -> Option<Contact> {
    conn.query_row(
        "SELECT id, normalized_key, display_key, name, notes, photo_path, phone_number FROM contacts WHERE id = ?1",
        params![contact_id],
        |row| Ok(Contact {
            id: row.get(0)?,
            normalized_key: row.get(1)?,
            display_key: row.get(2)?,
            name: row.get(3)?,
            notes: row.get(4)?,
            photo_path: row.get(5)?,
            phone_number: row.get(6)?,
        })
    ).ok()
}

// Links a 1-on-1 chat to a contact, then copies the contact's non-null fields down into the
// chat's own profile (same COALESCE semantics as update_profile) so its Profile dialog reflects
// the shared data immediately.
fn link_chat_and_contact(conn: &Connection, chat_id: &str, contact_id: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE chats SET contact_id = ?1 WHERE id = ?2",
        params![contact_id, chat_id],
    ).map_err(|e| e.to_string())?;
    if let Some(contact) = get_contact_by_id(conn, contact_id) {
        conn.execute(
            "INSERT INTO profiles (chat_id, name, notes, photo_path, phone_number, profile_modified)
             VALUES (?1, ?2, ?3, ?4, ?5, 1)
             ON CONFLICT(chat_id) DO UPDATE SET
             name = COALESCE(?2, name),
             notes = COALESCE(?3, notes),
             photo_path = COALESCE(?4, photo_path),
             phone_number = COALESCE(?5, phone_number),
             profile_modified = 1",
            params![chat_id, &contact.name, &contact.notes, &contact.photo_path, &contact.phone_number],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

// Finds an unlinked 1-on-1 chat whose *display* name (`chats.name` — already stripped of any
// "WhatsApp-chat met " / "WhatsApp Chat with " / etc. export prefix at import time) normalizes
// to match. Deliberately does NOT use `original_name`: that column keeps the raw ZIP filename
// (prefix and all) for merge-on-reimport dedup, which lives in a different identity space than
// a bare group-participant sender string like "Manuel" — comparing against it would (and did)
// silently fail to match ordinary prefixed exports.
fn find_unlinked_one_on_one_chat_by_name(conn: &Connection, normalized: &str) -> Option<String> {
    let mut stmt = conn.prepare(
        "SELECT id, name FROM chats WHERE is_group = 0 AND contact_id IS NULL"
    ).ok()?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    }).ok()?;
    for row in rows.flatten() {
        if normalize_chat_name(&row.1) == normalized {
            return Some(row.0);
        }
    }
    None
}

// Seeds a new contact from an existing 1-on-1 chat's profile data and links them together.
// Shared by the interactive participant-edit resolver and the reconciliation pass below.
fn create_contact_from_chat_and_link(conn: &Connection, chat_id: &str, normalized: &str, display_key: &str) -> Result<String, String> {
    let existing_profile: (Option<String>, Option<String>, Option<String>, Option<String>) = conn.query_row(
        "SELECT name, notes, photo_path, phone_number FROM profiles WHERE chat_id = ?1",
        params![chat_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    ).unwrap_or((None, None, None, None));
    let new_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO contacts (id, normalized_key, display_key, name, notes, photo_path, phone_number) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![&new_id, normalized, display_key, &existing_profile.0, &existing_profile.1, &existing_profile.2, &existing_profile.3],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE chats SET contact_id = ?1 WHERE id = ?2",
        params![&new_id, chat_id],
    ).map_err(|e| e.to_string())?;
    Ok(new_id)
}

fn chat_name_for(conn: &Connection, chat_id: &str) -> String {
    conn.query_row("SELECT name FROM chats WHERE id = ?1", params![chat_id], |row| row.get(0))
        .unwrap_or_else(|_| chat_id.to_string())
}

// Comprehensive reconciliation: links any contact and 1-on-1 chat that represent the same person
// but haven't been connected yet. Runs at app startup and after every import so this doesn't
// require manually opening/editing each chat, or restarting the app to pick up new matches.
fn reconcile_contacts_and_chats(conn: &Connection) -> Vec<AutoLinkEvent> {
    let mut events = Vec::new();
    // Pass 1: for every group's participants (including former members — matching is about
    // identity, not current membership), auto-create + link a contact when a matching unlinked
    // 1-on-1 chat exists and no contact exists yet for that person. Scoped strictly to "a match
    // was found" — never creates a contact for a participant with no matching chat.
    let group_chats: Vec<(String, String)> = {
        let mut stmt = match conn.prepare("SELECT id, name FROM chats WHERE is_group = 1") {
            Ok(s) => s,
            Err(_) => return events,
        };
        let rows = match stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
            Ok(rows) => rows,
            Err(_) => return events,
        };
        rows.flatten().collect()
    };
    for (group_chat_id, group_name) in group_chats {
        let participants: Vec<String> = {
            let mut stmt = match conn.prepare(
                "SELECT DISTINCT sender FROM messages WHERE chat_id = ?1 AND sender != 'System' AND msg_type != 'system'"
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };
            let rows = match stmt.query_map(params![&group_chat_id], |row| row.get::<_, String>(0)) {
                Ok(rows) => rows,
                Err(_) => continue,
            };
            rows.flatten().collect()
        };
        for participant in participants {
            let normalized = normalize_chat_name(&participant);
            let existing_contact_id: Option<String> = conn.query_row(
                "SELECT id FROM contacts WHERE normalized_key = ?1",
                params![&normalized],
                |row| row.get(0),
            ).ok();
            if let Some(contact_id) = existing_contact_id {
                // Contact already resolved elsewhere (edit-click or earlier match) — backfill this
                // group into contact_groups too. ON CONFLICT DO NOTHING so a group the user
                // explicitly removed (excluded = 1) is never silently resurrected by a later
                // reimport/restart; only re-editing that participant from this group brings it back.
                let _ = conn.execute(
                    "INSERT INTO contact_groups (contact_id, chat_id, excluded) VALUES (?1, ?2, 0)
                     ON CONFLICT(contact_id, chat_id) DO NOTHING",
                    params![&contact_id, &group_chat_id],
                );
                continue;
            }
            if let Some(chat_id) = find_unlinked_one_on_one_chat_by_name(conn, &normalized) {
                let Ok(contact_id) = create_contact_from_chat_and_link(conn, &chat_id, &normalized, &participant) else { continue; };
                let _ = conn.execute(
                    "INSERT INTO contact_groups (contact_id, chat_id, excluded) VALUES (?1, ?2, 0)
                     ON CONFLICT(contact_id, chat_id) DO UPDATE SET excluded = 0",
                    params![&contact_id, &group_chat_id],
                );
                events.push(AutoLinkEvent {
                    contact_id,
                    contact_name: participant.clone(),
                    chat_name: chat_name_for(conn, &chat_id),
                    chat_id,
                    via_group: Some(group_name.clone()),
                });
            }
        }
    }
    // Pass 2: contacts that already existed before this pass (from a previous edit-click, or
    // just created above), matched against unlinked chats — covers the "import while the app is
    // already running" gap, without needing a restart for the startup pass to catch it.
    let contacts: Vec<(String, String)> = {
        let mut stmt = match conn.prepare("SELECT id, normalized_key FROM contacts") {
            Ok(s) => s,
            Err(_) => return events,
        };
        let rows = match stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
            Ok(rows) => rows,
            Err(_) => return events,
        };
        rows.flatten().collect()
    };
    for (contact_id, normalized_key) in contacts {
        let already_linked: bool = conn.query_row(
            "SELECT 1 FROM chats WHERE contact_id = ?1",
            params![&contact_id],
            |_| Ok(true),
        ).unwrap_or(false);
        if already_linked { continue; }
        if let Some(chat_id) = find_unlinked_one_on_one_chat_by_name(conn, &normalized_key) {
            if link_chat_and_contact(conn, &chat_id, &contact_id).is_ok() {
                let contact_name = get_contact_by_id(conn, &contact_id)
                    .map(|c| c.name.unwrap_or(c.display_key))
                    .unwrap_or_else(|| normalized_key.clone());
                events.push(AutoLinkEvent {
                    contact_id,
                    contact_name,
                    chat_name: chat_name_for(conn, &chat_id),
                    chat_id,
                    via_group: None,
                });
            }
        }
    }
    events
}

// Runs the reconciliation pass once, off the async runtime thread, after an import operation
// completes — so batch-import cross-references (a group and its matching 1-on-1 chat imported
// together) resolve immediately, without requiring an app restart.
async fn run_post_import_reconciliation() {
    let _ = tauri::async_runtime::spawn_blocking(|| {
        let conn = get_db();
        queue_auto_link_events(reconcile_contacts_and_chats(&conn));
    }).await;
}

// Called after a 1-on-1 chat is created/resolved during import. No-op for groups or chats
// already linked. If an unlinked contact exists whose normalized key matches this chat's
// (already prefix-stripped) display name, links them.
fn link_chat_to_contact_if_match(conn: &Connection, chat_id: &str, is_group: bool) {
    if is_group { return; }
    let already_linked: bool = conn.query_row(
        "SELECT contact_id IS NOT NULL FROM chats WHERE id = ?1",
        params![chat_id],
        |row| row.get(0),
    ).unwrap_or(false);
    if already_linked { return; }
    let name: Option<String> = conn.query_row(
        "SELECT name FROM chats WHERE id = ?1",
        params![chat_id],
        |row| row.get(0),
    ).ok();
    let Some(name) = name else { return; };
    let normalized = normalize_chat_name(&name);
    let contact_id: Option<String> = conn.query_row(
        "SELECT id FROM contacts WHERE normalized_key = ?1",
        params![&normalized],
        |row| row.get(0),
    ).ok();
    if let Some(contact_id) = contact_id {
        let _ = link_chat_and_contact(conn, chat_id, &contact_id);
    }
}

#[tauri::command]
fn get_or_create_contact_for_participant(participant_name: String, group_chat_id: String) -> Result<Contact, String> {
    let conn = get_db();
    let normalized = normalize_chat_name(&participant_name);
    // A contact for this normalized key may already exist (e.g. created from a different
    // group), regardless of whether a matching 1-on-1 chat also exists — always check for and
    // reuse it first, rather than potentially creating a duplicate.
    let existing_contact_id: Option<String> = conn.query_row(
        "SELECT id FROM contacts WHERE normalized_key = ?1",
        params![&normalized],
        |row| row.get(0),
    ).ok();
    let contact_id = if let Some(cid) = existing_contact_id {
        let already_linked: bool = conn.query_row(
            "SELECT 1 FROM chats WHERE contact_id = ?1",
            params![&cid],
            |_| Ok(true),
        ).unwrap_or(false);
        // Not linked yet — opportunistically link now if a matching chat exists (it may have
        // existed all along, or been imported at any point since this contact was created).
        if !already_linked {
            if let Some(chat_id) = find_unlinked_one_on_one_chat_by_name(&conn, &normalized) {
                let _ = link_chat_and_contact(&conn, &chat_id, &cid);
            }
        }
        cid
    } else if let Some(chat_id) = find_unlinked_one_on_one_chat_by_name(&conn, &normalized) {
        // No contact yet, but a matching unlinked 1-on-1 chat exists — seed a new contact from
        // its current profile data, then link them.
        create_contact_from_chat_and_link(&conn, &chat_id, &normalized, &participant_name)?
    } else {
        // No contact, no matching chat — create a fresh shadow contact.
        let new_id = Uuid::new_v4().to_string();
        let (seed_name, seed_phone) = if looks_like_phone_number(&participant_name) {
            (None, Some(participant_name.clone()))
        } else {
            (Some(participant_name.clone()), None)
        };
        conn.execute(
            "INSERT INTO contacts (id, normalized_key, display_key, name, phone_number) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![&new_id, &normalized, &participant_name, &seed_name, &seed_phone],
        ).map_err(|e| e.to_string())?;
        new_id
    };
    // Regardless of which branch above, record/reaffirm this contact's presence in this group.
    conn.execute(
        "INSERT INTO contact_groups (contact_id, chat_id, excluded) VALUES (?1, ?2, 0)
         ON CONFLICT(contact_id, chat_id) DO UPDATE SET excluded = 0",
        params![&contact_id, &group_chat_id],
    ).map_err(|e| e.to_string())?;
    get_contact_by_id(&conn, &contact_id).ok_or_else(|| "Failed to load contact".to_string())
}

// Drains and returns any auto-links reconcile_contacts_and_chats has established since this was
// last called, so the frontend can show a one-time review popup. Called once after each import
// operation and once on initial app load (to catch whatever the startup pass found).
#[tauri::command]
fn take_pending_auto_links() -> Vec<AutoLinkEvent> {
    let mutex = PENDING_AUTO_LINKS.get_or_init(|| Mutex::new(Vec::new()));
    match mutex.lock() {
        Ok(mut pending) => std::mem::take(&mut *pending),
        Err(_) => Vec::new(),
    }
}

#[tauri::command]
fn update_contact_profile(contact_id: String, name: Option<String>, notes: Option<String>, photo_path: Option<String>, phone_number: Option<String>) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "UPDATE contacts SET
         name = COALESCE(?2, name),
         notes = COALESCE(?3, notes),
         photo_path = COALESCE(?4, photo_path),
         phone_number = COALESCE(?5, phone_number)
         WHERE id = ?1",
        params![&contact_id, &name, &notes, &photo_path, &phone_number],
    ).map_err(|e| e.to_string())?;
    // Mirror into the linked chat's own profile, if any.
    let linked_chat_id: Option<String> = conn.query_row(
        "SELECT id FROM chats WHERE contact_id = ?1",
        params![&contact_id],
        |row| row.get(0),
    ).ok();
    if let Some(chat_id) = linked_chat_id {
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
    }
    Ok(())
}

#[tauri::command]
fn remove_contact_photo(contact_id: String) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "UPDATE contacts SET photo_path = NULL WHERE id = ?1",
        params![&contact_id],
    ).map_err(|e| e.to_string())?;
    let linked_chat_id: Option<String> = conn.query_row(
        "SELECT id FROM chats WHERE contact_id = ?1",
        params![&contact_id],
        |row| row.get(0),
    ).ok();
    if let Some(chat_id) = linked_chat_id {
        conn.execute(
            "UPDATE profiles SET photo_path = NULL WHERE chat_id = ?1",
            params![&chat_id],
        ).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_contact_groups(contact_id: String) -> Result<Vec<ChatMeta>, String> {
    let conn = get_db();
    let mut stmt = conn.prepare(
        "SELECT chats.id, COALESCE(profiles.name, chats.name), chats.last_message, chats.timestamp, chats.is_group, chats.zip_path, profiles.photo_path
         FROM contact_groups
         JOIN chats ON chats.id = contact_groups.chat_id
         LEFT JOIN profiles ON profiles.chat_id = chats.id
         WHERE contact_groups.contact_id = ?1 AND contact_groups.excluded = 0
         ORDER BY chats.last_message_epoch DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![&contact_id], |row| {
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
    Ok(rows.flatten().collect())
}

#[tauri::command]
fn remove_contact_group(contact_id: String, chat_id: String) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "UPDATE contact_groups SET excluded = 1 WHERE contact_id = ?1 AND chat_id = ?2",
        params![&contact_id, &chat_id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_linked_participants(participant_names: Vec<String>) -> Result<Vec<String>, String> {
    let conn = get_db();
    let mut linked = Vec::new();
    for name in participant_names {
        let normalized = normalize_chat_name(&name);
        let is_linked: bool = conn.query_row(
            "SELECT 1 FROM contacts JOIN chats ON chats.contact_id = contacts.id WHERE contacts.normalized_key = ?1",
            params![&normalized],
            |_| Ok(true),
        ).unwrap_or(false);
        if is_linked {
            linked.push(name);
        }
    }
    Ok(linked)
}

// Read-only lookup — never creates a contact, just reports the custom name already set on one
// (via the contact's profile) if the participant has been linked before. Used to make a group's
// per-message sender label reflect the contact's chosen name instead of the raw exported string.
#[tauri::command]
fn get_display_names_for_participants(participant_names: Vec<String>) -> Result<HashMap<String, String>, String> {
    let conn = get_db();
    let mut overrides = HashMap::new();
    for name in participant_names {
        let normalized = normalize_chat_name(&name);
        let custom_name: Option<String> = conn.query_row(
            "SELECT name FROM contacts WHERE normalized_key = ?1 AND name IS NOT NULL AND name != ''",
            params![&normalized],
            |row| row.get(0),
        ).ok();
        if let Some(custom_name) = custom_name {
            overrides.insert(name, custom_name);
        }
    }
    Ok(overrides)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MembershipEvent {
    Left,
    Added,
}

// Some group-attribute-change system messages ("removed the group picture", "changed the group
// name") share the same trailing verb as a person add/remove ("verwijderd"/"toegevoegd") and
// would otherwise be misread as someone leaving/joining. Reject those specific targets.
fn is_group_attribute_target(target: &str) -> bool {
    let t = target.trim();
    t.starts_with("de groep")
        || t.starts_with("le groupe")
        || t.starts_with("la description")
        || t.starts_with("l'icône")
        || t.starts_with("la photo")
        || t.eq_ignore_ascii_case("je") // "{actor} heeft je toegevoegd/verwijderd" references "you", not a trackable name
        || t.eq_ignore_ascii_case("you")
        || t.eq_ignore_ascii_case("vous")
        || t.eq_ignore_ascii_case("the group")
        || t.eq_ignore_ascii_case("this group")
        || t.to_lowercase().starts_with("the group ")
        // Azerbaijani
        || t.eq_ignore_ascii_case("səni")
        || t.to_lowercase().starts_with("qrup")
        // Catalan
        || t.to_lowercase().starts_with("el grup")
        // Czech
        || t.to_lowercase().starts_with("skupinu")
        || t.eq_ignore_ascii_case("tě")
        || t.eq_ignore_ascii_case("vás")
        // Danish
        || t.to_lowercase().starts_with("gruppen")
        || t.eq_ignore_ascii_case("dig")
        // German
        || t.to_lowercase().starts_with("die gruppe")
        || t.eq_ignore_ascii_case("dich")
        || t.eq_ignore_ascii_case("euch")
        // Spanish
        || t.to_lowercase().starts_with("el grupo")
        || t.eq_ignore_ascii_case("te")
        // Estonian
        || t.to_lowercase().starts_with("grupi")
        || t.eq_ignore_ascii_case("sind")
        // Finnish
        || t.to_lowercase().starts_with("ryhmän")
        || t.eq_ignore_ascii_case("sinut")
        // Croatian
        || t.to_lowercase().starts_with("grupu")
        || t.eq_ignore_ascii_case("tebe")
        // Hungarian
        || t.to_lowercase().starts_with("csoport")
        || t.eq_ignore_ascii_case("téged")
        // Indonesian / Malay (share "anda")
        || t.eq_ignore_ascii_case("anda")
        // Italian
        || t.to_lowercase().starts_with("il gruppo")
        // Lithuanian
        || t.to_lowercase().starts_with("grupę")
        || t.eq_ignore_ascii_case("tave")
        // Latvian
        || t.to_lowercase().starts_with("grupu")
        || t.eq_ignore_ascii_case("tevi")
        // Norwegian Bokmal
        || t.eq_ignore_ascii_case("deg")
        // Polish
        || t.eq_ignore_ascii_case("cię")
        // Portuguese (Portugal + Brazil)
        || t.to_lowercase().starts_with("este grupo")
        || t.to_lowercase().starts_with("o grupo")
        || t.to_lowercase().starts_with("deste grupo")
        || t.eq_ignore_ascii_case("você")
        // Romanian
        || t.to_lowercase().starts_with("grupul")
        // Slovak
        || t.eq_ignore_ascii_case("ťa")
        || t.eq_ignore_ascii_case("teba")
        // Slovenian
        || t.to_lowercase().starts_with("skupino")
        // Albanian
        || t.to_lowercase().starts_with("grupin")
        || t.eq_ignore_ascii_case("ty")
        // Swedish
        || t.eq_ignore_ascii_case("dig")
        // Swahili
        || t.to_lowercase().starts_with("ikoni ya kikundi")
        // Turkish
        || t.to_lowercase().starts_with("grubu")
        || t.eq_ignore_ascii_case("seni")
        // Uzbek
        || t.to_lowercase().starts_with("guruh rasmi")
        // Vietnamese
        || t.eq_ignore_ascii_case("bạn")
}

// WhatsApp batches multiple people into one system message: "X heeft Y en Z toegevoegd" or
// "X heeft Y, Z en W toegevoegd" or "X a ajouté Y et Z". Split on list separators across all
// supported languages (Dutch/English/French verified in spirit; the rest are inferred from each
// language's ordinary word for "and", not confirmed against a real batched export).
fn split_membership_targets(targets: &str) -> Vec<String> {
    let re = MEMBERSHIP_RE_SPLIT_TARGETS.get_or_init(|| regex::Regex::new(r",\s*| en | and | et | və | i | a | og | und | y | ja | és | dan | e | ir | un | și | in | dhe | och | na | at | ve | va | và ").unwrap());
    re.split(targets)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// Scans a group's system messages (in stored/chronological order) for membership-change events
// and returns (normalized_name, event) pairs in that same order — the caller keeps only each
// name's *last* event to know whether they've currently left. Dutch patterns are verified
// against real exported system messages; English, French, and the ~25 other languages below are
// best-effort, sourced from a decompiled WhatsApp APK's values-XX/strings.xml resources
// (https://github.com/GigaDroid/Decompiled-Whatsapp), not verified against real exports.
fn extract_membership_events(messages: &[String]) -> Vec<(String, MembershipEvent)> {
    let self_left = MEMBERSHIP_RE_SELF_LEFT.get_or_init(|| regex::Regex::new(r"^(.+) heeft de groep verlaten$").unwrap());
    let third_add = MEMBERSHIP_RE_THIRD_ADD.get_or_init(|| regex::Regex::new(r"^.+? heeft (.+) toegevoegd$").unwrap());
    let third_remove = MEMBERSHIP_RE_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"^.+? heeft (.+) verwijderd$").unwrap());
    let you_add = MEMBERSHIP_RE_YOU_ADD.get_or_init(|| regex::Regex::new(r"^Je hebt (.+) toegevoegd$").unwrap());
    let you_remove = MEMBERSHIP_RE_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"^Je hebt (.+) verwijderd$").unwrap());
    let en_self_left = MEMBERSHIP_RE_EN_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) left$").unwrap());
    let en_third_add = MEMBERSHIP_RE_EN_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? added (.+)$").unwrap());
    let en_third_remove = MEMBERSHIP_RE_EN_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? removed (.+)$").unwrap());
    let en_was_added = MEMBERSHIP_RE_EN_WAS_ADDED.get_or_init(|| regex::Regex::new(r"(?i)^(.+) was added$").unwrap());
    let en_was_removed = MEMBERSHIP_RE_EN_WAS_REMOVED.get_or_init(|| regex::Regex::new(r"(?i)^(.+) was removed$").unwrap());
    let fr_self_left = MEMBERSHIP_RE_FR_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) est parti[e]?$").unwrap());
    let fr_third_add = MEMBERSHIP_RE_FR_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? a ajouté (.+)$").unwrap());
    let fr_third_remove = MEMBERSHIP_RE_FR_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? a retiré (.+)$").unwrap());
    let fr_you_add = MEMBERSHIP_RE_FR_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Vous avez ajouté (.+)$").unwrap());
    let fr_you_remove = MEMBERSHIP_RE_FR_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Vous avez retiré (.+)$").unwrap());
    let az_self_left = MEMBERSHIP_RE_AZ_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) tərk etdi$").unwrap());
    let az_third_add = MEMBERSHIP_RE_AZ_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? əlavə etdi: (.+)$").unwrap());
    let az_third_remove = MEMBERSHIP_RE_AZ_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^(.+) .+? tərəfindən çıxarıldı$").unwrap());
    let az_you_add = MEMBERSHIP_RE_AZ_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^(.+) qrupa əlavə etdiniz$").unwrap());
    let az_you_remove = MEMBERSHIP_RE_AZ_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^(.+) sizin tərəfinizdən silindi$").unwrap());
    let ca_self_left = MEMBERSHIP_RE_CA_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) marxa$").unwrap());
    let ca_third_add = MEMBERSHIP_RE_CA_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? ha afegit (.+)$").unwrap());
    let ca_third_remove = MEMBERSHIP_RE_CA_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? ha expulsat a (.+)$").unwrap());
    let ca_you_add = MEMBERSHIP_RE_CA_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Has afegit a (.+)$").unwrap());
    let ca_you_remove = MEMBERSHIP_RE_CA_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Has esborrat a (.+)$").unwrap());
    let cs_self_left = MEMBERSHIP_RE_CS_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) odešel/a$").unwrap());
    let cs_third_add = MEMBERSHIP_RE_CS_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? přidal/a uživatele (.+)\.$").unwrap());
    let cs_third_remove = MEMBERSHIP_RE_CS_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? odebral\(a\) uživatele (.+)\.$").unwrap());
    let cs_you_add = MEMBERSHIP_RE_CS_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Přidal/a jste uživatele (.+)\.$").unwrap());
    let cs_you_remove = MEMBERSHIP_RE_CS_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Odebrali jste uživatele (.+)\.$").unwrap());
    let da_self_left = MEMBERSHIP_RE_DA_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) forlod$").unwrap());
    let da_third_add = MEMBERSHIP_RE_DA_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? tilføjede (.+)$").unwrap());
    let da_third_remove = MEMBERSHIP_RE_DA_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? fjernede (.+)$").unwrap());
    let da_you_add = MEMBERSHIP_RE_DA_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Du tilføjede (.+)$").unwrap());
    let da_you_remove = MEMBERSHIP_RE_DA_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Du fjernede (.+)$").unwrap());
    let de_self_left = MEMBERSHIP_RE_DE_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) hat die Gruppe verlassen$").unwrap());
    let de_third_add = MEMBERSHIP_RE_DE_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? hat (.+) hinzugefügt$").unwrap());
    let de_third_remove = MEMBERSHIP_RE_DE_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? hat (.+) entfernt$").unwrap());
    let de_you_add = MEMBERSHIP_RE_DE_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Du hast (.+) hinzugefügt$").unwrap());
    let de_you_remove = MEMBERSHIP_RE_DE_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Du hast (.+) entfernt$").unwrap());
    let es_self_left = MEMBERSHIP_RE_ES_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) salió$").unwrap());
    let es_third_add = MEMBERSHIP_RE_ES_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? añadió a (.+)$").unwrap());
    let es_third_remove = MEMBERSHIP_RE_ES_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? eliminó a (.+)$").unwrap());
    let es_you_add = MEMBERSHIP_RE_ES_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Añadiste a (.+)$").unwrap());
    let es_you_remove = MEMBERSHIP_RE_ES_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Eliminaste a (.+)$").unwrap());
    let et_self_left = MEMBERSHIP_RE_ET_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) lahkus$").unwrap());
    let et_third_add = MEMBERSHIP_RE_ET_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? lisas (.+)$").unwrap());
    let et_third_remove = MEMBERSHIP_RE_ET_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? eemaldas (.+)$").unwrap());
    let et_you_add = MEMBERSHIP_RE_ET_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^(.+) lisatud sinu poolt$").unwrap());
    let et_you_remove = MEMBERSHIP_RE_ET_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^(.+) eemaldatud sinu poolt$").unwrap());
    let fi_self_left = MEMBERSHIP_RE_FI_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) poistui$").unwrap());
    let fi_third_add = MEMBERSHIP_RE_FI_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? lisäsi henkilön (.+)$").unwrap());
    let fi_third_remove = MEMBERSHIP_RE_FI_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? poisti henkilön (.+)$").unwrap());
    let fi_you_add = MEMBERSHIP_RE_FI_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Lisäsit henkilön (.+)$").unwrap());
    let fi_you_remove = MEMBERSHIP_RE_FI_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Poistit henkilön (.+)$").unwrap());
    let hr_self_left = MEMBERSHIP_RE_HR_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) izašao$").unwrap());
    let hr_third_add = MEMBERSHIP_RE_HR_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? dodao/la (.+)$").unwrap());
    let hr_third_remove = MEMBERSHIP_RE_HR_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? je uklonio (.+)$").unwrap());
    let hr_you_add = MEMBERSHIP_RE_HR_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Dodali ste (.+)$").unwrap());
    let hr_you_remove = MEMBERSHIP_RE_HR_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Uklonili ste (.+)$").unwrap());
    let hu_self_left = MEMBERSHIP_RE_HU_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) kilépett$").unwrap());
    let hu_third_add = MEMBERSHIP_RE_HU_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? hozzáadta (.+)$").unwrap());
    let hu_third_remove = MEMBERSHIP_RE_HU_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? eltávolította (.+)$").unwrap());
    let hu_you_add = MEMBERSHIP_RE_HU_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^(.+) hozzáadva$").unwrap());
    let hu_you_remove = MEMBERSHIP_RE_HU_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Eltávolítottad őt: (.+)$").unwrap());
    let id_self_left = MEMBERSHIP_RE_ID_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) keluar$").unwrap());
    let id_third_add = MEMBERSHIP_RE_ID_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? menambahkan (.+)$").unwrap());
    let id_third_remove = MEMBERSHIP_RE_ID_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? mengeluarkan (.+)$").unwrap());
    let id_you_add = MEMBERSHIP_RE_ID_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Anda menambahkan (.+)$").unwrap());
    let id_you_remove = MEMBERSHIP_RE_ID_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Anda mengeluarkan (.+)$").unwrap());
    let it_self_left = MEMBERSHIP_RE_IT_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) ha abbandonato$").unwrap());
    let it_third_add = MEMBERSHIP_RE_IT_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? ha aggiunto (.+)$").unwrap());
    let it_third_remove = MEMBERSHIP_RE_IT_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? ha rimosso (.+)$").unwrap());
    let it_you_add = MEMBERSHIP_RE_IT_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Hai aggiunto (.+)$").unwrap());
    let it_you_remove = MEMBERSHIP_RE_IT_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Hai rimosso (.+)$").unwrap());
    let lt_self_left = MEMBERSHIP_RE_LT_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) išėjo$").unwrap());
    let lt_third_add = MEMBERSHIP_RE_LT_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? pridėjo (.+)$").unwrap());
    let lt_third_remove = MEMBERSHIP_RE_LT_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? pašalino (.+)$").unwrap());
    let lt_you_add = MEMBERSHIP_RE_LT_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Jūs pridėjote (.+)$").unwrap());
    let lt_you_remove = MEMBERSHIP_RE_LT_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Jūs pašalinote (.+)$").unwrap());
    let lv_self_left = MEMBERSHIP_RE_LV_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) aizgāja$").unwrap());
    let lv_third_add = MEMBERSHIP_RE_LV_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? pievienoja (.+)$").unwrap());
    let lv_third_remove = MEMBERSHIP_RE_LV_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? noņēma (.+)$").unwrap());
    let lv_you_add = MEMBERSHIP_RE_LV_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Pievienojāt (.+)$").unwrap());
    let lv_you_remove = MEMBERSHIP_RE_LV_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Jūs noņēmāt (.+)$").unwrap());
    let ms_self_left = MEMBERSHIP_RE_MS_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) keluar$").unwrap());
    let ms_third_add = MEMBERSHIP_RE_MS_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? telah menambah (.+)$").unwrap());
    let ms_third_remove = MEMBERSHIP_RE_MS_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? telah membuang (.+)$").unwrap());
    let ms_you_add = MEMBERSHIP_RE_MS_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Anda telah menambah (.+)$").unwrap());
    let ms_you_remove = MEMBERSHIP_RE_MS_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Anda telah membuang (.+)$").unwrap());
    let nb_self_left = MEMBERSHIP_RE_NB_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) forlot gruppen$").unwrap());
    let nb_third_add = MEMBERSHIP_RE_NB_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? la til (.+)$").unwrap());
    let nb_third_remove = MEMBERSHIP_RE_NB_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? fjernet (.+)$").unwrap());
    let nb_you_add = MEMBERSHIP_RE_NB_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Du la til (.+)$").unwrap());
    let nb_you_remove = MEMBERSHIP_RE_NB_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Du fjernet (.+)$").unwrap());
    let pl_self_left = MEMBERSHIP_RE_PL_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) opuścił\(a\)$").unwrap());
    let pl_third_add = MEMBERSHIP_RE_PL_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? dodał\(a\) (.+)$").unwrap());
    let pl_third_remove = MEMBERSHIP_RE_PL_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? usunął\(ęła\) (.+)$").unwrap());
    let pl_you_add = MEMBERSHIP_RE_PL_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Dodałeś\(aś\) (.+)$").unwrap());
    let pl_you_remove = MEMBERSHIP_RE_PL_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Usunąłeś\(ęłaś\) (.+)$").unwrap());
    let pt_self_left = MEMBERSHIP_RE_PT_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) saiu do grupo$").unwrap());
    let pt_third_add = MEMBERSHIP_RE_PT_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? adicionou (.+) a este grupo$").unwrap());
    let pt_third_remove = MEMBERSHIP_RE_PT_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? removeu (.+) deste grupo$").unwrap());
    let pt_you_add = MEMBERSHIP_RE_PT_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Adicionou (.+) a este grupo$").unwrap());
    let pt_you_remove = MEMBERSHIP_RE_PT_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Removeu (.+) deste grupo$").unwrap());
    let ptbr_self_left = MEMBERSHIP_RE_PTBR_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) saiu$").unwrap());
    let ptbr_third_add = MEMBERSHIP_RE_PTBR_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? adicionou (.+)$").unwrap());
    let ptbr_third_remove = MEMBERSHIP_RE_PTBR_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? removeu (.+)$").unwrap());
    let ptbr_you_add = MEMBERSHIP_RE_PTBR_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Você adicionou (.+)$").unwrap());
    let ptbr_you_remove = MEMBERSHIP_RE_PTBR_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Você removeu (.+)$").unwrap());
    let ro_self_left = MEMBERSHIP_RE_RO_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) a ieșit$").unwrap());
    let ro_third_add = MEMBERSHIP_RE_RO_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? a adăugat (.+)$").unwrap());
    let ro_third_remove = MEMBERSHIP_RE_RO_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? a eliminat (.+)$").unwrap());
    let ro_you_add = MEMBERSHIP_RE_RO_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Ați adăugat pe (.+)$").unwrap());
    let ro_you_remove = MEMBERSHIP_RE_RO_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Ați eliminat pe (.+)$").unwrap());
    let sk_self_left = MEMBERSHIP_RE_SK_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) odišiel/a$").unwrap());
    let sk_third_add = MEMBERSHIP_RE_SK_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? pridal/a používateľa (.+)\.$").unwrap());
    let sk_third_remove = MEMBERSHIP_RE_SK_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? odobral/a používateľa (.+)\.$").unwrap());
    let sk_you_add = MEMBERSHIP_RE_SK_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Pridali ste používateľa (.+)\.$").unwrap());
    let sk_you_remove = MEMBERSHIP_RE_SK_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Odobrali ste používateľa (.+)$").unwrap());
    let sl_self_left = MEMBERSHIP_RE_SL_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) je odšel/a$").unwrap());
    let sl_third_add = MEMBERSHIP_RE_SL_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? je dodal/a (.+)$").unwrap());
    let sl_third_remove = MEMBERSHIP_RE_SL_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? je odstranil/a (.+)$").unwrap());
    let sl_you_add = MEMBERSHIP_RE_SL_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Dodali ste (.+)$").unwrap());
    let sl_you_remove = MEMBERSHIP_RE_SL_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Odstranili ste (.+)$").unwrap());
    let sq_self_left = MEMBERSHIP_RE_SQ_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) u largua$").unwrap());
    let sq_third_add = MEMBERSHIP_RE_SQ_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? shtoi (.+)$").unwrap());
    let sq_third_remove = MEMBERSHIP_RE_SQ_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? hoqi (.+)$").unwrap());
    let sq_you_add = MEMBERSHIP_RE_SQ_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Ti ke shtuar (.+)$").unwrap());
    let sq_you_remove = MEMBERSHIP_RE_SQ_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Keni hequr (.+)$").unwrap());
    let sv_self_left = MEMBERSHIP_RE_SV_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) lämnade$").unwrap());
    let sv_third_add = MEMBERSHIP_RE_SV_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? lade till (.+)$").unwrap());
    let sv_third_remove = MEMBERSHIP_RE_SV_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? tog bort (.+)$").unwrap());
    let sv_you_add = MEMBERSHIP_RE_SV_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Du lade till (.+)$").unwrap());
    let sv_you_remove = MEMBERSHIP_RE_SV_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Du tog bort (.+)$").unwrap());
    let sw_self_left = MEMBERSHIP_RE_SW_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) katoka$").unwrap());
    let sw_third_add = MEMBERSHIP_RE_SW_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? amemuongeza (.+)$").unwrap());
    let sw_third_remove = MEMBERSHIP_RE_SW_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? amemuondoa (.+)$").unwrap());
    let sw_you_add = MEMBERSHIP_RE_SW_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Umemuongeza (.+)$").unwrap());
    let sw_you_remove = MEMBERSHIP_RE_SW_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Umemwondoa (.+)$").unwrap());
    let tl_self_left = MEMBERSHIP_RE_TL_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^Umalis si (.+)$").unwrap());
    let tl_third_add = MEMBERSHIP_RE_TL_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? nakapasok (.+)$").unwrap());
    let tl_third_remove = MEMBERSHIP_RE_TL_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Inalis ni .+? si (.+)$").unwrap());
    let tl_you_add = MEMBERSHIP_RE_TL_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Idinagdag mo si (.+)$").unwrap());
    let tl_you_remove = MEMBERSHIP_RE_TL_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Inalis mo si (.+)$").unwrap());
    let tr_self_left = MEMBERSHIP_RE_TR_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) ayrıldı$").unwrap());
    let tr_third_add = MEMBERSHIP_RE_TR_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+?, (.+) kişisini ekledi$").unwrap());
    let tr_third_remove = MEMBERSHIP_RE_TR_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+?, (.+) kişisini çıkardı$").unwrap());
    let tr_you_add = MEMBERSHIP_RE_TR_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^(.+) kişisini eklediniz$").unwrap());
    let tr_you_remove = MEMBERSHIP_RE_TR_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^(.+) kişisini çıkardınız$").unwrap());
    let uz_self_left = MEMBERSHIP_RE_UZ_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) tark etdi$").unwrap());
    let uz_third_add = MEMBERSHIP_RE_UZ_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? (.+)ni qo‘shdi$").unwrap());
    let uz_third_remove = MEMBERSHIP_RE_UZ_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? (.+)ni o‘chirdi$").unwrap());
    let uz_you_add = MEMBERSHIP_RE_UZ_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Siz (.+)ni qo‘shdingiz$").unwrap());
    let uz_you_remove = MEMBERSHIP_RE_UZ_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Siz (.+)ni o‘chirdingiz$").unwrap());
    let vi_self_left = MEMBERSHIP_RE_VI_SELF_LEFT.get_or_init(|| regex::Regex::new(r"(?i)^(.+) đã rời nhóm$").unwrap());
    let vi_third_add = MEMBERSHIP_RE_VI_THIRD_ADD.get_or_init(|| regex::Regex::new(r"(?i)^.+? đã thêm (.+) vào nhóm$").unwrap());
    let vi_third_remove = MEMBERSHIP_RE_VI_THIRD_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^.+? đã bỏ (.+) khỏi nhóm$").unwrap());
    let vi_you_add = MEMBERSHIP_RE_VI_YOU_ADD.get_or_init(|| regex::Regex::new(r"(?i)^Bạn đã thêm (.+) vào nhóm$").unwrap());
    let vi_you_remove = MEMBERSHIP_RE_VI_YOU_REMOVE.get_or_init(|| regex::Regex::new(r"(?i)^Bạn đã bỏ (.+) khỏi nhóm$").unwrap());
    let mut events = Vec::new();
    for raw in messages {
        // Strip WhatsApp's invisible LTR/RTL marks that prefix many system messages.
        let content: String = raw.chars().filter(|c| !matches!(*c,
            '\u{200E}' | '\u{200F}' | '\u{FEFF}' | '\u{200B}'
        )).collect();
        let content = content.trim();
        if content == "Je hebt de groep verlaten"
            || content.eq_ignore_ascii_case("you left")
            || content.eq_ignore_ascii_case("groupe quitté")
            || content.eq_ignore_ascii_case("vous avez quitté le groupe")
            || content.eq_ignore_ascii_case("Siz tərk etdiniz")
            || content.eq_ignore_ascii_case("Has marxat")
            || content.eq_ignore_ascii_case("Opustili jste skupinu.")
            || content.eq_ignore_ascii_case("Du forlod")
            || content.eq_ignore_ascii_case("Du hast die Gruppe verlassen")
            || content.eq_ignore_ascii_case("Saliste")
            || content.eq_ignore_ascii_case("Sa lahkusid")
            || content.eq_ignore_ascii_case("Sinä poistuit")
            || content.eq_ignore_ascii_case("Izašli ste")
            || content.eq_ignore_ascii_case("Kiléptél")
            || content.eq_ignore_ascii_case("Anda keluar")
            || content.eq_ignore_ascii_case("Hai abbandonato")
            || content.eq_ignore_ascii_case("Jūs palikote")
            || content.eq_ignore_ascii_case("Jūs aizgājāt")
            || content.eq_ignore_ascii_case("Anda telah keluar")
            || content.eq_ignore_ascii_case("Du forlot gruppen")
            || content.eq_ignore_ascii_case("Opuściłeś(aś)")
            || content.eq_ignore_ascii_case("Saiu do grupo")
            || content.eq_ignore_ascii_case("Você saiu")
            || content.eq_ignore_ascii_case("Ați ieșit")
            || content.eq_ignore_ascii_case("Opustili ste skupinu.")
            || content.eq_ignore_ascii_case("Odšli ste")
            || content.eq_ignore_ascii_case("U largove")
            || content.eq_ignore_ascii_case("Du lämnade")
            || content.eq_ignore_ascii_case("Umejitoa")
            || content.eq_ignore_ascii_case("Umalis ka")
            || content.eq_ignore_ascii_case("Ayrıldınız")
            || content.eq_ignore_ascii_case("Siz tark etdingiz")
            || content.eq_ignore_ascii_case("Bạn đã rời nhóm")
        {
            continue; // "you" left — no participant name to extract
        }
        if let Some(caps) = self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        } else if let Some(caps) = en_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = en_was_removed.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
            }
        } else if let Some(caps) = en_was_added.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                events.push((normalize_chat_name(&caps[1]), MembershipEvent::Added));
            }
        } else if let Some(caps) = en_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = en_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        } else if let Some(caps) = fr_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = fr_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = fr_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = fr_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = fr_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Azerbaijani
        } else if let Some(caps) = az_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = az_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = az_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = az_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        } else if let Some(caps) = az_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        // Catalan
        } else if let Some(caps) = ca_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = ca_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = ca_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = ca_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = ca_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Czech
        } else if let Some(caps) = cs_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = cs_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = cs_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = cs_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = cs_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Danish
        } else if let Some(caps) = da_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = da_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = da_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = da_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = da_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // German
        } else if let Some(caps) = de_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = de_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = de_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = de_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = de_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Spanish
        } else if let Some(caps) = es_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = es_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = es_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = es_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = es_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Estonian
        } else if let Some(caps) = et_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = et_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = et_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = et_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = et_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Finnish
        } else if let Some(caps) = fi_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = fi_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = fi_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = fi_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = fi_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Croatian
        } else if let Some(caps) = hr_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = hr_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = hr_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = hr_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = hr_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Hungarian
        } else if let Some(caps) = hu_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = hu_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = hu_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = hu_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = hu_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Indonesian
        } else if let Some(caps) = id_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = id_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = id_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = id_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = id_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Italian
        } else if let Some(caps) = it_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = it_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = it_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = it_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = it_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Lithuanian
        } else if let Some(caps) = lt_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = lt_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = lt_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = lt_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = lt_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Latvian
        } else if let Some(caps) = lv_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = lv_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = lv_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = lv_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = lv_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Malay
        } else if let Some(caps) = ms_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = ms_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = ms_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = ms_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = ms_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Norwegian Bokmal
        } else if let Some(caps) = nb_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = nb_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = nb_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = nb_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = nb_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Polish
        } else if let Some(caps) = pl_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = pl_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = pl_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = pl_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = pl_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Portuguese (Portugal)
        } else if let Some(caps) = pt_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = pt_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = pt_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = pt_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = pt_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Portuguese (Brazil)
        } else if let Some(caps) = ptbr_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = ptbr_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = ptbr_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = ptbr_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = ptbr_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Romanian
        } else if let Some(caps) = ro_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = ro_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = ro_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = ro_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = ro_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Slovak
        } else if let Some(caps) = sk_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = sk_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = sk_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = sk_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = sk_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Slovenian
        } else if let Some(caps) = sl_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = sl_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = sl_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = sl_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = sl_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Albanian
        } else if let Some(caps) = sq_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = sq_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = sq_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = sq_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = sq_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Swedish
        } else if let Some(caps) = sv_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = sv_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = sv_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = sv_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = sv_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Swahili
        } else if let Some(caps) = sw_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = sw_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = sw_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = sw_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = sw_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Tagalog
        } else if let Some(caps) = tl_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = tl_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = tl_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = tl_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = tl_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Turkish
        } else if let Some(caps) = tr_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = tr_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = tr_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = tr_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = tr_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Uzbek
        } else if let Some(caps) = uz_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = uz_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = uz_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = uz_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = uz_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        // Vietnamese
        } else if let Some(caps) = vi_self_left.captures(content) {
            events.push((normalize_chat_name(&caps[1]), MembershipEvent::Left));
        } else if let Some(caps) = vi_you_remove.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Left));
                }
            }
        } else if let Some(caps) = vi_you_add.captures(content) {
            for name in split_membership_targets(&caps[1]) {
                if !is_group_attribute_target(&name) {
                    events.push((normalize_chat_name(&name), MembershipEvent::Added));
                }
            }
        } else if let Some(caps) = vi_third_remove.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Left));
                    }
                }
            }
        } else if let Some(caps) = vi_third_add.captures(content) {
            if !is_group_attribute_target(&caps[1]) {
                for name in split_membership_targets(&caps[1]) {
                    if !is_group_attribute_target(&name) {
                        events.push((normalize_chat_name(&name), MembershipEvent::Added));
                    }
                }
            }
        }
    }
    events
}

// Title-cases a normalized (already-lowercased) name for display, e.g. "anouk" -> "Anouk".
// Used only as a fallback label for participants who never sent a real chat message, so there's
// no display-cased sender string to reuse — a phone number like "+31623347487" passes through
// unchanged since it has no lowercase letters to capitalize.
fn title_case(name: &str) -> String {
    name.split(' ')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// Merges message senders with names mentioned in membership events into one deduplicated,
// sorted display list. A sender's own display-cased name always wins over a title-cased fallback
// for the same person (matched case-insensitively via normalize_chat_name), since it's the real
// cased name from the export rather than a guess.
fn merge_group_participants(senders: Vec<String>, events: Vec<(String, MembershipEvent)>) -> Vec<String> {
    let mut by_normalized: HashMap<String, String> = HashMap::new();
    for sender in senders {
        by_normalized.insert(normalize_chat_name(&sender), sender);
    }
    for (normalized, _) in events {
        by_normalized.entry(normalized.clone()).or_insert_with(|| title_case(&normalized));
    }
    let mut result: Vec<String> = by_normalized.into_values().collect();
    result.sort();
    result
}

// Returns every participant ever known in a group chat: people who sent a real message, plus
// people only ever mentioned in "added"/"removed"/"left" system messages (e.g. someone who was
// added and removed before ever sending anything). Without the latter, a group where nobody who
// was added/removed ever posted a real message would show 0 participants and no former-member
// badges, even though the system messages clearly record former members.
#[tauri::command]
fn get_group_participants(chat_id: String) -> Result<Vec<String>, String> {
    let conn = get_db();
    let senders: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT DISTINCT sender FROM messages WHERE chat_id = ?1 AND msg_type != 'system' AND sender != 'System'"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![&chat_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.flatten().collect()
    };
    let system_messages: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT content FROM messages WHERE chat_id = ?1 AND (sender = 'System' OR msg_type = 'system') ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![&chat_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.flatten().collect()
    };
    let events = extract_membership_events(&system_messages);
    Ok(merge_group_participants(senders, events))
}

#[tauri::command]
fn get_former_members(chat_id: String, participant_names: Vec<String>) -> Result<Vec<String>, String> {
    let conn = get_db();
    let messages: Vec<String> = {
        let mut stmt = conn.prepare(
            "SELECT content FROM messages WHERE chat_id = ?1 AND (sender = 'System' OR msg_type = 'system') ORDER BY id"
        ).map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![&chat_id], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.flatten().collect()
    };
    let events = extract_membership_events(&messages);
    let mut last_event: HashMap<String, MembershipEvent> = HashMap::new();
    for (name, event) in events {
        last_event.insert(name, event);
    }
    let former: Vec<String> = participant_names.into_iter()
        .filter(|name| {
            let normalized = normalize_chat_name(name);
            matches!(last_event.get(&normalized), Some(MembershipEvent::Left))
        })
        .collect();
    Ok(former)
}

#[tauri::command]
fn get_linked_chat_for_contact(contact_id: String) -> Result<Option<ChatMeta>, String> {
    let conn = get_db();
    let mut stmt = conn.prepare(
        "SELECT chats.id, COALESCE(profiles.name, chats.name), chats.last_message, chats.timestamp, chats.is_group, chats.zip_path, profiles.photo_path
         FROM chats LEFT JOIN profiles ON chats.id = profiles.chat_id
         WHERE chats.contact_id = ?1"
    ).map_err(|e| e.to_string())?;
    let result = stmt.query_row(params![&contact_id], |row| {
        Ok(ChatMeta {
            id: row.get(0)?,
            name: row.get(1)?,
            last_message: row.get(2)?,
            timestamp: row.get(3)?,
            is_group: row.get::<_, i32>(4)? != 0,
            zip_path: row.get(5)?,
            photo_path: row.get(6)?,
        })
    });
    Ok(result.ok())
}

#[tauri::command]
fn get_unlinked_one_on_one_chats() -> Result<Vec<ChatMeta>, String> {
    let conn = get_db();
    let mut stmt = conn.prepare(
        "SELECT chats.id, COALESCE(profiles.name, chats.name), chats.last_message, chats.timestamp, chats.is_group, chats.zip_path, profiles.photo_path
         FROM chats LEFT JOIN profiles ON chats.id = profiles.chat_id
         WHERE chats.is_group = 0 AND chats.contact_id IS NULL
         ORDER BY chats.last_message_epoch DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
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
    Ok(rows.flatten().collect())
}

#[tauri::command]
fn link_contact_to_chat(contact_id: String, chat_id: String) -> Result<(), String> {
    let conn = get_db();
    link_chat_and_contact(&conn, &chat_id, &contact_id)
}

#[tauri::command]
fn unlink_contact_from_chat(contact_id: String) -> Result<(), String> {
    let conn = get_db();
    conn.execute(
        "UPDATE chats SET contact_id = NULL WHERE contact_id = ?1",
        params![&contact_id],
    ).map_err(|e| e.to_string())?;
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
    // 3. Export favorite messages
    let mut fav_stmt = conn.prepare(
        "SELECT (SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = m1.chat_id AND m2.id <= m1.id) - 1 as msg_idx
         FROM messages m1
         WHERE m1.chat_id = ?1 AND m1.is_favorite = 1
         ORDER BY m1.id ASC"
    ).map_err(|e| e.to_string())?;
    let fav_indices: Vec<i64> = fav_stmt
        .query_map([&chat_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    if !fav_indices.is_empty() {
        all_modifications.push(serde_json::json!({
            "type": "favorites",
            "indices": fav_indices
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
        } else if mod_type == "favorites" {
            // Restore favorite messages by index
            if let Some(indices) = mod_entry.get("indices").and_then(|v| v.as_array()) {
                for idx_val in indices {
                    if let Some(idx) = idx_val.as_i64() {
                        conn.execute(
                            "UPDATE messages SET is_favorite = 1 WHERE chat_id = ?1 AND id = (SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3)",
                            rusqlite::params![&chat_id, &chat_id, idx],
                        ).map_err(|e| e.to_string())?;
                    }
                }
            }
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
    match result {
        None => Ok(None),
        Some(file) => {
            let picked_path_str = file.to_string();
            let picked = std::path::Path::new(&picked_path_str);
            let ext = picked.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_lowercase();
            let bg_dir = get_app_data_dir().join("user_backgrounds");
            fs::create_dir_all(&bg_dir).map_err(|e| e.to_string())?;
            let filename = format!("{}.{}", std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(), ext);
            let dst = bg_dir.join(&filename);
            fs::copy(picked, &dst).map_err(|e| format!("Failed to copy background: {}", e))?;
            Ok(Some(dst.to_string_lossy().to_string()))
        }
    }
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
    let _count: i64 = conn.query_row(
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
    // Export favorites
    let mut fav_stmt = conn.prepare(
        "SELECT (SELECT COUNT(*) FROM messages m2 WHERE m2.chat_id = m1.chat_id AND m2.id <= m1.id) - 1 as msg_idx
         FROM messages m1
         WHERE m1.chat_id = ?1 AND m1.is_favorite = 1
         ORDER BY m1.id ASC"
    ).map_err(|e| e.to_string())?;
    let fav_indices: Vec<i64> = fav_stmt
        .query_map([chat_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    if !fav_indices.is_empty() {
        all_modifications.push(serde_json::json!({
            "type": "favorites",
            "indices": fav_indices
        }));
    }
    // Export file renames
    let mut rename_stmt = conn.prepare(
        "SELECT message_index, original_filename, new_filename FROM chat_file_renames WHERE chat_id = ?1 ORDER BY changed_at ASC"
    ).map_err(|e| e.to_string())?;
    let renames: Vec<(i64, String, String)> = rename_stmt.query_map([chat_id], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    if !renames.is_empty() {
        all_modifications.push(serde_json::json!({
            "type": "file_renames",
            "renames": renames.iter().map(|(idx, orig, new)| serde_json::json!({
                "message_index": idx,
                "original_filename": orig,
                "new_filename": new
            })).collect::<Vec<_>>()
        }));
    }
    // Export name history
    let mut name_history_stmt = conn.prepare(
        "SELECT name, changed_at FROM chat_name_history WHERE chat_id = ?1 ORDER BY changed_at ASC"
    ).map_err(|e| e.to_string())?;
    let name_history: Vec<(String, String)> = name_history_stmt.query_map([chat_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    if !name_history.is_empty() {
        all_modifications.push(serde_json::json!({
            "type": "name_history",
            "entries": name_history.iter().map(|(name, changed_at)| serde_json::json!({
                "name": name,
                "changed_at": changed_at
            })).collect::<Vec<_>>()
        }));
    }
    // Export background history
    let mut bg_history_stmt = conn.prepare(
        "SELECT background_path, changed_at FROM chat_background_history WHERE chat_id = ?1 ORDER BY changed_at ASC"
    ).map_err(|e| e.to_string())?;
    let bg_history: Vec<(String, String)> = bg_history_stmt.query_map([chat_id], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();
    if !bg_history.is_empty() {
        all_modifications.push(serde_json::json!({
            "type": "background_history",
            "entries": bg_history.iter().map(|(path, changed_at)| serde_json::json!({
                "background_path": path,
                "changed_at": changed_at
            })).collect::<Vec<_>>()
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
    let result = tauri::async_runtime::spawn_blocking(move || import_from_export_inner(zip_path)).await.map_err(|e| e.to_string())?;
    if result.is_ok() {
        run_post_import_reconciliation().await;
    }
    result
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
    run_post_import_reconciliation().await;
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
            link_chat_to_contact_if_match(&conn, &existing_id, is_group);
            imported_ids.push(existing_id.clone());
            // Apply modifications if present (merge scenario)
            let mods_path = format!("{}/modifications.json", folder);
            if let Ok(mut file) = archive.by_name(&mods_path) {
                let mut buf = String::new();
                if file.read_to_string(&mut buf).is_ok() && buf != "[]" {
                    let _ = apply_chat_modifications_internal(&mut conn, &existing_id, &buf);
                }
            }
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
        link_chat_to_contact_if_match(&conn, &chat_id, is_group);
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
            "favorites" => {
                // Restore favorite messages by index
                if let Some(indices) = modification["indices"].as_array() {
                    for idx_val in indices {
                        if let Some(idx) = idx_val.as_i64() {
                            let _ = conn.execute(
                                "UPDATE messages SET is_favorite = 1 WHERE chat_id = ?1 AND id = (SELECT id FROM messages WHERE chat_id = ?2 ORDER BY id LIMIT 1 OFFSET ?3)",
                                params![chat_id, chat_id, idx]
                            );
                        }
                    }
                }
            }
            "file_renames" => {
                // Restore file renames
                if let Some(renames) = modification["renames"].as_array() {
                    let app_data = get_app_data_dir();
                    let media_dir = app_data.join("chats").join(chat_id).join("media");
                    for rename in renames {
                        if let (Some(idx), Some(orig), Some(new)) = (
                            rename["message_index"].as_i64(),
                            rename["original_filename"].as_str(),
                            rename["new_filename"].as_str()
                        ) {
                            let old_path = media_dir.join(orig);
                            let new_path = media_dir.join(new);
                            if old_path.exists() && !new_path.exists() {
                                let _ = fs::rename(&old_path, &new_path);
                                // Update database media column
                                let _ = conn.execute(
                                    "UPDATE messages SET media = ?1 WHERE chat_id = ?2 AND id = (SELECT id FROM messages WHERE chat_id = ?3 ORDER BY id LIMIT 1 OFFSET ?4)",
                                    params![new, chat_id, chat_id, idx]
                                );
                                // Record in history
                                let _ = conn.execute(
                                    "INSERT INTO chat_file_renames (chat_id, message_index, original_filename, new_filename) VALUES (?1, ?2, ?3, ?4)",
                                    params![chat_id, idx, orig, new]
                                );
                            }
                        }
                    }
                }
            }
            "name_history" => {
                // Restore name history
                if let Some(entries) = modification["entries"].as_array() {
                    for entry in entries {
                        if let (Some(name), Some(changed_at)) = (
                            entry["name"].as_str(),
                            entry["changed_at"].as_str()
                        ) {
                            let _ = conn.execute(
                                "INSERT INTO chat_name_history (chat_id, name, changed_at) VALUES (?1, ?2, ?3)",
                                params![chat_id, name, changed_at]
                            );
                        }
                    }
                }
            }
            "background_history" => {
                // Restore background history
                if let Some(entries) = modification["entries"].as_array() {
                    for entry in entries {
                        if let (Some(path), Some(changed_at)) = (
                            entry["background_path"].as_str(),
                            entry["changed_at"].as_str()
                        ) {
                            // Remap path to new chat's custom dir
                            let remapped: Option<String> = std::path::Path::new(path).file_name().and_then(|fname| {
                                let new_path = get_app_data_dir().join("chats").join(chat_id).join("custom").join(fname);
                                if new_path.exists() { Some(new_path.to_string_lossy().to_string()) } else { None }
                            });
                            if let Some(ref bg) = remapped {
                                let _ = conn.execute(
                                    "INSERT INTO chat_background_history (chat_id, background_path, changed_at) VALUES (?1, ?2, ?3)",
                                    params![chat_id, bg, changed_at]
                                );
                            }
                        }
                    }
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
            pick_zip_files,
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
            search_messages_across_chats,
            find_message_index_for_date,
            has_pin,
            set_pin,
            change_pin,
            change_recovery_question,
            verify_pin,
            clear_pin,
            get_recovery_question,
            get_recovery_answer,
            reset_pin_with_recovery_answer,
            get_profile,
            update_profile,
            remove_profile_photo,
            get_name_history,
            revert_profile_name,
            pick_profile_photo,
            get_or_create_contact_for_participant,
            take_pending_auto_links,
            update_contact_profile,
            remove_contact_photo,
            get_contact_groups,
            remove_contact_group,
            get_linked_participants,
            get_display_names_for_participants,
            get_group_participants,
            get_former_members,
            get_unlinked_one_on_one_chats,
            get_linked_chat_for_contact,
            link_contact_to_chat,
            unlink_contact_from_chat,
            pick_global_background,
            save_background_from_b64,
            read_file_as_base64,
            list_default_backgrounds,
            set_message_type,
            open_media_file,
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
            CREATE TABLE profiles (
                chat_id TEXT PRIMARY KEY,
                name TEXT
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
    // ==================== find_message_index_for_date_impl ====================
    fn insert_test_message(conn: &rusqlite::Connection, chat_id: &str, timestamp: &str, content: &str) {
        conn.execute(
            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content) VALUES (?1, ?2, 'Alice', 'text', ?3)",
            rusqlite::params![chat_id, timestamp, content],
        ).unwrap();
    }
    #[test]
    fn find_message_index_for_date_impl__returns_first_message_on_or_after_date() {
        let chat_id = "test-chat";
        let conn = in_memory_db_with_chat(chat_id);
        insert_test_message(&conn, chat_id, "15/06/2017 00:00", "before");
        insert_test_message(&conn, chat_id, "17/06/2017 00:04", "on target date");
        insert_test_message(&conn, chat_id, "20/06/2017 00:00", "after");
        let target_epoch = iso_date_to_epoch("2017-06-17", false).unwrap();
        let result = find_message_index_for_date_impl(&conn, chat_id, target_epoch).unwrap();
        assert_eq!(result, Some(1));
    }
    #[test]
    fn find_message_index_for_date_impl__returns_none_when_all_messages_are_before_date() {
        let chat_id = "test-chat";
        let conn = in_memory_db_with_chat(chat_id);
        insert_test_message(&conn, chat_id, "01/01/2017 00:00", "old");
        let target_epoch = iso_date_to_epoch("2020-01-01", false).unwrap();
        let result = find_message_index_for_date_impl(&conn, chat_id, target_epoch).unwrap();
        assert_eq!(result, None);
    }
    #[test]
    fn find_message_index_for_date_impl__returns_index_zero_when_date_before_all_messages() {
        let chat_id = "test-chat";
        let conn = in_memory_db_with_chat(chat_id);
        insert_test_message(&conn, chat_id, "01/01/2020 00:00", "first");
        insert_test_message(&conn, chat_id, "02/01/2020 00:00", "second");
        let target_epoch = iso_date_to_epoch("2010-01-01", false).unwrap();
        let result = find_message_index_for_date_impl(&conn, chat_id, target_epoch).unwrap();
        assert_eq!(result, Some(0));
    }
    #[test]
    fn find_message_index_for_date_impl__unparseable_timestamp_treated_as_epoch_zero() {
        let chat_id = "test-chat";
        let conn = in_memory_db_with_chat(chat_id);
        insert_test_message(&conn, chat_id, "not a date", "garbage timestamp");
        // Target epoch 0 (1970-01-01) matches immediately since unparseable timestamps fall back to epoch 0
        let result = find_message_index_for_date_impl(&conn, chat_id, 0).unwrap();
        assert_eq!(result, Some(0));
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
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(),   msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Charlie".into(),msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(detect_group_chat(&msgs));
    }
    #[test]
    fn detect_group_chat__two_distinct_senders_without_indicator_returns_false() {
        // 2 senders with no group-creation/membership system message is indistinguishable
        // from an ordinary 1-on-1 chat (you + them, both messaging under their real names —
        // real exports never actually label the account owner "You"), so this must NOT be
        // classified as a group. Only an explicit indicator, or 3+ senders, should trigger true.
        let msgs = vec![
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(),   msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(!detect_group_chat(&msgs));
    }
    #[test]
    fn detect_group_chat__system_and_you_senders_are_excluded_from_count() {
        let msgs = vec![
            Message { id: None, sender: "System".into(), msg_type: "system".into(), content: "end-to-end".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "You".into(),    msg_type: "text".into(),   content: "hi".into(),         timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Alice".into(),  msg_type: "text".into(),   content: "hi".into(),         timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(!detect_group_chat(&msgs));
    }
    // ==================== detect_group_chat — indicator phrases (new languages) ====================
    #[test]
    fn detect_group_chat__german_created_group_indicator_returns_true_even_with_two_senders() {
        let msgs = vec![
            Message { id: None, sender: "System".into(), msg_type: "system".into(), content: "Alice hat die Gruppe erstellt".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(detect_group_chat(&msgs));
    }
    #[test]
    fn detect_group_chat__spanish_created_group_indicator_returns_true_even_with_two_senders() {
        let msgs = vec![
            Message { id: None, sender: "System".into(), msg_type: "system".into(), content: "Alice creó el grupo".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(detect_group_chat(&msgs));
    }
    #[test]
    fn detect_group_chat__italian_created_group_indicator_returns_true_even_with_two_senders() {
        let msgs = vec![
            Message { id: None, sender: "System".into(), msg_type: "system".into(), content: "Alice ha creato il gruppo".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(detect_group_chat(&msgs));
    }
    #[test]
    fn detect_group_chat__portuguese_created_group_indicator_returns_true_even_with_two_senders() {
        let msgs = vec![
            Message { id: None, sender: "System".into(), msg_type: "system".into(), content: "Alice criou o grupo".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(detect_group_chat(&msgs));
    }
    #[test]
    fn detect_group_chat__polish_created_group_indicator_returns_true_even_with_two_senders() {
        let msgs = vec![
            Message { id: None, sender: "System".into(), msg_type: "system".into(), content: "Alice utworzył grupę".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Alice".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
            Message { id: None, sender: "Bob".into(), msg_type: "text".into(), content: "hi".into(), timestamp: "".into(), media: None, duration: None, tag_ext: None, display_name: None, is_favorite: None },
        ];
        assert!(detect_group_chat(&msgs));
    }
    // ==================== is_group_attribute_target ====================
    #[test]
    fn is_group_attribute_target__dutch_group_targets_return_true() {
        assert!(is_group_attribute_target("de groep"));
        assert!(is_group_attribute_target("de groepsafbeelding"));
        assert!(is_group_attribute_target("de groepsbeschrijving"));
        assert!(is_group_attribute_target("je"));
    }
    #[test]
    fn is_group_attribute_target__english_group_targets_return_true() {
        assert!(is_group_attribute_target("you"));
        assert!(is_group_attribute_target("the group"));
        assert!(is_group_attribute_target("this group"));
        assert!(is_group_attribute_target("the group icon"));
    }
    #[test]
    fn is_group_attribute_target__french_group_targets_return_true() {
        assert!(is_group_attribute_target("le groupe"));
        assert!(is_group_attribute_target("la description du groupe"));
        assert!(is_group_attribute_target("l'icône du groupe"));
        assert!(is_group_attribute_target("la photo du groupe"));
        assert!(is_group_attribute_target("vous"));
    }
    #[test]
    fn is_group_attribute_target__german_group_targets_return_true() {
        assert!(is_group_attribute_target("die Gruppe"));
        assert!(is_group_attribute_target("die Gruppenbeschreibung"));
        assert!(is_group_attribute_target("dich"));
        assert!(is_group_attribute_target("euch"));
    }
    #[test]
    fn is_group_attribute_target__spanish_group_targets_return_true() {
        assert!(is_group_attribute_target("el grupo"));
        assert!(is_group_attribute_target("el grupo de amigos"));
        assert!(is_group_attribute_target("te"));
    }
    #[test]
    fn is_group_attribute_target__ordinary_person_names_return_false() {
        for name in ["John Smith", "Marie", "Jose Garcia", "Francois Dubois", "Ahmet Yilmaz", "Piet Jansen"] {
            assert!(!is_group_attribute_target(name), "expected {name:?} to not be treated as a group-attribute target");
        }
    }
    // ==================== split_membership_targets ====================
    #[test]
    fn split_membership_targets__dutch_en_separator_splits_two_names() {
        assert_eq!(split_membership_targets("Marie en Piet"), vec!["Marie", "Piet"]);
    }
    #[test]
    fn split_membership_targets__english_and_separator_splits_two_names() {
        assert_eq!(split_membership_targets("Marie and Piet"), vec!["Marie", "Piet"]);
    }
    #[test]
    fn split_membership_targets__french_et_separator_splits_two_names() {
        assert_eq!(split_membership_targets("Marie et Piet"), vec!["Marie", "Piet"]);
    }
    #[test]
    fn split_membership_targets__german_und_separator_splits_two_names() {
        assert_eq!(split_membership_targets("Marie und Piet"), vec!["Marie", "Piet"]);
    }
    #[test]
    fn split_membership_targets__vietnamese_va_separator_splits_two_names() {
        assert_eq!(split_membership_targets("Marie và Piet"), vec!["Marie", "Piet"]);
    }
    #[test]
    fn split_membership_targets__comma_and_and_separator_splits_three_names() {
        assert_eq!(split_membership_targets("Alice, Bob and Charlie"), vec!["Alice", "Bob", "Charlie"]);
    }
    #[test]
    fn split_membership_targets__name_with_capitalized_middle_initial_is_not_split() {
        // The split regex has no case-insensitive flag, so a capitalized middle initial like
        // " A " must not be confused with the lowercase " a " ("and") list separator.
        assert_eq!(split_membership_targets("John A Smith"), vec!["John A Smith"]);
        assert_eq!(split_membership_targets("Mary E Johnson"), vec!["Mary E Johnson"]);
    }
    // ==================== extract_membership_events ====================
    // --- Dutch ---
    #[test]
    fn extract_membership_events__dutch_third_person_add_returns_added_event() {
        let msgs = vec!["John heeft Marie toegevoegd".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__dutch_third_person_remove_returns_left_event() {
        let msgs = vec!["John heeft Marie verwijderd".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__dutch_self_left_returns_left_event() {
        let msgs = vec!["John heeft de groep verlaten".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- English ---
    #[test]
    fn extract_membership_events__english_third_person_add_returns_added_event() {
        let msgs = vec!["John added Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__english_third_person_remove_returns_left_event() {
        let msgs = vec!["John removed Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__english_self_left_returns_left_event() {
        let msgs = vec!["John left".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- French ---
    #[test]
    fn extract_membership_events__french_third_person_add_returns_added_event() {
        let msgs = vec!["John a ajouté Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__french_third_person_remove_returns_left_event() {
        let msgs = vec!["John a retiré Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__french_self_left_returns_left_event() {
        let msgs = vec!["John est parti".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Azerbaijani ---
    #[test]
    fn extract_membership_events__azerbaijani_third_person_add_returns_added_event() {
        let msgs = vec!["John əlavə etdi: Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    // Real string is target-before-actor: "{target} {actor} tərəfindən çıxarıldı".
    #[test]
    fn extract_membership_events__azerbaijani_third_person_remove_returns_left_event() {
        let msgs = vec!["Marie John tərəfindən çıxarıldı".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__azerbaijani_self_left_returns_left_event() {
        let msgs = vec!["John tərk etdi".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Catalan ---
    #[test]
    fn extract_membership_events__catalan_third_person_add_returns_added_event() {
        let msgs = vec!["John ha afegit Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__catalan_third_person_remove_returns_left_event() {
        let msgs = vec!["John ha expulsat a Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__catalan_self_left_returns_left_event() {
        let msgs = vec!["John marxa".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Czech ---
    #[test]
    fn extract_membership_events__czech_third_person_add_returns_added_event() {
        let msgs = vec!["John přidal/a uživatele Marie.".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__czech_third_person_remove_returns_left_event() {
        let msgs = vec!["John odebral(a) uživatele Marie.".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__czech_self_left_returns_left_event() {
        let msgs = vec!["John odešel/a".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Danish ---
    #[test]
    fn extract_membership_events__danish_third_person_add_returns_added_event() {
        let msgs = vec!["John tilføjede Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__danish_third_person_remove_returns_left_event() {
        let msgs = vec!["John fjernede Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__danish_self_left_returns_left_event() {
        let msgs = vec!["John forlod".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- German ---
    #[test]
    fn extract_membership_events__german_third_person_add_returns_added_event() {
        let msgs = vec!["John hat Marie hinzugefügt".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__german_third_person_remove_returns_left_event() {
        let msgs = vec!["John hat Marie entfernt".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__german_self_left_returns_left_event() {
        let msgs = vec!["John hat die Gruppe verlassen".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Spanish ---
    #[test]
    fn extract_membership_events__spanish_third_person_add_returns_added_event() {
        let msgs = vec!["John añadió a Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__spanish_third_person_remove_returns_left_event() {
        let msgs = vec!["John eliminó a Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__spanish_self_left_returns_left_event() {
        let msgs = vec!["John salió".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Estonian ---
    #[test]
    fn extract_membership_events__estonian_third_person_add_returns_added_event() {
        let msgs = vec!["John lisas Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__estonian_third_person_remove_returns_left_event() {
        let msgs = vec!["John eemaldas Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__estonian_self_left_returns_left_event() {
        let msgs = vec!["John lahkus".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Finnish ---
    #[test]
    fn extract_membership_events__finnish_third_person_add_returns_added_event() {
        let msgs = vec!["John lisäsi henkilön Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__finnish_third_person_remove_returns_left_event() {
        let msgs = vec!["John poisti henkilön Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__finnish_self_left_returns_left_event() {
        let msgs = vec!["John poistui".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Croatian ---
    #[test]
    fn extract_membership_events__croatian_third_person_add_returns_added_event() {
        let msgs = vec!["John dodao/la Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__croatian_third_person_remove_returns_left_event() {
        let msgs = vec!["John je uklonio Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__croatian_self_left_returns_left_event() {
        let msgs = vec!["John izašao".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Hungarian ---
    #[test]
    fn extract_membership_events__hungarian_third_person_add_returns_added_event() {
        let msgs = vec!["John hozzáadta Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__hungarian_third_person_remove_returns_left_event() {
        let msgs = vec!["John eltávolította Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__hungarian_self_left_returns_left_event() {
        let msgs = vec!["John kilépett".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Indonesian ---
    #[test]
    fn extract_membership_events__indonesian_third_person_add_returns_added_event() {
        let msgs = vec!["John menambahkan Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__indonesian_third_person_remove_returns_left_event() {
        let msgs = vec!["John mengeluarkan Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__indonesian_self_left_returns_left_event() {
        let msgs = vec!["John keluar".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Italian ---
    #[test]
    fn extract_membership_events__italian_third_person_add_returns_added_event() {
        let msgs = vec!["John ha aggiunto Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__italian_third_person_remove_returns_left_event() {
        let msgs = vec!["John ha rimosso Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__italian_self_left_returns_left_event() {
        let msgs = vec!["John ha abbandonato".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Lithuanian ---
    #[test]
    fn extract_membership_events__lithuanian_third_person_add_returns_added_event() {
        let msgs = vec!["John pridėjo Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__lithuanian_third_person_remove_returns_left_event() {
        let msgs = vec!["John pašalino Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__lithuanian_self_left_returns_left_event() {
        let msgs = vec!["John išėjo".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Latvian ---
    #[test]
    fn extract_membership_events__latvian_third_person_add_returns_added_event() {
        let msgs = vec!["John pievienoja Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__latvian_third_person_remove_returns_left_event() {
        let msgs = vec!["John noņēma Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__latvian_self_left_returns_left_event() {
        let msgs = vec!["John aizgāja".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Malay ---
    #[test]
    fn extract_membership_events__malay_third_person_add_returns_added_event() {
        let msgs = vec!["John telah menambah Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__malay_third_person_remove_returns_left_event() {
        let msgs = vec!["John telah membuang Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__malay_self_left_returns_left_event() {
        let msgs = vec!["John keluar".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Norwegian Bokmal ---
    #[test]
    fn extract_membership_events__norwegian_bokmal_third_person_add_returns_added_event() {
        let msgs = vec!["John la til Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__norwegian_bokmal_third_person_remove_returns_left_event() {
        let msgs = vec!["John fjernet Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__norwegian_bokmal_self_left_returns_left_event() {
        let msgs = vec!["John forlot gruppen".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Polish ---
    #[test]
    fn extract_membership_events__polish_third_person_add_returns_added_event() {
        let msgs = vec!["John dodał(a) Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__polish_third_person_remove_returns_left_event() {
        let msgs = vec!["John usunął(ęła) Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__polish_self_left_returns_left_event() {
        let msgs = vec!["John opuścił(a)".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Portuguese (Portugal) ---
    #[test]
    fn extract_membership_events__portuguese_third_person_add_returns_added_event() {
        let msgs = vec!["John adicionou Marie a este grupo".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__portuguese_third_person_remove_returns_left_event() {
        let msgs = vec!["John removeu Marie deste grupo".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__portuguese_self_left_returns_left_event() {
        let msgs = vec!["John saiu do grupo".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Portuguese (Brazil) ---
    #[test]
    fn extract_membership_events__portuguese_brazil_third_person_add_returns_added_event() {
        let msgs = vec!["John adicionou Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__portuguese_brazil_third_person_remove_returns_left_event() {
        let msgs = vec!["John removeu Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__portuguese_brazil_self_left_returns_left_event() {
        let msgs = vec!["John saiu".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Romanian ---
    #[test]
    fn extract_membership_events__romanian_third_person_add_returns_added_event() {
        let msgs = vec!["John a adăugat Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__romanian_third_person_remove_returns_left_event() {
        let msgs = vec!["John a eliminat Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__romanian_self_left_returns_left_event() {
        let msgs = vec!["John a ieșit".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Slovak ---
    #[test]
    fn extract_membership_events__slovak_third_person_add_returns_added_event() {
        let msgs = vec!["John pridal/a používateľa Marie.".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__slovak_third_person_remove_returns_left_event() {
        let msgs = vec!["John odobral/a používateľa Marie.".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__slovak_self_left_returns_left_event() {
        let msgs = vec!["John odišiel/a".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Slovenian ---
    #[test]
    fn extract_membership_events__slovenian_third_person_add_returns_added_event() {
        let msgs = vec!["John je dodal/a Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__slovenian_third_person_remove_returns_left_event() {
        let msgs = vec!["John je odstranil/a Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__slovenian_self_left_returns_left_event() {
        let msgs = vec!["John je odšel/a".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Albanian ---
    #[test]
    fn extract_membership_events__albanian_third_person_add_returns_added_event() {
        let msgs = vec!["John shtoi Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__albanian_third_person_remove_returns_left_event() {
        let msgs = vec!["John hoqi Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__albanian_self_left_returns_left_event() {
        let msgs = vec!["John u largua".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Swedish ---
    #[test]
    fn extract_membership_events__swedish_third_person_add_returns_added_event() {
        let msgs = vec!["John lade till Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__swedish_third_person_remove_returns_left_event() {
        let msgs = vec!["John tog bort Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__swedish_self_left_returns_left_event() {
        let msgs = vec!["John lämnade".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Swahili ---
    #[test]
    fn extract_membership_events__swahili_third_person_add_returns_added_event() {
        let msgs = vec!["John amemuongeza Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__swahili_third_person_remove_returns_left_event() {
        let msgs = vec!["John amemuondoa Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__swahili_self_left_returns_left_event() {
        let msgs = vec!["John katoka".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Tagalog ---
    #[test]
    fn extract_membership_events__tagalog_third_person_add_returns_added_event() {
        let msgs = vec!["John nakapasok Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__tagalog_third_person_remove_returns_left_event() {
        let msgs = vec!["Inalis ni John si Marie".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__tagalog_self_left_returns_left_event() {
        let msgs = vec!["Umalis si John".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Turkish ---
    #[test]
    fn extract_membership_events__turkish_third_person_add_returns_added_event() {
        let msgs = vec!["John, Marie kişisini ekledi".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__turkish_third_person_remove_returns_left_event() {
        let msgs = vec!["John, Marie kişisini çıkardı".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__turkish_self_left_returns_left_event() {
        let msgs = vec!["John ayrıldı".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Uzbek ---
    #[test]
    fn extract_membership_events__uzbek_third_person_add_returns_added_event() {
        let msgs = vec!["John Marieni qo‘shdi".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__uzbek_third_person_remove_returns_left_event() {
        let msgs = vec!["John Marieni o‘chirdi".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__uzbek_self_left_returns_left_event() {
        let msgs = vec!["John tark etdi".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
    }
    // --- Vietnamese ---
    #[test]
    fn extract_membership_events__vietnamese_third_person_add_returns_added_event() {
        let msgs = vec!["John đã thêm Marie vào nhóm".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Added)]);
    }
    #[test]
    fn extract_membership_events__vietnamese_third_person_remove_returns_left_event() {
        let msgs = vec!["John đã bỏ Marie khỏi nhóm".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("marie".to_string(), MembershipEvent::Left)]);
    }
    #[test]
    fn extract_membership_events__vietnamese_self_left_returns_left_event() {
        let msgs = vec!["John đã rời nhóm".to_string()];
        assert_eq!(extract_membership_events(&msgs), vec![("john".to_string(), MembershipEvent::Left)]);
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
    fn parse_chat_text__azerbaijani_added_message_with_colon_is_still_typed_as_system() {
        let chat = "12/04/2024, 14:32 - John əlavə etdi: Marie\n";
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
    // ==================== search_messages_across_chats_impl ====================
    fn insert_second_test_chat(conn: &rusqlite::Connection, chat_id: &str, name: &str) {
        conn.execute(
            "INSERT INTO chats (id, name, original_name) VALUES (?1, ?2, ?2)",
            rusqlite::params![chat_id, name],
        ).unwrap();
    }
    #[test]
    fn search_messages_across_chats_impl__matches_across_multiple_chats_returns_results_from_each() {
        let conn = in_memory_db_with_chat("chat-a");
        insert_second_test_chat(&conn, "chat-b", "Chat B");
        insert_test_message(&conn, "chat-a", "01/01/2024 00:00", "hello banana world");
        insert_test_message(&conn, "chat-b", "02/01/2024 00:00", "another banana here");
        let results = search_messages_across_chats_impl(&conn, "%banana%", 200).unwrap();
        assert_eq!(results.len(), 2);
        let chat_ids: std::collections::HashSet<_> = results.iter().map(|r| r.chat_id.as_str()).collect();
        assert!(chat_ids.contains("chat-a"));
        assert!(chat_ids.contains("chat-b"));
    }
    #[test]
    fn search_messages_across_chats_impl__respects_limit_caps_total_results() {
        let conn = in_memory_db_with_chat("chat-a");
        for _ in 0..5 {
            insert_test_message(&conn, "chat-a", "01/01/2024 00:00", "banana");
        }
        let results = search_messages_across_chats_impl(&conn, "%banana%", 3).unwrap();
        assert_eq!(results.len(), 3);
    }
    #[test]
    fn search_messages_across_chats_impl__excludes_system_messages() {
        let conn = in_memory_db_with_chat("chat-a");
        conn.execute(
            "INSERT INTO messages (chat_id, timestamp, sender, msg_type, content) VALUES ('chat-a', '01/01/2024 00:00', 'System', 'system', 'banana system message')",
            [],
        ).unwrap();
        insert_test_message(&conn, "chat-a", "01/01/2024 00:00", "banana text message");
        let results = search_messages_across_chats_impl(&conn, "%banana%", 200).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].msg_type, "text");
    }
    #[test]
    fn search_messages_across_chats_impl__resolves_profile_override_name_over_chat_name() {
        let conn = in_memory_db_with_chat("chat-a");
        conn.execute(
            "INSERT INTO profiles (chat_id, name) VALUES ('chat-a', 'Custom Name')",
            [],
        ).unwrap();
        insert_test_message(&conn, "chat-a", "01/01/2024 00:00", "banana");
        let results = search_messages_across_chats_impl(&conn, "%banana%", 200).unwrap();
        assert_eq!(results[0].chat_name, "Custom Name");
    }
    #[test]
    fn search_messages_across_chats_impl__message_index_is_position_within_its_own_chat_not_global() {
        let conn = in_memory_db_with_chat("chat-a");
        insert_second_test_chat(&conn, "chat-b", "Chat B");
        insert_test_message(&conn, "chat-a", "01/01/2024 00:00", "first in a");
        insert_test_message(&conn, "chat-a", "02/01/2024 00:00", "second in a, banana");
        insert_test_message(&conn, "chat-b", "01/01/2024 00:00", "first in b, banana");
        let results = search_messages_across_chats_impl(&conn, "%banana%", 200).unwrap();
        let a_result = results.iter().find(|r| r.chat_id == "chat-a").unwrap();
        let b_result = results.iter().find(|r| r.chat_id == "chat-b").unwrap();
        assert_eq!(a_result.message_index, 1, "second message in chat-a should have local index 1");
        assert_eq!(b_result.message_index, 0, "first message in chat-b should have local index 0");
    }
    // ==================== PIN lock (hash_secret / normalize_recovery_answer / app_settings) ====================
    fn in_memory_db_with_settings_table() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT);").unwrap();
        conn
    }
    #[test]
    fn hash_secret__correct_value_verifies_successfully() {
        let hash = hash_secret("1234").unwrap();
        assert!(verify_secret_hash("1234", &hash).unwrap());
    }
    #[test]
    fn hash_secret__wrong_value_fails_verification() {
        let hash = hash_secret("1234").unwrap();
        assert!(!verify_secret_hash("9999", &hash).unwrap());
    }
    #[test]
    fn hash_secret__same_value_hashed_twice_produces_different_hashes() {
        let hash_a = hash_secret("1234").unwrap();
        let hash_b = hash_secret("1234").unwrap();
        assert_ne!(hash_a, hash_b, "each hash should embed a fresh random salt");
    }
    #[test]
    fn normalize_recovery_answer__trims_and_lowercases() {
        assert_eq!(normalize_recovery_answer("  Blue  "), "blue");
    }
    #[test]
    fn normalize_recovery_answer__differently_cased_answers_match_after_hashing() {
        let hash = hash_secret(&normalize_recovery_answer("Rex")).unwrap();
        assert!(verify_secret_hash(&normalize_recovery_answer("  rex "), &hash).unwrap());
    }
    #[test]
    fn set_app_setting__upsert_overwrites_existing_value_for_same_key() {
        let conn = in_memory_db_with_settings_table();
        set_app_setting(&conn, "pin_hash", "hash-a").unwrap();
        set_app_setting(&conn, "pin_hash", "hash-b").unwrap();
        assert_eq!(get_app_setting(&conn, "pin_hash").unwrap(), Some("hash-b".to_string()));
    }
    #[test]
    fn app_settings__delete_removes_key_get_returns_none() {
        let conn = in_memory_db_with_settings_table();
        set_app_setting(&conn, "pin_hash", "hash-a").unwrap();
        delete_app_setting(&conn, "pin_hash").unwrap();
        assert_eq!(get_app_setting(&conn, "pin_hash").unwrap(), None);
    }
    // ==================== merge_messages_into_chat (dedup) ====================
    #[test]
    fn merge_messages_into_chat__duplicate_timestamp_and_sender_pair_is_skipped_not_inserted() {
        let chat_id = "test-chat-1";
        let mut conn = in_memory_db_with_chat(chat_id);
        let existing = Message {
            id: None,
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
            id: None,
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
            id: None,
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
    // ==================== sanitize_filename ====================
    #[test]
    fn sanitize_filename__plain_filename_is_returned_unchanged() {
        assert_eq!(sanitize_filename("photo.jpg").unwrap(), "photo.jpg");
    }
    #[test]
    fn sanitize_filename__path_traversal_with_dotdot_strips_to_basename() {
        assert_eq!(sanitize_filename("../../etc/passwd").unwrap(), "passwd");
    }
    #[test]
    fn sanitize_filename__windows_path_strips_to_basename() {
        assert_eq!(sanitize_filename("C:\\Users\\foo\\secret.txt").unwrap(), "secret.txt");
    }
    #[test]
    fn sanitize_filename__unix_nested_path_strips_to_basename() {
        assert_eq!(sanitize_filename("/var/data/file.png").unwrap(), "file.png");
    }
    #[test]
    fn sanitize_filename__dot_only_name_returns_error() {
        assert!(sanitize_filename(".").is_err());
        assert!(sanitize_filename("..").is_err());
    }
    #[test]
    fn sanitize_filename__empty_string_returns_error() {
        assert!(sanitize_filename("").is_err());
    }
    // ==================== validate_chat_id ====================
    #[test]
    fn validate_chat_id__uuid_style_id_is_accepted() {
        assert!(validate_chat_id("550e8400-e29b-41d4-a716-446655440000").is_ok());
    }
    #[test]
    fn validate_chat_id__empty_string_is_rejected() {
        assert!(validate_chat_id("").is_err());
    }
    #[test]
    fn validate_chat_id__forward_slash_in_id_is_rejected() {
        assert!(validate_chat_id("chat/secret").is_err());
    }
    #[test]
    fn validate_chat_id__backslash_in_id_is_rejected() {
        assert!(validate_chat_id("chat\\secret").is_err());
    }
    #[test]
    fn validate_chat_id__dotdot_in_id_is_rejected() {
        assert!(validate_chat_id("../etc/passwd").is_err());
    }
    #[test]
    fn validate_chat_id__plain_alphanumeric_id_is_accepted() {
        assert!(validate_chat_id("abc123def456").is_ok());
    }
    // ==================== is_safe_open_extension ====================
    #[test]
    fn is_safe_open_extension__common_image_extensions_are_safe() {
        for ext in &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif", "avif"] {
            assert!(is_safe_open_extension(ext), "{} should be safe", ext);
        }
    }
    #[test]
    fn is_safe_open_extension__common_video_extensions_are_safe() {
        for ext in &["mp4", "mkv", "mov", "avi", "webm", "m4v", "3gp"] {
            assert!(is_safe_open_extension(ext), "{} should be safe", ext);
        }
    }
    #[test]
    fn is_safe_open_extension__common_audio_extensions_are_safe() {
        for ext in &["mp3", "m4a", "aac", "ogg", "opus", "flac", "wav"] {
            assert!(is_safe_open_extension(ext), "{} should be safe", ext);
        }
    }
    #[test]
    fn is_safe_open_extension__pdf_and_vcf_and_ico_are_safe() {
        assert!(is_safe_open_extension("pdf"));
        assert!(is_safe_open_extension("vcf"));
        assert!(is_safe_open_extension("ico"));
    }
    #[test]
    fn is_safe_open_extension__executable_extensions_are_not_safe() {
        for ext in &["exe", "bat", "cmd", "sh", "ps1", "msi", "dll", "com"] {
            assert!(!is_safe_open_extension(ext), "{} should NOT be safe", ext);
        }
    }
    #[test]
    fn is_safe_open_extension__script_extensions_are_not_safe() {
        for ext in &["js", "vbs", "py", "rb", "pl", "php", "jar"] {
            assert!(!is_safe_open_extension(ext), "{} should NOT be safe", ext);
        }
    }
    #[test]
    fn is_safe_open_extension__uppercase_extension_is_not_safe_because_caller_must_lowercase() {
        // The function expects the caller to pass a lowercase extension
        assert!(!is_safe_open_extension("JPG"));
    }
    // ==================== validate_url_scheme ====================
    #[test]
    fn validate_url_scheme__https_url_is_accepted() {
        assert!(validate_url_scheme("https://example.com/path").is_ok());
    }
    #[test]
    fn validate_url_scheme__http_url_is_accepted() {
        assert!(validate_url_scheme("http://example.com").is_ok());
    }
    #[test]
    fn validate_url_scheme__https_url_with_uppercase_letters_is_accepted() {
        assert!(validate_url_scheme("HTTPS://example.com").is_ok());
    }
    #[test]
    fn validate_url_scheme__javascript_scheme_is_rejected() {
        assert!(validate_url_scheme("javascript:alert(1)").is_err());
    }
    #[test]
    fn validate_url_scheme__file_scheme_is_rejected() {
        assert!(validate_url_scheme("file:///etc/passwd").is_err());
    }
    #[test]
    fn validate_url_scheme__data_scheme_is_rejected() {
        assert!(validate_url_scheme("data:text/html,<script>alert(1)</script>").is_err());
    }
    #[test]
    fn validate_url_scheme__empty_string_is_rejected() {
        assert!(validate_url_scheme("").is_err());
    }
    #[test]
    fn validate_url_scheme__plain_text_no_scheme_is_rejected() {
        assert!(validate_url_scheme("example.com").is_err());
    }
    // ==================== title_case ====================
    #[test]
    fn title_case__lowercase_name_gets_capitalized() {
        assert_eq!(title_case("anouk"), "Anouk");
    }
    #[test]
    fn title_case__multi_word_name_capitalizes_each_word() {
        assert_eq!(title_case("john smith"), "John Smith");
    }
    #[test]
    fn title_case__phone_number_passes_through_unchanged() {
        assert_eq!(title_case("+31623347487"), "+31623347487");
    }
    // ==================== merge_group_participants ====================
    #[test]
    fn merge_group_participants__person_only_in_events_is_included() {
        let senders = vec!["Alice".to_string()];
        let events = vec![("bob".to_string(), MembershipEvent::Left)];
        assert_eq!(merge_group_participants(senders, events), vec!["Alice".to_string(), "Bob".to_string()]);
    }
    #[test]
    fn merge_group_participants__sender_cased_name_wins_over_title_cased_fallback() {
        let senders = vec!["AnOuk".to_string()];
        let events = vec![("anouk".to_string(), MembershipEvent::Left)];
        assert_eq!(merge_group_participants(senders, events), vec!["AnOuk".to_string()]);
    }
    #[test]
    fn merge_group_participants__no_senders_still_returns_event_only_names() {
        // The bug this guards against: a group where every message is a system message (nobody
        // ever sent a real chat message) must still surface former members, not an empty list.
        let senders: Vec<String> = vec![];
        let events = vec![
            ("anouk".to_string(), MembershipEvent::Left),
            ("gerard".to_string(), MembershipEvent::Left),
        ];
        assert_eq!(merge_group_participants(senders, events), vec!["Anouk".to_string(), "Gerard".to_string()]);
    }
}
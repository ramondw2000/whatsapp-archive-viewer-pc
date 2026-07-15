/**
 * Core types and constants for WhatsApp Archive Viewer
 */

export interface Message {
  id?: number | null;
  timestamp: string;
  sender: string;
  type: string;
  content: string;
  media: string | null;
  duration: string | null;
  tag_ext: string | null;
  caption: string | null;
  hideControls: boolean;
  display_name: string | null;
  is_favorite?: boolean;
}

export interface SearchFilters {
  query: string;
  date_from: string | null;
  date_to: string | null;
  sender: string | null;
  msg_type: string | null;
}

export interface ChatMeta {
  id: string;
  name: string;
  last_message: string;
  timestamp: string;
  is_group: boolean;
  photo_path: string | null;
}

export interface ChatData {
  messages: Message[];
}

export interface SearchResult {
  message_index: number;
  timestamp: string;
  sender: string;
  content: string;
  msg_type: string;
}

export interface Profile {
  chat_id: string;
  name: string | null;
  notes: string | null;
  photo_path: string | null;
  phone_number: string | null;
  original_name: string | null;
  contact_id: string | null;
}

export interface Contact {
  id: string;
  normalized_key: string;
  display_key: string;
  name: string | null;
  notes: string | null;
  photo_path: string | null;
  phone_number: string | null;
}

export interface AutoLinkEvent {
  contact_id: string;
  contact_name: string;
  chat_id: string;
  chat_name: string;
  via_group: string | null;
}

export interface NameHistoryEntry {
  id: number;
  name: string;
  changed_at: string;
}

export interface BackgroundHistoryEntry {
  id: number;
  background_path: string;
  changed_at: string;
}

export type SortOrder = "newest" | "oldest";

// Extension sets for media type detection
export const IMAGE_EXTS = new Set([
  "jpg", "jpeg", "png", "gif", "webp", "bmp", "heic", "heif", "svg", "tif", "tiff", "avif", "jfif", "ico"
]);

export const VIDEO_EXTS = new Set([
  "mp4", "mov", "avi", "mkv", "webm", "m4v", "ts", "flv", "wmv"
]);

export const AUDIO_EXTS = new Set([
  "mp3", "ogg", "opus", "wav", "m4a", "aac", "3gp", "3gpp", "amr", "flac", "wma"
]);

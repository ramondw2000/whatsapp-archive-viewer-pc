import { useCallback, useEffect, useState } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

// ============================================================================
// Media loading helpers
//
// Media files that already live on disk are loaded through Tauri's asset
// protocol (convertFileSrc) so the webview streams/decodes them natively
// instead of receiving multi-megabyte base64 strings over IPC. Anything the
// browser can't render natively (TIFF, HEIC, extensionless files), or a file
// that is missing on disk, falls back to the `get_media_as_base64` command,
// which also extracts missing files from the source archive and converts
// TIFF -> PNG.
// ============================================================================

/** Same characters the Rust side strips from WhatsApp filenames. */
const INVISIBLE_MARKS = /[‎‏‪-‮⁦-⁩﻿​]/g;

export function cleanMediaFilename(filename: string): string {
  return filename.replace(INVISIBLE_MARKS, "");
}

/** Extensions we hand straight to the browser through the asset protocol. */
const ASSET_EXTS = new Set([
  // images
  "jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "avif", "ico",
  // video
  "mp4", "m4v", "mov", "webm", "mkv", "avi", "3gp", "3gpp", "ts",
  // audio
  "mp3", "m4a", "aac", "ogg", "opus", "wav", "flac", "amr",
]);

function extOf(name: string): string {
  const i = name.lastIndexOf(".");
  return i > 0 && i < name.length - 1 ? name.slice(i + 1).toLowerCase() : "";
}

/** Basename as the Rust side computes it (`sanitize_filename`). */
function baseName(name: string): string {
  const parts = name.split(/[\\/]/);
  return parts[parts.length - 1] ?? "";
}

/** True when the file can be loaded straight from disk by the webview. */
export function canUseAssetUrl(filename: string): boolean {
  const clean = baseName(cleanMediaFilename(filename));
  if (!clean || /^\.+$/.test(clean)) return false;
  return ASSET_EXTS.has(extOf(clean));
}

/** Join the media folder path (as returned by Rust) and a filename. */
export function joinMediaPath(baseDir: string, filename: string): string {
  const sep = baseDir.includes("\\") && !baseDir.includes("/") ? "\\" : "/";
  const trimmed = baseDir.replace(/[\\/]+$/, "");
  return `${trimmed}${sep}${baseName(cleanMediaFilename(filename))}`;
}

// ---- media base dir (per chat) ---------------------------------------------

const baseDirCache = new Map<string, string>();
const baseDirPending = new Map<string, Promise<string>>();

export function getCachedMediaBaseDir(chatId: string): string | undefined {
  return baseDirCache.get(chatId);
}

export function getMediaBaseDir(chatId: string): Promise<string> {
  const cached = baseDirCache.get(chatId);
  if (cached !== undefined) return Promise.resolve(cached);
  let p = baseDirPending.get(chatId);
  if (!p) {
    p = invoke<string>("get_media_base_dir", { chatId })
      .then((dir) => {
        baseDirCache.set(chatId, dir);
        return dir;
      })
      .finally(() => baseDirPending.delete(chatId));
    baseDirPending.set(chatId, p);
  }
  return p;
}

/** Build the asset URL for a media file when the base dir is already known. */
export function getCachedAssetUrl(chatId: string, filename: string): string | null {
  const dir = baseDirCache.get(chatId);
  if (dir === undefined || !canUseAssetUrl(filename)) return null;
  return convertFileSrc(joinMediaPath(dir, filename));
}

// ---- dimensions ------------------------------------------------------------

export interface MediaDims {
  width: number;
  height: number;
}

const dimsCache = new Map<string, MediaDims>();
const dimsPending = new Map<string, Promise<MediaDims | null>>();
const DIMS_CACHE_MAX = 20000;

export function getCachedMediaDims(chatId: string, filename: string): MediaDims | null {
  return dimsCache.get(`${chatId}:${filename}`) ?? null;
}

export function rememberMediaDims(chatId: string, filename: string, dims: MediaDims) {
  if (dims.width <= 0 || dims.height <= 0) return;
  if (dimsCache.size >= DIMS_CACHE_MAX) dimsCache.clear();
  dimsCache.set(`${chatId}:${filename}`, dims);
}

/** Header-only dimension lookup (no media data crosses IPC). */
export function fetchMediaDims(chatId: string, filename: string): Promise<MediaDims | null> {
  const key = `${chatId}:${filename}`;
  const cached = dimsCache.get(key);
  if (cached) return Promise.resolve(cached);
  let p = dimsPending.get(key);
  if (!p) {
    p = invoke<MediaDims>("get_media_dims", { chatId, filename: cleanMediaFilename(filename) })
      .then((d) => {
        if (d && d.width > 0 && d.height > 0) {
          rememberMediaDims(chatId, filename, d);
          return { width: d.width, height: d.height };
        }
        return null;
      })
      .catch(() => null)
      .finally(() => dimsPending.delete(key));
    dimsPending.set(key, p);
  }
  return p;
}

/** Cached dimensions of a media file; fetches the header once when `enabled`. */
export function useMediaDims(chatId: string, filename: string, enabled = true): MediaDims | null {
  const [, force] = useState(0);
  const cached = getCachedMediaDims(chatId, filename);
  useEffect(() => {
    if (!enabled || !chatId || !filename || getCachedMediaDims(chatId, filename)) return;
    let cancelled = false;
    fetchMediaDims(chatId, filename).then((d) => {
      if (d && !cancelled) force((n) => n + 1);
    });
    return () => { cancelled = true; };
  }, [chatId, filename, enabled]);
  return cached;
}

// ---- hook ------------------------------------------------------------------

type Mode = "asset" | "invoke";

interface SourceState {
  key: string;
  mode: Mode;
  fallbackSrc: string | null;
  error: string | null;
  existsInZip: boolean;
}

function initialState(key: string, filename: string): SourceState {
  return {
    key,
    mode: canUseAssetUrl(filename) ? "asset" : "invoke",
    fallbackSrc: null,
    error: null,
    existsInZip: false,
  };
}

export interface MediaSource {
  /** URL to put in src, or null while it is still being resolved. */
  src: string | null;
  /** Set when the file could not be loaded by any route. */
  error: string | null;
  existsInZip: boolean;
  /** True when src is a base64 data URL from the fallback route. */
  isFallback: boolean;
  /** Call from the element's onError handler. */
  onSourceError: () => void;
}

/**
 * Resolves a playable/displayable URL for a chat media file.
 * 1. asset protocol URL (file on disk, browser-native format)
 * 2. on element error / unsupported format: base64 via `get_media_as_base64`
 */
export function useMediaSource(chatId: string, filename: string): MediaSource {
  const clean = cleanMediaFilename(filename);
  const key = `${chatId}\u0000${clean}`;
  const [state, setState] = useState<SourceState>(() => initialState(key, clean));
  const cur = state.key === key ? state : initialState(key, clean);
  const [baseDir, setBaseDir] = useState<string | undefined>(() => getCachedMediaBaseDir(chatId));
  const effectiveBaseDir = getCachedMediaBaseDir(chatId) ?? baseDir;

  const patch = useCallback((p: Partial<SourceState>) => {
    setState((prev) => ({ ...(prev.key === key ? prev : initialState(key, clean)), ...p }));
  }, [key, clean]);

  // Resolve the media folder once per chat.
  useEffect(() => {
    if (!chatId || cur.mode !== "asset" || effectiveBaseDir !== undefined) return;
    let cancelled = false;
    getMediaBaseDir(chatId)
      .then((dir) => { if (!cancelled) setBaseDir(dir); })
      .catch(() => { if (!cancelled) patch({ mode: "invoke" }); });
    return () => { cancelled = true; };
  }, [chatId, cur.mode, effectiveBaseDir, patch]);

  // Fallback route: base64 over IPC (also extracts from the source archive).
  useEffect(() => {
    if (!chatId || !clean || cur.mode !== "invoke") return;
    let cancelled = false;
    invoke<string>("get_media_as_base64", { chatId, filename: clean, mimeHint: null })
      .then((data) => { if (!cancelled) patch({ fallbackSrc: data }); })
      .catch(async (e) => {
        if (cancelled) return;
        patch({ error: String(e) });
        try {
          const inZip = await invoke<boolean>("check_file_in_zip", { chatId, filename: clean });
          if (!cancelled) patch({ existsInZip: inZip });
        } catch { /* ignore zip check errors */ }
      });
    return () => { cancelled = true; };
    // Only restart on chat/file/mode change: patching fallbackSrc/error must not cancel the zip check.
  }, [chatId, clean, cur.mode, patch]);

  const onSourceError = useCallback(() => {
    setState((prev) => {
      const base = prev.key === key ? prev : initialState(key, clean);
      if (base.mode === "asset") return { ...base, mode: "invoke" };
      return base.error ? base : { ...base, error: "decode-error" };
    });
  }, [key, clean]);

  let src: string | null = null;
  if (cur.mode === "asset") {
    src = effectiveBaseDir !== undefined ? convertFileSrc(joinMediaPath(effectiveBaseDir, clean)) : null;
  } else {
    src = cur.fallbackSrc;
  }

  return {
    src: cur.error ? null : src,
    error: cur.error,
    existsInZip: cur.existsInZip,
    isFallback: cur.mode === "invoke",
    onSourceError,
  };
}

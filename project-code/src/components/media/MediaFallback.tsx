import { useState } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { mediaCache, showModuleToast } from "../../LazyMediaImage";

interface MediaFallbackProps {
  filename: string;
  className?: string;
  chatId?: string;
  existsInZip?: boolean;
  onRestored?: () => void;
}

export function MediaFallback({ filename, className, chatId, existsInZip, onRestored }: MediaFallbackProps) {
  const [restoring, setRestoring] = useState(false);

  async function restoreFromZip() {
    if (!chatId) return;

    setRestoring(true);

    try {
      await invoke("extract_file_from_zip", { chatId, filename });
      // Evict from frontend cache so LazyMediaImage re-fetches from disk on next scroll
      mediaCache.delete(`${chatId}:${filename}`);
      setRestoring(false);
      onRestored?.();
    } catch (e) {
      showModuleToast("Failed to restore from ZIP: " + e);
      setRestoring(false);
    }
  }

  return (
    <div className={`media-fallback ${className ?? ""}`}>
      <div className="media-fallback-content">
        <div className="media-fallback-icon">📄</div>
        <div className="media-fallback-filename">{filename}</div>
        <div className="media-fallback-status">
          {existsInZip ? "File in ZIP (can restore)" : "File not found"}
        </div>
        {existsInZip && chatId && (
          <button
            className="media-fallback-restore"
            onClick={restoreFromZip}
            disabled={restoring}
          >
            {restoring ? "Restoring…" : "Restore from ZIP"}
          </button>
        )}
      </div>
    </div>
  );
}

import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { PhotoHistoryEntry } from "../../types";

interface PhotoHistoryDialogProps {
  entries: PhotoHistoryEntry[];
  currentPhotoPath: string | null | undefined;
  onClose: () => void;
  onView: (base64Data: string) => void;
  onRestore: (photoPath: string) => void;
  onDelete: (id: number) => void;
}

export function PhotoHistoryDialog({
  entries,
  currentPhotoPath,
  onClose,
  onView,
  onRestore,
  onDelete,
}: PhotoHistoryDialogProps) {
  // Profile photos are loaded as base64 everywhere else in the app (see ProfileDialog) rather
  // than via convertFileSrc — the asset-protocol path never reliably resolves for this
  // directory, so this mirrors the proven-working approach instead.
  const [photoSrcs, setPhotoSrcs] = useState<Record<number, string>>({});

  useEffect(() => {
    let cancelled = false;
    entries.forEach(entry => {
      if (photoSrcs[entry.id]) return;
      invoke<string>("read_file_as_base64", { path: entry.photo_path })
        .then(src => { if (!cancelled) setPhotoSrcs(prev => ({ ...prev, [entry.id]: src })); })
        .catch(() => {});
    });
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [entries]);

  function formatDate(raw: string): string {
    const d = new Date(raw);
    return isNaN(d.getTime()) ? raw : d.toLocaleString();
  }

  return (
    <div className="dialog-overlay" onClick={onClose}>
      <div className="bg-modal photo-history-modal" onClick={(e) => e.stopPropagation()}>
        <button className="dialog-close-btn" onClick={onClose}>×</button>
        <h3 className="bg-modal-title">Photo History</h3>
        {entries.length === 0 ? (
          <div className="photo-history-empty">
            <p className="bg-empty-hint">No previous photos yet.</p>
          </div>
        ) : (
          <div className="media-grid photo-history-grid">
            {entries.map((entry) => {
              const isCurrent = currentPhotoPath === entry.photo_path;
              const src = photoSrcs[entry.id];
              return (
                <div key={entry.id} className="media-grid-item-wrapper">
                  <div
                    className={`media-grid-item${src ? "" : " broken"}`}
                    onClick={() => src && onView(src)}
                  >
                    {src && <img src={src} alt="Previous profile photo" />}
                    <div className="media-grid-overlay">
                      <span className="media-grid-date">{formatDate(entry.changed_at)}</span>
                    </div>
                    {isCurrent ? (
                      <span className="media-grid-time">Current</span>
                    ) : (
                      <div className="photo-history-actions">
                        <button
                          className="media-jump-btn"
                          title="Use this photo"
                          onClick={(e) => { e.stopPropagation(); onRestore(entry.photo_path); }}
                        >↺</button>
                        <button
                          className="media-jump-btn photo-history-delete-btn"
                          title="Delete from history"
                          onClick={(e) => { e.stopPropagation(); onDelete(entry.id); }}
                        >✕</button>
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}

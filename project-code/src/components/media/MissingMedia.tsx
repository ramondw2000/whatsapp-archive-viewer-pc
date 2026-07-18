const ICONS: Record<string, string> = {
  image: "🖼️",
  video: "🎥",
  gif: "🎞️",
  audio: "🎵",
  sticker: "😊",
  location: "📍",
  file: "📄",
};

const LABELS: Record<string, string> = {
  image: "Image not included in export",
  video: "Video not included in export",
  gif: "GIF not included in export",
  audio: "Audio not included in export",
  sticker: "Sticker not included in export",
  location: "Location not included in export",
  file: "File not included in export",
};

interface MissingMediaProps {
  type: string;
  className?: string;
}

// Shown when a message is typed as media (WhatsApp's own export wrote a placeholder like
// "<Media omitted>" as the content, which is how the type gets detected in the first
// place) but has no filename at all — distinct from MediaFallback, which handles a
// filename that's in the database but whose file is missing from disk.
export function MissingMedia({ type, className }: MissingMediaProps) {
  return (
    <div className={`media-fallback ${className ?? ""}`}>
      <div className="media-fallback-content">
        <div className="media-fallback-icon">{ICONS[type] ?? "📎"}</div>
        <div className="media-fallback-status">{LABELS[type] ?? "Media not included in export"}</div>
      </div>
    </div>
  );
}

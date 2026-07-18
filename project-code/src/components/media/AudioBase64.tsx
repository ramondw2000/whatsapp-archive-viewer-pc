import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { useLazyVisibility } from "../../LazyMediaImage";
import { MediaFallback } from "./MediaFallback";

interface AudioBase64InnerProps {
  chatId: string;
  filename: string;
  isVideo?: boolean;
}

function AudioBase64Inner({ chatId, filename, isVideo }: AudioBase64InnerProps) {
  const [src, setSrc] = useState<string>("");
  const [err, setErr] = useState<string>("");
  const [existsInZip, setExistsInZip] = useState(false);

  useEffect(() => {
    if (!chatId || !filename) return;

    setSrc("");
    setErr("");
    setExistsInZip(false);

    const clean = filename.replace(/[\u200E\u200F\u202A-\u202E\u2066-\u2069\uFEFF\u200B]/g, "");
    invoke<string>("get_media_as_base64", { chatId, filename: clean, mimeHint: null })
      .then((data) => setSrc(data))
      .catch(async (e) => {
        setErr(String(e));
        // Check if file exists in ZIP
        try {
          const inZip = await invoke<boolean>("check_file_in_zip", { chatId, filename: clean });
          setExistsInZip(inZip);
        } catch { /* ignore zip check errors */ }
      });
  }, [chatId, filename]);

  if (err) return <MediaFallback filename={filename} chatId={chatId} existsInZip={existsInZip} />;
  if (!src) return <span className="media-audio-loading">Loading…</span>;
  if (isVideo) return <video src={src} controls autoPlay className="lightbox-video" onClick={(e) => e.stopPropagation()} />;
  return <audio controls src={src} className="media-audio-player" />;
}

interface AudioBase64Props extends AudioBase64InnerProps {
  className?: string;
}

export function AudioBase64(props: AudioBase64Props) {
  const { ref, isVisible, style, className } = useLazyVisibility();

  return (
    <div ref={ref} style={style} className={className}>
      {isVisible ? <AudioBase64Inner {...props} /> : (
        <div className="media-skeleton" style={{ width: '100%', minHeight: 54 }}><div className="skeleton-shimmer"></div></div>
      )}
    </div>
  );
}

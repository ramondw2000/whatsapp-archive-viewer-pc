import { useState, useEffect, useRef } from 'react';
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { useLazyVisibility } from "../../LazyMediaImage";
import { MediaFallback } from "./MediaFallback";

interface GifPlayerInnerProps {
  chatId: string;
  filename: string;
  onToggleType?: () => void;
  autoPlay?: boolean;
}

function GifPlayerInner({ chatId, filename, onToggleType, autoPlay = false }: GifPlayerInnerProps) {
  const [src, setSrc] = useState<string>("");
  const [playing, setPlaying] = useState(autoPlay);
  const [err, setErr] = useState<string>("");
  const [existsInZip, setExistsInZip] = useState(false);
  const videoRef = useRef<HTMLVideoElement>(null);

  useEffect(() => {
    if (!chatId || !filename) return;

    const clean = filename.replace(/[\u200E\u200F\u202A-\u202E\u2066-\u2069\uFEFF\u200B]/g, "");
    setSrc("");
    setErr("");
    setExistsInZip(false);

    invoke<string>("get_media_path", { chatId, filename: clean })
      .then((path) => setSrc(convertFileSrc(path)))
      .catch(async (e) => {
        const msg = String(e);
        setErr(msg);
        // Check if file exists in ZIP
        try {
          const inZip = await invoke<boolean>("check_file_in_zip", { chatId, filename: clean });
          setExistsInZip(inZip);
        } catch { /* ignore zip check errors */ }
      });
  }, [chatId, filename]);

  useEffect(() => {
    if (autoPlay && src && videoRef.current) {
      videoRef.current.play().then(() => setPlaying(true)).catch(() => {});
    }
  }, [autoPlay, src]);

  function toggle() {
    const v = videoRef.current;
    if (!v) return;
    if (playing) { v.pause(); v.currentTime = 0; setPlaying(false); }
    else { v.play(); setPlaying(true); }
  }

  if (err) return <MediaFallback filename={filename} chatId={chatId} existsInZip={existsInZip} />;
  if (!src) return <div className="gif-placeholder"><span className="gif-badge">GIF</span></div>;

  return (
    <div className="gif-container" onClick={toggle}>
      <video
        ref={videoRef}
        src={src}
        muted
        loop
        playsInline
        className="gif-video"
        onEnded={() => setPlaying(false)}
        preload="metadata"
        onLoadedMetadata={() => { if (videoRef.current && !playing) videoRef.current.currentTime = 0.001; }}
      />
      {!playing && (
        <div className="gif-overlay">
          <span className="gif-badge">GIF</span>
        </div>
      )}
      {onToggleType && (
        <button
          className="media-type-toggle"
          title="Mark as video instead"
          onClick={(e) => { e.stopPropagation(); onToggleType(); }}
        >🎬</button>
      )}
    </div>
  );
}

interface GifPlayerProps extends GifPlayerInnerProps {
  index?: number;
}

export function GifPlayer(props: GifPlayerProps) {
  const { ref, isVisible, style, className } = useLazyVisibility();
  const { index, ...innerProps } = props;
  const [dims, setDims] = useState<{ width: number; height: number } | null>(null);

  useEffect(() => {
    if (!isVisible || !innerProps.chatId || !innerProps.filename) return;

    const clean = innerProps.filename.replace(/[\u200E\u200F\u202A-\u202E\u2066-\u2069\uFEFF\u200B]/g, "");

    invoke<{ data: string; width: number; height: number }>("get_media_with_dims", {
      chatId: innerProps.chatId,
      filename: clean,
      mimeHint: null
    }).then(result => {
      if (result.width > 0 && result.height > 0) {
        const scale = Math.min(1, 300 / result.height);
        setDims({ width: Math.round(result.width * scale), height: Math.round(result.height * scale) });
      }
    }).catch(() => {});
  }, [isVisible, innerProps.chatId, innerProps.filename]);

  const skeletonStyle = dims ? { width: dims.width, height: dims.height } : { width: 250, minHeight: 180 };

  return (
    <div ref={ref} style={style} className={className} data-lazy-gif="true" data-index={index} data-filename={innerProps.filename} data-chat-id={innerProps.chatId}>
      {isVisible ? <GifPlayerInner {...innerProps} /> : (
        <div className="media-skeleton" style={skeletonStyle}><div className="skeleton-shimmer"></div></div>
      )}
    </div>
  );
}

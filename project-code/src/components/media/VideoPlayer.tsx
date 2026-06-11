import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { useLazyVisibility } from "../../LazyMediaImage";
import { MediaFallback } from "./MediaFallback";

interface VideoPlayerInnerProps {
  chatId: string;
  filename: string;
  className?: string;
  controls?: boolean;
  autoPlay?: boolean;
  onToggleType?: () => void;
}

function VideoPlayerInner({ chatId, filename, className, controls = true, autoPlay = false, onToggleType }: VideoPlayerInnerProps) {
  const [src, setSrc] = useState<string>("");
  const [err, setErr] = useState<string>("");
  const [existsInZip, setExistsInZip] = useState(false);

  useEffect(() => {
    if (!chatId || !filename) return;

    const clean = filename.replace(/[\u200E\u200F\u202A-\u202E\u2066-\u2069\uFEFF\u200B]/g, "");
    setSrc("");
    setErr("");
    setExistsInZip(false);

    invoke<string>("get_media_as_base64", { chatId, filename: clean, mimeHint: null })
      .then((data) => setSrc(data))
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

  if (err) return <MediaFallback filename={filename} className={className} chatId={chatId} existsInZip={existsInZip} />;
  if (!src) return <div className="media-video-loading"><span>▶</span></div>;

  return (
    <div className="video-container">
      <video
        src={src}
        controls={controls}
        autoPlay={autoPlay}
        className={className ?? "chat-video"}
        onClick={(e) => e.stopPropagation()}
      />
      {onToggleType && (
        <button
          className="media-type-toggle"
          title="Mark as GIF instead"
          onClick={(e) => { e.stopPropagation(); onToggleType(); }}
        >GIF</button>
      )}
    </div>
  );
}

interface VideoPlayerProps extends VideoPlayerInnerProps {
  index?: number;
}

export function VideoPlayer(props: VideoPlayerProps) {
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

  const skeletonStyle = dims ? { width: dims.width, height: dims.height } : { width: 300, minHeight: 200 };

  return (
    <div ref={ref} style={style} className={className} data-lazy-video="true" data-index={index} data-filename={innerProps.filename} data-chat-id={innerProps.chatId}>
      {isVisible ? <VideoPlayerInner {...innerProps} /> : (
        <div className="media-skeleton" style={skeletonStyle}><div className="skeleton-shimmer"></div></div>
      )}
    </div>
  );
}

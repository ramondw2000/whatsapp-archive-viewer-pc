import { useLazyVisibility } from "../../LazyMediaImage";
import { useMediaSource } from "../../utils/mediaSource";
import { MediaFallback } from "./MediaFallback";

interface AudioBase64InnerProps {
  chatId: string;
  filename: string;
  isVideo?: boolean;
}

function AudioBase64Inner({ chatId, filename, isVideo }: AudioBase64InnerProps) {
  const media = useMediaSource(chatId, filename);

  if (media.error) return <MediaFallback filename={filename} chatId={chatId} existsInZip={media.existsInZip} />;
  if (!media.src) return <span className="media-audio-loading">Loading…</span>;
  if (isVideo) return <video src={media.src} controls autoPlay preload="metadata" className="lightbox-video" onClick={(e) => e.stopPropagation()} onError={media.onSourceError} />;
  return <audio controls preload="metadata" src={media.src} className="media-audio-player" onError={media.onSourceError} />;
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

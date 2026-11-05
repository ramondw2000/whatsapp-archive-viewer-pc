import { useState, useEffect, useRef } from 'react';
import { useLazyVisibility } from "../../LazyMediaImage";
import { useMediaSource, useMediaDims } from "../../utils/mediaSource";
import { MediaFallback } from "./MediaFallback";

interface GifPlayerInnerProps {
  chatId: string;
  filename: string;
  onToggleType?: () => void;
  autoPlay?: boolean;
}

function GifPlayerInner({ chatId, filename, onToggleType, autoPlay = false }: GifPlayerInnerProps) {
  const media = useMediaSource(chatId, filename);
  const src = media.src;
  const [playing, setPlaying] = useState(autoPlay);
  const videoRef = useRef<HTMLVideoElement>(null);

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

  if (media.error) return <MediaFallback filename={filename} chatId={chatId} existsInZip={media.existsInZip} />;
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
        onError={media.onSourceError}
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
  const { index: _index, ...innerProps } = props;
  const dims = useMediaDims(innerProps.chatId, innerProps.filename, isVisible);

  let skeletonStyle: React.CSSProperties = { width: 250, minHeight: 180 };
  if (dims) {
    const scale = Math.min(1, 300 / dims.height);
    skeletonStyle = { width: Math.round(dims.width * scale), height: Math.round(dims.height * scale) };
  }

  return (
    <div ref={ref} style={style} className={className}>
      {isVisible ? <GifPlayerInner {...innerProps} /> : (
        <div className="media-skeleton" style={skeletonStyle}><div className="skeleton-shimmer"></div></div>
      )}
    </div>
  );
}

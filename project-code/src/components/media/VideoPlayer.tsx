import { useLazyVisibility } from "../../LazyMediaImage";
import { useMediaSource, useMediaDims } from "../../utils/mediaSource";
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
  const media = useMediaSource(chatId, filename);

  if (media.error) return <MediaFallback filename={filename} className={className} chatId={chatId} existsInZip={media.existsInZip} />;
  if (!media.src) return <div className="media-video-loading"><span>▶</span></div>;

  return (
    <div className="video-container">
      <video
        // #t=0.001 makes the browser paint the first frame as a thumbnail with preload=metadata
        src={media.isFallback ? media.src : `${media.src}#t=0.001`}
        controls={controls}
        autoPlay={autoPlay}
        preload="metadata"
        className={className ?? "chat-video"}
        onClick={(e) => e.stopPropagation()}
        onError={media.onSourceError}
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
  const { index: _index, ...innerProps } = props;
  // Header-only dimension lookup (no video data is read or sent over IPC)
  const dims = useMediaDims(innerProps.chatId, innerProps.filename, isVisible);

  let skeletonStyle: React.CSSProperties = { width: 300, minHeight: 200 };
  if (dims) {
    const scale = Math.min(1, 300 / dims.height);
    skeletonStyle = { width: Math.round(dims.width * scale), height: Math.round(dims.height * scale) };
  }

  return (
    <div ref={ref} style={style} className={className}>
      {isVisible ? <VideoPlayerInner {...innerProps} /> : (
        <div className="media-skeleton" style={skeletonStyle}><div className="skeleton-shimmer"></div></div>
      )}
    </div>
  );
}

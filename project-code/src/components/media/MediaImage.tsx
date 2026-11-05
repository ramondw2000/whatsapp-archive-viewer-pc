import { useMediaSource } from "../../utils/mediaSource";
import { MediaFallback } from "./MediaFallback";

interface MediaImageProps {
  chatId: string;
  filename: string;
  alt?: string;
  className?: string;
  onError?: (e: React.SyntheticEvent<HTMLImageElement>) => void;
  onLoad?: (e: React.SyntheticEvent<HTMLImageElement>) => void;
}

export function MediaImage({ chatId, filename, alt, className, onError, onLoad }: MediaImageProps) {
  // Loads are tied to the component lifecycle (the hook ignores results for
  // unmounted / re-targeted cells), and the browser streams + decodes the file.
  const media = useMediaSource(chatId, filename);

  if (media.error) return <MediaFallback filename={filename} className={className} chatId={chatId} existsInZip={media.existsInZip} />;
  if (!media.src) return null;

  return (
    <img
      src={media.src}
      alt={alt ?? filename}
      className={className}
      loading="lazy"
      decoding="async"
      onError={(e) => {
        const wasFallback = media.isFallback;
        media.onSourceError();
        if (wasFallback) onError?.(e);
      }}
      onLoad={onLoad}
    />
  );
}

import { useMediaSource } from "../../utils/mediaSource";
import { MediaFallback } from "./MediaFallback";

interface StickerImageProps {
  chatId: string;
  filename: string;
}

export function StickerImage({ chatId, filename }: StickerImageProps) {
  const media = useMediaSource(chatId, filename);

  if (media.error) return <MediaFallback filename={filename} chatId={chatId} existsInZip={media.existsInZip} />;
  if (!media.src) return null;

  return <img src={media.src} alt="sticker" className="sticker-img" decoding="async" onError={media.onSourceError} />;
}

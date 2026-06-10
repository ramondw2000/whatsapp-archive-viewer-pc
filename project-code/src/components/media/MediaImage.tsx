import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
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
  const [src, setSrc] = useState<string | null>(null);
  const [hasError, setHasError] = useState(false);
  const [existsInZip, setExistsInZip] = useState(false);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!chatId || !filename) return;

    setSrc(null);
    setHasError(false);
    setExistsInZip(false);
    setLoading(true);

    invoke<string>("get_media_as_base64", { chatId, filename, mimeHint: null })
      .then((s) => {
        setSrc(s);
        setLoading(false);
      })
      .catch(async () => {
        setHasError(true);
        setLoading(false);
        // Check if file exists in ZIP
        try {
          const inZip = await invoke<boolean>("check_file_in_zip", { chatId, filename });
          setExistsInZip(inZip);
        } catch { /* ignore */ }
      });
  }, [chatId, filename]);

  if (hasError) return <MediaFallback filename={filename} className={className} chatId={chatId} existsInZip={existsInZip} />;
  if (!src && loading) return null;
  if (!src) return <MediaFallback filename={filename} className={className} chatId={chatId} existsInZip={existsInZip} />;

  return <img src={src} alt={alt ?? filename} className={className} onError={(e) => { setHasError(true); onError?.(e); }} onLoad={onLoad} />;
}

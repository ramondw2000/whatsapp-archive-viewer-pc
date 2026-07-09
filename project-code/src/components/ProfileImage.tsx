import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";

interface ProfileImageProps {
  photoPath: string | null | undefined;
  alt: string;
  className?: string;
  onPhotoClick?: (base64Data: string) => void;
}

export function ProfileImage({ photoPath, alt, className, onPhotoClick }: ProfileImageProps) {
  const [src, setSrc] = useState<string | null>(null);

  useEffect(() => {
    if (!photoPath) {
      setSrc(null);
      return;
    }
    invoke<string>("read_file_as_base64", { path: photoPath })
      .then(setSrc)
      .catch(() => setSrc(null));
  }, [photoPath]);

  if (!src) return null;
  return <img src={src} alt={alt} className={className} onClick={() => onPhotoClick?.(src)} />;
}

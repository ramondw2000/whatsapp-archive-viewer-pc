import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";

interface ProfileImageProps {
  photoPath: string | null | undefined;
  alt: string;
  className?: string;
  onPhotoClick?: (base64Data: string) => void;
  // Shown when there's no photoPath, or the file it points to can't be read (e.g. deleted
  // externally) — without this, a stale photo_path would leave the avatar permanently blank
  // instead of falling back to initials/GroupAvatar like a chat with no photo at all.
  fallback?: React.ReactNode;
}

export function ProfileImage({ photoPath, alt, className, onPhotoClick, fallback = null }: ProfileImageProps) {
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

  if (!src) return <>{fallback}</>;
  return <img src={src} alt={alt} className={className} onClick={() => onPhotoClick?.(src)} />;
}

import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { MediaFallback } from "./MediaFallback";

interface StickerImageProps {
  chatId: string;
  filename: string;
}

export function StickerImage({ chatId, filename }: StickerImageProps) {
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
        setErr(String(e));
        try {
          const inZip = await invoke<boolean>("check_file_in_zip", { chatId, filename: clean });
          setExistsInZip(inZip);
        } catch { /* ignore */ }
      });
  }, [chatId, filename]);

  if (err) return <MediaFallback filename={filename} chatId={chatId} existsInZip={existsInZip} />;
  if (!src) return null;

  return <img src={src} alt="sticker" className="sticker-img" onError={() => setErr("decode-error")} />;
}

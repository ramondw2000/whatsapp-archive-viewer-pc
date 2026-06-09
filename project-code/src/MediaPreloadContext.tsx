import React, { createContext, useContext, useCallback, useRef } from 'react';
import { invoke } from "@tauri-apps/api/core";

interface MediaPreloadContextValue {
  registerImages: (chatId: string, filenames: string[]) => void;
  preloadNext: (chatId: string, currentFilename: string, count: number) => void;
}

const MediaPreloadContext = createContext<MediaPreloadContextValue | null>(null);

export function MediaPreloadProvider({ children }: { children: React.ReactNode }) {
  const imageListsRef = useRef<Map<string, string[]>>(new Map());

  const registerImages = useCallback((chatId: string, filenames: string[]) => {
    imageListsRef.current.set(chatId, filenames);
  }, []);

  const preloadNext = useCallback(async (chatId: string, currentFilename: string, count: number = 5) => {
    const images = imageListsRef.current.get(chatId);
    if (!images || images.length === 0) return;

    const currentIndex = images.indexOf(currentFilename);
    if (currentIndex === -1) return;

    // Get next 'count' images after current
    const toPreload = images
      .slice(currentIndex + 1, currentIndex + 1 + count)
      .filter(filename => filename !== currentFilename);

    if (toPreload.length === 0) return;

    try {
      const loaded = await invoke<number>("preload_media", { chatId, filenames: toPreload });
      console.log(`[MediaPreload] Preloaded ${loaded}/${toPreload.length} images after ${currentFilename}`);
    } catch (err) {
      console.debug('[MediaPreload] Failed:', err);
    }
  }, []);

  return (
    <MediaPreloadContext.Provider value={{ registerImages, preloadNext }}>
      {children}
    </MediaPreloadContext.Provider>
  );
}

export function useMediaPreload() {
  const ctx = useContext(MediaPreloadContext);
  if (!ctx) {
    throw new Error('useMediaPreload must be used within MediaPreloadProvider');
  }
  return ctx;
}

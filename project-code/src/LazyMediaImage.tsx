import React, { useState, useEffect, useRef, startTransition, type ReactNode } from 'react';
import { invoke } from "@tauri-apps/api/core";

// ============================================================================
// LAZY MEDIA LOADING SYSTEM
// ============================================================================

// LRU Cache for loaded media
class MediaCache {
  private cache: Map<string, string> = new Map();
  private maxSize = 200;

  get(key: string): string | undefined {
    if (this.cache.has(key)) {
      const value = this.cache.get(key)!;
      this.cache.delete(key);
      this.cache.set(key, value);
      return value;
    }
    return undefined;
  }

  set(key: string, value: string) {
    if (this.cache.has(key)) {
      this.cache.delete(key);
    }
    this.cache.set(key, value);
    
    if (this.cache.size > this.maxSize) {
      const firstKey = this.cache.keys().next().value as string | undefined;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    }
  }

  delete(key: string) {
    this.cache.delete(key);
  }

  clear() {
    this.cache.clear();
  }
}

export const mediaCache = new MediaCache();

// Persistent dimension cache — survives image unload/reload cycles
const dimensionCache = new Map<string, { width: number; height: number }>();

// ============================================================================
// PRELOAD HELPER
// ============================================================================

/**
 * Preload media items above and below the current index by querying DOM for data-index
 */
async function preloadNextImages(chatId: string, currentIndex: number | undefined, count: number = 10) {
  if (currentIndex === undefined) return;

  try {
    // Find all lazy media containers with data-index (images, videos, GIFs)
    const imageContainers = document.querySelectorAll('[data-lazy-image="true"][data-index]');
    const videoContainers = document.querySelectorAll('[data-lazy-video="true"][data-index]');
    const gifContainers = document.querySelectorAll('[data-lazy-gif="true"][data-index]');
    const allCandidates: { index: number; filename: string }[] = [];

    const processContainers = (containers: NodeListOf<Element>) => {
      for (const container of containers) {
        const idx = parseInt(container.getAttribute('data-index') || '-1', 10);
        const filename = container.getAttribute('data-filename');
        const containerChatId = container.getAttribute('data-chat-id');
        if (idx !== currentIndex && filename && containerChatId === chatId && !mediaCache.get(`${chatId}:${filename}`)) {
          allCandidates.push({ index: idx, filename });
        }
      }
    };

    processContainers(imageContainers);
    processContainers(videoContainers);
    processContainers(gifContainers);

    // Separate into above and below
    const above = allCandidates.filter(c => c.index < currentIndex).sort((a, b) => b.index - a.index); // descending
    const below = allCandidates.filter(c => c.index > currentIndex).sort((a, b) => a.index - b.index); // ascending

    // Take 'count' from each direction
    const toPreload = [...above.slice(0, count), ...below.slice(0, count)].map(c => c.filename);

    if (toPreload.length === 0) return;

    const loaded = await invoke<number>("preload_media", { chatId, filenames: toPreload });
    console.log(`[LazyMediaImage] Preloaded ${loaded}/${toPreload.length} media items around index ${currentIndex}`);
  } catch (err) {
    // Preload errors are non-fatal, just log them
    console.debug('[LazyMediaImage] Preload failed:', err);
  }
}

// ============================================================================
// LAZY MEDIA IMAGE COMPONENT
// ============================================================================

interface LazyMediaImageProps {
  chatId: string;
  filename: string;
  alt?: string;
  className?: string;
  onError?: (e: React.SyntheticEvent<HTMLImageElement>) => void;
  onLoad?: (e: React.SyntheticEvent<HTMLImageElement>) => void;
  aspectRatio?: string;
  index?: number; // Message index for determining preload order
}

// Placeholder for MediaFallback - will be imported from App
let MediaFallback: any = null;

export function setMediaFallback(component: any) {
  MediaFallback = component;
}

// Module-level toast callback injected from App
let _showToast: ((msg: string) => void) | null = null;

export function setToastCallback(fn: (msg: string) => void) {
  _showToast = fn;
}

export function showModuleToast(msg: string) {
  if (_showToast) _showToast(msg);
  else console.error('[Toast]', msg);
}

export function LazyMediaImage({
  chatId,
  filename,
  alt,
  className,
  onError,
  onLoad,
  aspectRatio = '4/3',
  index
}: LazyMediaImageProps) {
  const [src, setSrc] = useState<string | null>(null);
  const [hasError, setHasError] = useState(false);
  const [existsInZip, setExistsInZip] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);
  const imgRef = useRef<HTMLImageElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const isLoadedRef = useRef(false);

  const dimKey = `${chatId}:${filename}`;
  const [dims, setDims] = useState<{ width: number; height: number } | null>(() => dimensionCache.get(dimKey) ?? null);

  const cacheKey = `${chatId}:${filename}`;

  useEffect(() => {
    if (!chatId || !filename) return;

    const cached = mediaCache.get(cacheKey);
    if (cached) {
      setSrc(cached);
      setIsLoaded(true);
      isLoadedRef.current = true;
    } else {
      setSrc(null);
      setHasError(false);
      setExistsInZip(false);
      isLoadedRef.current = false;
    }

    let isMounted = true;
    let isLoading = false;

    // Multipliers for flexible distance calculations
    // Load when within 1x viewport, unload when >3x viewport (~2400px) to free memory
    const LOAD_MULTIPLIER = 1;
    const UNLOAD_MULTIPLIER = 3;

    const checkAndLoad = async () => {
      if (!isMounted || !containerRef.current || isLoading) return;

      const rect = containerRef.current.getBoundingClientRect();
      const viewportHeight = window.innerHeight;

      // Calculate thresholds based on viewport height
      const loadThreshold = viewportHeight * LOAD_MULTIPLIER;
      const unloadThreshold = viewportHeight * UNLOAD_MULTIPLIER;

      // Check if element is within viewport + load threshold
      const isAboveLoadZone = rect.bottom < -loadThreshold;
      const isBelowLoadZone = rect.top > viewportHeight + loadThreshold;
      const isInLoadZone = !isAboveLoadZone && !isBelowLoadZone;

      // Load if in load zone and not loaded
      if (isInLoadZone && !isLoadedRef.current) {
        isLoading = true;
        try {
          const result = await invoke<{ data: string; width: number; height: number }>("get_media_with_dims", {
            chatId,
            filename,
            mimeHint: null
          });

          if (isMounted) {
            mediaCache.set(cacheKey, result.data);
            // Set dims first, let skeleton render at correct size, then swap to image
            if (result.width > 0 && result.height > 0) {
              const d = { width: result.width, height: result.height };
              dimensionCache.set(dimKey, d);
              setDims(d);
              // Use startTransition so media renders are non-urgent (React can yield to input)
              setTimeout(() => {
                if (isMounted) {
                  startTransition(() => {
                    setSrc(result.data);
                    isLoadedRef.current = true;
                  });
                }
              }, 0);
            } else {
              startTransition(() => {
                setSrc(result.data);
                isLoadedRef.current = true;
              });
            }
            console.log(`[LazyMediaImage] LOADED: ${filename} (${result.width}x${result.height})`);

            // Preload next 5 images — deferred to idle time so it doesn't compete with active loads
            if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
              window.requestIdleCallback(() => preloadNextImages(chatId, index, 5), { timeout: 2000 });
            } else {
              setTimeout(() => preloadNextImages(chatId, index, 5), 500);
            }
          }
        } catch (err) {
          console.error(`[LazyMediaImage] FAILED to load ${filename}:`, err);
          if (isMounted) {
            setHasError(true);

            try {
              const inZip = await invoke<boolean>("check_file_in_zip", {
                chatId,
                filename
              });
              if (isMounted) {
                setExistsInZip(inZip);
              }
            } catch { /* ignore */ }
          }
        }
        isLoading = false;
      }

      // Unload if beyond viewport + unload threshold (outside load zone with buffer)
      const isAboveUnloadZone = rect.bottom < -unloadThreshold;
      const isBelowUnloadZone = rect.top > viewportHeight + unloadThreshold;
      const isOutsideUnloadZone = isAboveUnloadZone || isBelowUnloadZone;

      if (isOutsideUnloadZone && isLoadedRef.current) {
        console.log(`[LazyMediaImage] UNLOADED: ${filename}`);
        setSrc(null);
        setIsLoaded(false);
        isLoadedRef.current = false;
      }
    };

    // Find the nearest scrollable parent first (needed for both observer and scroll)
    const findScrollContainer = (el: HTMLElement | null): HTMLElement | null => {
      while (el) {
        const style = window.getComputedStyle(el);
        if (style.overflow === 'auto' || style.overflow === 'scroll' ||
            style.overflowY === 'auto' || style.overflowY === 'scroll') {
          return el;
        }
        el = el.parentElement;
      }
      return null;
    };

    const scrollContainer = findScrollContainer(containerRef.current);

    // IntersectionObserver handles loading — throttle via rAF to prevent burst renders
    let rafScheduled = false;
    const observer = new IntersectionObserver(
      (entries) => {
        const hasIntersecting = entries.some(e => e.isIntersecting);
        if (hasIntersecting && !rafScheduled) {
          rafScheduled = true;
          requestAnimationFrame(() => {
            rafScheduled = false;
            checkAndLoad();
          });
        }
      },
      {
        root: scrollContainer,
        rootMargin: "1000px 0px 1000px 0px",
        threshold: 0
      }
    );

    if (containerRef.current) {
      observer.observe(containerRef.current);
    }

    // Scroll listener only for unload checks — debounced at 250ms, runs checkAndLoad
    // which handles the unload logic. Per-instance but cheap since isLoaded guards it.
    let scrollTimeout: ReturnType<typeof setTimeout>;
    const debouncedUnloadCheck = () => {
      clearTimeout(scrollTimeout);
      scrollTimeout = setTimeout(() => {
        if (isLoadedRef.current) checkAndLoad();
      }, 250);
    };

    if (scrollContainer) {
      scrollContainer.addEventListener('scroll', debouncedUnloadCheck, { passive: true });
    }

    return () => {
      isMounted = false;
      observer.disconnect();
      if (scrollContainer) {
        scrollContainer.removeEventListener('scroll', debouncedUnloadCheck);
      }
      clearTimeout(scrollTimeout);
    };
  }, [chatId, filename, cacheKey]);

  if (hasError && MediaFallback) {
    return (
      <MediaFallback
        filename={filename}
        className={className}
        chatId={chatId}
        existsInZip={existsInZip}
      />
    );
  }

  const showSkeleton = !src;

  // Compute reserved size: use cached natural dims if available, else fall back to aspectRatio
  const reservedStyle: React.CSSProperties = (() => {
    if (dims) {
      const scale = Math.min(1, 300 / dims.height);
      return { width: Math.round(dims.width * scale), height: Math.round(dims.height * scale) };
    }
    return { aspectRatio };
  })();

  return (
    <div
      ref={containerRef}
      className="media-lazy-container"
      data-lazy-image="true"
      data-filename={filename}
      data-chat-id={chatId}
      data-index={index}
      style={{ ...reservedStyle, position: 'relative' }}
    >
      {showSkeleton && (
        <div className="media-skeleton" style={reservedStyle}>
          <div className="skeleton-shimmer"></div>
        </div>
      )}

      {src && (
        <img
          ref={imgRef}
          src={src}
          alt={alt ?? filename}
          className={`${className || ''} ${isLoaded ? 'media-loaded' : 'media-blur'}`}
          onError={(e) => {
            setHasError(true);
            onError?.(e);
          }}
          onLoad={(e) => {
            setIsLoaded(true);
            onLoad?.(e);
          }}
          style={{
            maxWidth: '100%',
            maxHeight: '300px',
            display: 'block',
            borderRadius: '6px'
          }}
        />
      )}
    </div>
  );
}

// ============================================================================
// LAZY VISIBILITY HOOK — used by media components internally
// ============================================================================

function findScrollParent(el: HTMLElement | null): HTMLElement | null {
  while (el) {
    const style = window.getComputedStyle(el);
    if (style.overflow === 'auto' || style.overflow === 'scroll' ||
        style.overflowY === 'auto' || style.overflowY === 'scroll') {
      return el;
    }
    el = el.parentElement;
  }
  return null;
}

export function useLazyVisibility(rootMargin = "500px 0px 500px 0px") {
  const ref = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const scrollRoot = findScrollParent(el.parentElement);

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setIsVisible(true);
            observer.unobserve(entry.target);
          }
        }
      },
      { root: scrollRoot, rootMargin, threshold: 0 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [rootMargin]);

  return { ref, isVisible, style: { display: 'inline-block' }, className: 'media-lazy-wrapper' };
}

// ============================================================================
// GENERIC LAZY MEDIA WRAPPER
// Defers rendering children until element is near the viewport.
// Works for video, gif, audio, file attachments, etc.
// ============================================================================

interface LazyMediaWrapperProps {
  children: ReactNode;
  className?: string;
  skeletonHeight?: number;
  skeletonWidth?: number | string;
  rootMargin?: string;
}

export function LazyMediaWrapper({
  children,
  className,
  skeletonHeight = 180,
  skeletonWidth = 250,
  rootMargin = "500px 0px 500px 0px"
}: LazyMediaWrapperProps) {
  const [isVisible, setIsVisible] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;

    const scrollRoot = findScrollParent(el.parentElement);

    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            setIsVisible(true);
            observer.unobserve(entry.target);
          }
        }
      },
      { root: scrollRoot, rootMargin, threshold: 0 }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [rootMargin]);

  return (
    <div ref={containerRef} className={className}>
      {isVisible ? children : (
        <div className="media-skeleton" style={{ width: skeletonWidth, minHeight: skeletonHeight }}>
          <div className="skeleton-shimmer"></div>
        </div>
      )}
    </div>
  );
}

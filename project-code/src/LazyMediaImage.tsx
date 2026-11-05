import React, { useState, useRef, useEffect, type ReactNode } from 'react';
import { useMediaSource, useMediaDims, rememberMediaDims } from "./utils/mediaSource";

// ============================================================================
// LAZY MEDIA IMAGE
//
// Virtuoso already virtualizes the message list, so rows that are far from
// the viewport are not mounted at all. Images are loaded straight from disk
// through the asset protocol (see utils/mediaSource) and decoded off the main
// thread; no per-instance scroll listeners or manual load/unload are needed.
// ============================================================================

interface LazyMediaImageProps {
  chatId: string;
  filename: string;
  alt?: string;
  className?: string;
  onError?: (e: React.SyntheticEvent<HTMLImageElement>) => void;
  onLoad?: (e: React.SyntheticEvent<HTMLImageElement>) => void;
  aspectRatio?: string;
  index?: number; // kept for call-site compatibility
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
}: LazyMediaImageProps) {
  const media = useMediaSource(chatId, filename);
  const dims = useMediaDims(chatId, filename);
  const [loadedKey, setLoadedKey] = useState<string | null>(null);
  const loadKey = `${chatId}:${filename}:${media.isFallback ? 'b' : 'a'}`;
  const isLoaded = loadedKey === loadKey;

  if (media.error && MediaFallback) {
    return (
      <MediaFallback
        filename={filename}
        className={className}
        chatId={chatId}
        existsInZip={media.existsInZip}
      />
    );
  }

  // Reserve space: natural dims when known (scaled to the 300px max height), else aspectRatio
  const reservedStyle: React.CSSProperties = dims
    ? (() => {
        const scale = Math.min(1, 300 / dims.height);
        return { width: Math.round(dims.width * scale), height: Math.round(dims.height * scale) };
      })()
    : { aspectRatio };

  return (
    <div
      className="media-lazy-container"
      style={{ ...reservedStyle, position: 'relative' }}
    >
      {!media.src && (
        <div className="media-skeleton" style={reservedStyle}>
          <div className="skeleton-shimmer"></div>
        </div>
      )}

      {media.src && (
        <img
          src={media.src}
          alt={alt ?? filename}
          className={`${className || ''} ${isLoaded ? 'media-loaded' : 'media-blur'}`}
          decoding="async"
          onError={(e) => {
            const wasFallback = media.isFallback;
            media.onSourceError();
            if (wasFallback) onError?.(e);
          }}
          onLoad={(e) => {
            const img = e.currentTarget;
            if (!dims && img.naturalWidth > 0 && img.naturalHeight > 0) {
              rememberMediaDims(chatId, filename, { width: img.naturalWidth, height: img.naturalHeight });
            }
            setLoadedKey(loadKey);
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
  // Fast path: Virtuoso marks its scroller, so no getComputedStyle walk is needed.
  const virtuoso = el?.closest<HTMLElement>('[data-virtuoso-scroller="true"]');
  if (virtuoso) return virtuoso;
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

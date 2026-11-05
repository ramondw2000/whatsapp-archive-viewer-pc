import React, { forwardRef, useImperativeHandle, useRef, useState, useEffect } from 'react';
import { Virtuoso, VirtuosoHandle } from 'react-virtuoso';

interface Message {
  [key: string]: any;
}

interface VirtualMessageListProps {
  messages: Message[];
  renderMessage: (msg: Message, index: number) => React.ReactNode;
  followOutput?: boolean | 'auto' | 'smooth';
  onScrollStateChange?: (state: { atTop: boolean; atBottom: boolean }) => void;
}

export interface VirtualMessageListRef {
  scrollToTop: () => void;
  scrollToBottom: () => void;
  scrollToIndex: (index: number, offset?: number) => void;
  adjustScrollBy: (pixels: number) => void;
  getScroller: () => HTMLElement | null;
}

// How long after mount the list keeps itself pinned to the bottom while rows
// above it settle (images/videos report their real height). Ends early on any
// user input or programmatic navigation.
const STICK_TO_BOTTOM_MS = 2500;

const INITIAL_INDEX = { index: 'LAST' as const, align: 'end' as const };
const INCREASE_VIEWPORT = { top: 300, bottom: 300 };

export const VirtualMessageList = forwardRef<VirtualMessageListRef, VirtualMessageListProps>(function VirtualMessageList({
  messages,
  renderMessage,
  followOutput = 'smooth',
  onScrollStateChange
}, ref) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const scrollerRef = useRef<HTMLElement | null>(null);
  const stickRef = useRef(true);
  const [atTop, setAtTop] = useState(false);
  const [atBottom, setAtBottom] = useState(true);

  useImperativeHandle(ref, () => ({
    scrollToTop: () => {
      stickRef.current = false;
      virtuosoRef.current?.scrollToIndex({ index: 0, behavior: 'smooth' });
    },
    scrollToBottom: () => {
      virtuosoRef.current?.scrollToIndex({ index: messages.length - 1, behavior: 'smooth' });
    },
    scrollToIndex: (index: number, offset?: number) => {
      stickRef.current = false;
      virtuosoRef.current?.scrollToIndex({ index, behavior: 'smooth', align: 'center', offset });
    },
    adjustScrollBy: (pixels: number) => {
      if (scrollerRef.current) {
        scrollerRef.current.scrollTop += pixels;
      }
    },
    getScroller: () => scrollerRef.current,
  }));

  // Short "stick to bottom" window after the list opens: rows above the bottom
  // change height as media loads, which would otherwise push the latest
  // messages out of view. Replaces the old always-on MutationObserver.
  useEffect(() => {
    const scroller = scrollerRef.current;
    const content = scroller?.firstElementChild as HTMLElement | null;
    if (!scroller || !content || typeof ResizeObserver === 'undefined') return;
    stickRef.current = true;
    let raf = 0;
    const stop = () => {
      stickRef.current = false;
      observer.disconnect();
    };
    const observer = new ResizeObserver(() => {
      if (!stickRef.current || raf) return;
      raf = requestAnimationFrame(() => {
        raf = 0;
        if (stickRef.current) {
          virtuosoRef.current?.scrollToIndex({ index: 'LAST', align: 'end', behavior: 'auto' });
        }
      });
    });
    observer.observe(content);
    const timer = setTimeout(stop, STICK_TO_BOTTOM_MS);
    scroller.addEventListener('wheel', stop, { passive: true, once: true });
    scroller.addEventListener('touchstart', stop, { passive: true, once: true });
    scroller.addEventListener('pointerdown', stop, { passive: true, once: true });
    scroller.addEventListener('keydown', stop, { once: true });
    return () => {
      clearTimeout(timer);
      if (raf) cancelAnimationFrame(raf);
      observer.disconnect();
      scroller.removeEventListener('wheel', stop);
      scroller.removeEventListener('touchstart', stop);
      scroller.removeEventListener('pointerdown', stop);
      scroller.removeEventListener('keydown', stop);
    };
  }, []);

  // Notify parent of scroll state changes
  useEffect(() => {
    if (onScrollStateChange) {
      onScrollStateChange({ atTop, atBottom });
    }
  }, [atTop, atBottom, onScrollStateChange]);

  return (
    <Virtuoso
      ref={virtuosoRef}
      scrollerRef={(el) => { scrollerRef.current = el as HTMLElement | null; }}
      data={messages}
      style={{ height: '100%', width: '100%' }}
      itemContent={(index: number, message: Message) => renderMessage(message, index)}
      computeItemKey={(index) => index}
      followOutput={followOutput}
      initialTopMostItemIndex={messages.length > 0 ? INITIAL_INDEX : 0}
      overscan={200}
      increaseViewportBy={INCREASE_VIEWPORT}
      atTopStateChange={setAtTop}
      atBottomStateChange={setAtBottom}
    />
  );
});

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

export const VirtualMessageList = forwardRef<VirtualMessageListRef, VirtualMessageListProps>(function VirtualMessageList({
  messages,
  renderMessage,
  followOutput = 'smooth',
  onScrollStateChange
}, ref) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);
  const scrollerRef = useRef<HTMLElement | null>(null);
  const [atTop, setAtTop] = useState(false);
  const [atBottom, setAtBottom] = useState(true);

  useImperativeHandle(ref, () => ({
    scrollToTop: () => {
      virtuosoRef.current?.scrollToIndex({ index: 0, behavior: 'smooth' });
    },
    scrollToBottom: () => {
      virtuosoRef.current?.scrollToIndex({ index: messages.length - 1, behavior: 'smooth' });
    },
    scrollToIndex: (index: number, offset?: number) => {
      virtuosoRef.current?.scrollToIndex({ index, behavior: 'smooth', align: 'center', offset });
    },
    adjustScrollBy: (pixels: number) => {
      if (scrollerRef.current) {
        scrollerRef.current.scrollTop += pixels;
      }
    },
    getScroller: () => scrollerRef.current,
  }));

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
      followOutput={followOutput}
      initialTopMostItemIndex={messages.length > 0 ? messages.length - 1 : 0}
      overscan={800}
      increaseViewportBy={{ top: 400, bottom: 400 }}
      atTopStateChange={setAtTop}
      atBottomStateChange={setAtBottom}
    />
  );
});

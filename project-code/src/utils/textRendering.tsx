import type { ReactNode } from "react";

// Returns a substring of `text` centered on the first match of `query`, so a long message
// whose match falls past a fixed character cutoff doesn't get truncated into a preview that
// never shows the part that actually matched.
export function getSearchSnippet(text: string, query: string, contextChars = 50): string {
  const idx = text.toLowerCase().indexOf(query.toLowerCase());
  if (idx === -1) return text.length > 100 ? text.slice(0, 100) + "…" : text;
  const start = Math.max(0, idx - contextChars);
  const end = Math.min(text.length, idx + query.length + contextChars);
  return (start > 0 ? "…" : "") + text.slice(start, end) + (end < text.length ? "…" : "");
}

export function highlightText(text: string, query: string): ReactNode {
  if (!query.trim()) return text;
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const parts = text.split(new RegExp(`(${escaped})`, 'gi'));
  return parts.map((part, i) =>
    part.toLowerCase() === query.toLowerCase()
      ? <mark key={i} className="search-highlight">{part}</mark>
      : part
  );
}

interface RenderMessageTextProps {
  onLinkClick: (url: string) => void;
}

export function createRenderMessageText({ onLinkClick }: RenderMessageTextProps) {
  return function renderMessageText(text: string, query?: string): ReactNode {
    const URL_RE = /((?:https?:\/\/|www\.)[^\s<>"']+)/gi;
    const parts: ReactNode[] = [];
    let last = 0;
    let match: RegExpExecArray | null;

    while ((match = URL_RE.exec(text)) !== null) {
      const before = text.slice(last, match.index);
      const rawUrl = match[1];
      const url = rawUrl.toLowerCase().startsWith('www.') ? 'https://' + rawUrl : rawUrl;

      if (before) parts.push(query ? highlightText(before, query) : before);

      parts.push(
        <a
          key={match.index}
          className="message-link"
          onClick={e => { e.preventDefault(); onLinkClick(url); }}
          href="#"
          title={url}
        >{url}</a>
      );

      last = match.index + url.length;
    }

    const tail = text.slice(last);
    if (tail) parts.push(query ? highlightText(tail, query) : tail);

    return parts.length > 0 ? parts : text;
  };
}

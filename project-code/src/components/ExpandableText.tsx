import { useState } from 'react';

const EXPAND_CHAR_LIMIT = 600;

interface ExpandableTextProps {
  content: string;
  highlight?: string;
  renderFn: (text: string, query?: string) => React.ReactNode;
  hideExpandButton?: boolean;
}

export function ExpandableText({ content, highlight, renderFn, hideExpandButton }: ExpandableTextProps) {
  const [expanded, setExpanded] = useState(false);
  const isLong = content.length > EXPAND_CHAR_LIMIT;
  const displayContent = isLong && !expanded ? content.slice(0, EXPAND_CHAR_LIMIT) : content;

  return (
    <p className={`message-text ${hideExpandButton ? '' : 'emoji-support'}`} key={expanded ? 'expanded' : 'collapsed'}>
      {renderFn(displayContent, highlight)}
      {isLong && !expanded && <span className="message-text-ellipsis">…</span>}
      {isLong && !hideExpandButton && (
        <button 
          className="expand-btn" 
          onClick={(e) => {
            e.preventDefault();
            e.stopPropagation();
            setExpanded(!expanded);
          }}
        >
          {expanded ? "Show less" : "Show more"}
        </button>
      )}
    </p>
  );
}

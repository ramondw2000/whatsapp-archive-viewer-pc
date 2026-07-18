import { useEffect, useState } from "react";

interface LoadingDotsProps {
  text: string;
  intervalMs?: number;
}

// Cycles the trailing dots (none -> . -> .. -> ... -> none -> ...) so a long-running
// operation still visibly reads as "in progress" instead of looking hung.
export function LoadingDots({ text, intervalMs = 450 }: LoadingDotsProps) {
  const [dots, setDots] = useState(0);

  useEffect(() => {
    const id = setInterval(() => setDots(d => (d + 1) % 4), intervalMs);
    return () => clearInterval(id);
  }, [intervalMs]);

  return (
    <>
      {text}
      {/* Fixed-width slot so the dot count changing doesn't change this element's total
          width — otherwise a centered parent re-centers on every tick, and "text" itself
          visibly shifts sideways instead of just the trailing dots growing. */}
      <span style={{ display: "inline-block", width: "1.5em", textAlign: "left" }}>
        {".".repeat(dots)}
      </span>
    </>
  );
}

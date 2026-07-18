// Fraction of the display's own usable width below which the two-pane (sidebar + chat)
// layout no longer has room to stay legible, so the app falls back to single-pane
// navigation instead. Expressed as a fraction of the *screen*, not a fixed pixel width,
// so "half screen" reliably means the same thing on a 1366px laptop display and a 4K
// monitor — a fixed px threshold would trip well before half screen on a large display,
// and well after half screen on a small one.
const MOBILE_LAYOUT_SCREEN_FRACTION = 0.5;

// Absolute floor regardless of screen size — even on a very small/low-DPI display, the
// two-pane layout needs at least this many CSS px of window width to render usably.
const MOBILE_LAYOUT_MIN_WIDTH = 700;

export function shouldUseMobileLayout(): boolean {
  const screenWidth = window.screen?.availWidth || window.screen?.width || window.innerWidth;
  const threshold = Math.max(MOBILE_LAYOUT_MIN_WIDTH, screenWidth * MOBILE_LAYOUT_SCREEN_FRACTION);
  return window.innerWidth <= threshold;
}

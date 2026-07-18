const BASE_WIDTH = 420;
const GROWTH_THRESHOLD = 10000;
const PX_PER_CHAR = 9;

// Below the threshold, the search box's default width already fits "9.999/9.999" (the
// widest count possible under it) with the input compressing to make room. Past it, the
// input alone can't keep shrinking forever, so the whole box grows instead. Sized off the
// *total* result count's digit length for both halves of the "current/total" string (i.e.
// assuming the worst case of "10.000/10.000") rather than the actual current cursor
// position, so the box doesn't keep resizing as the user pages through results — only the
// total result count (which is stable per search) changes the width.
export function getSearchOverlayWidth(totalResults: number): number {
  if (totalResults < GROWTH_THRESHOLD) return BASE_WIDTH;
  const digitsLength = totalResults.toLocaleString().length;
  const worstCaseLength = digitsLength * 2 + 1; // e.g. "10.000/10.000"
  const baselineLength = (GROWTH_THRESHOLD - 1).toLocaleString().length * 2 + 1; // "9.999/9.999"
  const extraChars = Math.max(0, worstCaseLength - baselineLength);
  return BASE_WIDTH + extraChars * PX_PER_CHAR;
}

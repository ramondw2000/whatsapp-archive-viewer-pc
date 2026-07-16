// Numeric per-component comparison (1.2.0 > 1.10.0 would sort wrong as plain strings) so a
// release only counts as an update if it's actually newer, not merely a different tag — e.g. a
// dev build ahead of the last published release must not be told to "update" backwards to it.
export function isNewerVersion(candidate: string, current: string): boolean {
  const parse = (v: string) => v.split(".").map(n => parseInt(n, 10) || 0);
  const a = parse(candidate);
  const b = parse(current);
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const ai = a[i] ?? 0;
    const bi = b[i] ?? 0;
    if (ai !== bi) return ai > bi;
  }
  return false;
}

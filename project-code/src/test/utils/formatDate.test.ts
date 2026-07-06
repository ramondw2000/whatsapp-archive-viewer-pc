import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { formatDate } from "../../utils/formatDate";

describe("formatDate", () => {
  // We freeze time to a fixed date so today/yesterday tests are deterministic.
  // Fixed: 2024-04-12 (Friday)
  const FIXED_NOW = new Date(2024, 3, 12); // month is 0-indexed

  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(FIXED_NOW);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  // ── slash full year (dd/mm/yyyy) ──────────────────────────────────────────

  it("slash full year: returns formatted date for a past date", () => {
    const result = formatDate("17/06/2017, 00:04");
    expect(result).toMatch(/17/);
    expect(result).toMatch(/2017/);
  });

  it("slash full year: returns 'Today' when date matches today", () => {
    // Today is 12/04/2024
    expect(formatDate("12/04/2024, 14:32")).toBe("Today");
  });

  it("slash full year: returns 'Yesterday' when date is one day ago", () => {
    expect(formatDate("11/04/2024, 09:00")).toBe("Yesterday");
  });

  it("slash full year: formats a known past date correctly", () => {
    // 1 January 2020
    const result = formatDate("01/01/2020, 10:00");
    expect(result).toContain("2020");
    expect(result).toContain("1");   // day 1
  });

  // ── slash short year (dd/mm/yy) ───────────────────────────────────────────

  it("slash short year: interprets 2-digit year as 2000+", () => {
    // 12/04/24 → 2024-04-12 → Today
    expect(formatDate("12/04/24, 14:00")).toBe("Today");
  });

  it("slash short year: past 2-digit year returns formatted string", () => {
    const result = formatDate("17/06/17, 00:04");
    expect(result).toContain("2017");
  });

  // ── dash format (dd-mm-yyyy) ──────────────────────────────────────────────

  it("dash full year: returns formatted date for a past date", () => {
    const result = formatDate("17-06-2017 00:04");
    expect(result).toContain("2017");
  });

  it("dash full year: returns 'Today' when date matches today", () => {
    expect(formatDate("12-04-2024 14:32")).toBe("Today");
  });

  it("dash full year: returns 'Yesterday'", () => {
    expect(formatDate("11-04-2024 09:00")).toBe("Yesterday");
  });

  // ── dot format (dd.mm.yyyy) ───────────────────────────────────────────────

  it("dot full year: returns formatted date for a past date", () => {
    const result = formatDate("17.06.2017 00:04");
    expect(result).toContain("2017");
  });

  it("dot full year: returns 'Today' when date matches today", () => {
    expect(formatDate("12.04.2024 14:32")).toBe("Today");
  });

  it("dot full year: returns 'Yesterday'", () => {
    expect(formatDate("11.04.2024 09:00")).toBe("Yesterday");
  });

  // ── edge cases ────────────────────────────────────────────────────────────

  it("unrecognised format: returns the raw timestamp unchanged", () => {
    const raw = "not-a-date";
    expect(formatDate(raw)).toBe(raw);
  });

  it("empty string: returns the input unchanged", () => {
    expect(formatDate("")).toBe("");
  });

  it("partial match without full date pattern: returns raw input", () => {
    // Only a time, no date
    expect(formatDate("14:32")).toBe("14:32");
  });
});

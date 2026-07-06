import { describe, it, expect } from "vitest";
import { formatTime } from "../../utils/formatTime";

describe("formatTime", () => {
  it("extracts HH:MM from a full WhatsApp Android timestamp", () => {
    expect(formatTime("12/04/2024, 14:32 - Alice: Hello")).toBe("14:32");
  });

  it("extracts HH:MM from a full iOS bracket timestamp", () => {
    expect(formatTime("[12/04/2024, 09:05] Alice: Hi")).toBe("09:05");
  });

  it("extracts HH:MM when timestamp contains seconds", () => {
    expect(formatTime("12/04/2024, 14:32:45")).toBe("14:32");
  });

  it("extracts HH:MM from a standalone time string", () => {
    expect(formatTime("07:00")).toBe("07:00");
  });

  it("returns original string when no HH:MM pattern is found", () => {
    expect(formatTime("no time here")).toBe("no time here");
  });

  it("returns original string for empty input", () => {
    expect(formatTime("")).toBe("");
  });

  it("handles midnight (00:00) correctly", () => {
    expect(formatTime("17-06-2017 00:04 - Bob: Hey")).toBe("00:04");
  });

  it("handles single-digit hours that are zero-padded in the match", () => {
    // The regex requires exactly \d{2}:\d{2}
    expect(formatTime("12/04/2024, 09:05")).toBe("09:05");
  });
});

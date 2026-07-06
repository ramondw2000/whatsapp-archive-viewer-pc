import { describe, it, expect } from "vitest";
import {
  parsePhoneNumber,
  formatPhoneNumber,
  getPlaceholder,
  validatePhoneInput,
  phoneOptions,
} from "../../utils/phoneNumber";

describe("phoneOptions", () => {
  it("has at least one entry", () => {
    expect(phoneOptions.length).toBeGreaterThan(0);
  });

  it("every option has required fields", () => {
    for (const opt of phoneOptions) {
      expect(opt.id).toBeTruthy();
      expect(opt.dial).toMatch(/^\+\d+$/);
      expect(opt.code).toBeTruthy();
      expect(opt.name).toBeTruthy();
      expect(opt.flag).toBeTruthy();
    }
  });
});

describe("parsePhoneNumber", () => {
  it("returns default NL-06 with empty local when given empty string", () => {
    const result = parsePhoneNumber("");
    expect(result.optionId).toBe("NL-06");
    expect(result.localNumber).toBe("");
  });

  it("parses a Netherlands mobile number (+31 6...)", () => {
    // +31 6 12345678  — dial +31, prefix 06, local 12345678
    const result = parsePhoneNumber("+31 612345678");
    expect(result.optionId).toBe("NL-06");
    expect(result.localNumber).toBe("12345678");
  });

  it("parses a Belgium mobile number (+32 04...)", () => {
    const result = parsePhoneNumber("+32 412345678");
    expect(result.optionId).toBe("BE-04");
  });

  it("parses a UK mobile number (+44 07...)", () => {
    const result = parsePhoneNumber("+44 712345678");
    expect(result.optionId).toBe("GB-07");
  });

  it("falls back for a US number (+1 ...) because US has no prefix to match on", () => {
    // US option has prefix:"" so prefixClean="" → the guard `if (prefixClean && ...)` never
    // matches, meaning parsePhoneNumber cannot detect US numbers and falls back to NL-06.
    const result = parsePhoneNumber("+1 2025551234");
    expect(result.optionId).toBe("NL-06");
  });

  it("falls back to NL-06 for unrecognised dial code", () => {
    const result = parsePhoneNumber("+999123456");
    expect(result.optionId).toBe("NL-06");
  });
});

describe("formatPhoneNumber", () => {
  it("formats NL-06 with local number into international format", () => {
    expect(formatPhoneNumber("NL-06", "12345678")).toBe("+31 12345678");
  });

  it("strips leading zeros from local number", () => {
    expect(formatPhoneNumber("NL-06", "012345678")).toBe("+31 12345678");
  });

  it("strips spaces from local number", () => {
    expect(formatPhoneNumber("NL-06", "123 456 78")).toBe("+31 12345678");
  });

  it("strips dashes from local number", () => {
    expect(formatPhoneNumber("NL-06", "123-456-78")).toBe("+31 12345678");
  });

  it("uses first option as fallback for unknown optionId", () => {
    const result = formatPhoneNumber("UNKNOWN", "12345678");
    expect(result).toContain("+");
  });

  it("formats US number correctly (no prefix stripping needed)", () => {
    expect(formatPhoneNumber("US", "2025551234")).toBe("+1 2025551234");
  });
});

describe("getPlaceholder", () => {
  it("returns a non-empty string for NL-06", () => {
    const placeholder = getPlaceholder("NL-06");
    expect(placeholder).toBeTruthy();
    expect(typeof placeholder).toBe("string");
  });

  it("returns a placeholder without a leading zero for NL-06 (international format)", () => {
    // Prefix "06" → stripped to "6" → placeholder starts with 6
    const placeholder = getPlaceholder("NL-06");
    expect(placeholder.startsWith("6")).toBe(true);
  });

  it("returns a placeholder for US (no prefix)", () => {
    const placeholder = getPlaceholder("US");
    expect(placeholder).toBeTruthy();
  });

  it("falls back to first option for unknown optionId", () => {
    const placeholder = getPlaceholder("UNKNOWN");
    expect(typeof placeholder).toBe("string");
  });
});

describe("validatePhoneInput", () => {
  it("allows digits", () => {
    expect(validatePhoneInput("1234567890")).toBe("1234567890");
  });

  it("allows spaces", () => {
    expect(validatePhoneInput("123 456")).toBe("123 456");
  });

  it("allows dashes", () => {
    expect(validatePhoneInput("123-456")).toBe("123-456");
  });

  it("strips plus signs", () => {
    expect(validatePhoneInput("+31612345678")).toBe("31612345678");
  });

  it("strips letters", () => {
    expect(validatePhoneInput("abc123")).toBe("123");
  });

  it("strips parentheses", () => {
    expect(validatePhoneInput("(020) 123-4567")).toBe("020 123-4567");
  });

  it("returns empty string for input with only invalid characters", () => {
    expect(validatePhoneInput("@#$%")).toBe("");
  });

  it("returns empty string for empty input", () => {
    expect(validatePhoneInput("")).toBe("");
  });
});

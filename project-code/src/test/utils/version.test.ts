import { describe, it, expect } from "vitest";
import { isNewerVersion } from "../../utils/version";

describe("isNewerVersion", () => {
  it("returns true when the candidate's patch version is higher", () => {
    expect(isNewerVersion("0.1.1", "0.1.0")).toBe(true);
  });

  it("returns true when the candidate's minor version is higher", () => {
    expect(isNewerVersion("0.2.0", "0.1.9")).toBe(true);
  });

  it("returns true when the candidate's major version is higher", () => {
    expect(isNewerVersion("1.0.0", "0.9.9")).toBe(true);
  });

  it("returns false when versions are equal", () => {
    expect(isNewerVersion("0.1.1", "0.1.1")).toBe(false);
  });

  it("returns false when the candidate is older than current — a dev build ahead of the last published release must not be told to update backwards", () => {
    expect(isNewerVersion("0.1.0", "0.1.1")).toBe(false);
  });

  it("compares numerically, not lexicographically (1.2.0 vs 1.10.0)", () => {
    expect(isNewerVersion("1.10.0", "1.2.0")).toBe(true);
    expect(isNewerVersion("1.2.0", "1.10.0")).toBe(false);
  });

  it("treats a missing component as 0", () => {
    expect(isNewerVersion("0.2", "0.1.9")).toBe(true);
  });
});

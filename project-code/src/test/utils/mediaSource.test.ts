import { describe, it, expect } from "vitest";
import "../mocks/tauri";
import { canUseAssetUrl, cleanMediaFilename, joinMediaPath } from "../../utils/mediaSource";

describe("mediaSource helpers", () => {
  it("strips WhatsApp direction marks and zero-width characters", () => {
    expect(cleanMediaFilename("\u200E\u202AIMG-1.jpg\u202C\uFEFF\u200B")).toBe("IMG-1.jpg");
  });

  it("uses asset URLs for browser-native formats only", () => {
    expect(canUseAssetUrl("a.jpg")).toBe(true);
    expect(canUseAssetUrl("a.MP4")).toBe(true);
    expect(canUseAssetUrl("a.opus")).toBe(true);
    expect(canUseAssetUrl("a.tif")).toBe(false);
    expect(canUseAssetUrl("a.tiff")).toBe(false);
    expect(canUseAssetUrl("a.heic")).toBe(false);
    expect(canUseAssetUrl("noextension")).toBe(false);
    expect(canUseAssetUrl("..")).toBe(false);
  });

  it("joins using the separator of the base dir and keeps only the basename", () => {
    expect(joinMediaPath("C:\\x\\media", "\u200Efoo.jpg")).toBe("C:\\x\\media\\foo.jpg");
    expect(joinMediaPath("/home/x/media/", "foo.jpg")).toBe("/home/x/media/foo.jpg");
    expect(joinMediaPath("/m", "../../etc/passwd")).toBe("/m/passwd");
  });
});

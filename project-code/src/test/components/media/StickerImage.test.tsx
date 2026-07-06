import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";

import "../../mocks/tauri";
import { mockInvoke } from "../../mocks/tauri";
import { StickerImage } from "../../../components/media/StickerImage";

describe("StickerImage", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders null initially (no src yet)", () => {
    mockInvoke.mockReturnValue(new Promise(() => {})); // never resolves
    const { container } = render(<StickerImage chatId="c1" filename="STK-001.webp" />);
    expect(container.firstChild).toBeNull();
  });

  it("renders an <img> when invoke resolves with base64 data", async () => {
    mockInvoke.mockResolvedValue("data:image/webp;base64,abc123");

    render(<StickerImage chatId="c1" filename="STK-001.webp" />);

    await waitFor(() => {
      const img = screen.getByRole("img", { name: /sticker/i });
      expect(img).toBeInTheDocument();
      expect(img).toHaveAttribute("src", "data:image/webp;base64,abc123");
    });
  });

  it("renders MediaFallback when invoke rejects", async () => {
    mockInvoke
      .mockRejectedValueOnce(new Error("not found"))  // get_media_as_base64
      .mockResolvedValueOnce(false);                   // check_file_in_zip

    render(<StickerImage chatId="c1" filename="STK-missing.webp" />);

    await waitFor(() => {
      expect(screen.getByText("STK-missing.webp")).toBeInTheDocument();
    });
  });

  it("strips invisible Unicode characters from filename before invoking", async () => {
    mockInvoke.mockResolvedValue("data:image/webp;base64,xyz");

    const lrm = "\u200E";
    render(<StickerImage chatId="c1" filename={`${lrm}STK-001.webp`} />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "get_media_as_base64",
        expect.objectContaining({ filename: "STK-001.webp" })
      );
    });
  });
});

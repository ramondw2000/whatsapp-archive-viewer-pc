import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";

import "../../mocks/tauri";
import { mockInvoke } from "../../mocks/tauri";
import { StickerImage } from "../../../components/media/StickerImage";

const BASE_DIR = "C:\\Users\\me\\AppData\\Roaming\\WhatsAppArchiveViewer\\chats\\c1\\media";

function mockCommands(overrides: Record<string, (args: any) => unknown> = {}) {
  mockInvoke.mockImplementation((cmd: string, args: any) => {
    if (overrides[cmd]) return Promise.resolve(overrides[cmd](args));
    if (cmd === "get_media_base_dir") return Promise.resolve(BASE_DIR);
    return Promise.resolve("");
  });
}

describe("StickerImage", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders null initially (no src yet)", () => {
    mockInvoke.mockReturnValue(new Promise(() => {})); // never resolves
    const { container } = render(<StickerImage chatId="pending" filename="STK-001.webp" />);
    expect(container.firstChild).toBeNull();
  });

  it("loads browser-native files straight from disk via the asset URL", async () => {
    mockCommands();

    render(<StickerImage chatId="c1" filename="STK-001.webp" />);

    await waitFor(() => {
      const img = screen.getByRole("img", { name: /sticker/i });
      expect(img).toHaveAttribute("src", `asset://localhost/${encodeURIComponent(BASE_DIR + "\\STK-001.webp")}`);
    });
    expect(mockInvoke).not.toHaveBeenCalledWith("get_media_as_base64", expect.anything());
  });

  it("falls back to base64 over IPC when the asset URL fails to load", async () => {
    mockCommands({ get_media_as_base64: () => "data:image/webp;base64,abc123" });

    render(<StickerImage chatId="c1" filename="STK-002.webp" />);

    const img = await screen.findByRole("img", { name: /sticker/i });
    fireEvent.error(img);

    await waitFor(() => {
      expect(screen.getByRole("img", { name: /sticker/i })).toHaveAttribute("src", "data:image/webp;base64,abc123");
    });
  });

  it("uses the IPC route directly for formats the browser cannot render (TIFF)", async () => {
    mockCommands({ get_media_as_base64: () => "data:image/png;base64,tiffpng" });

    render(<StickerImage chatId="c1" filename="scan.tif" />);

    await waitFor(() => {
      expect(screen.getByRole("img", { name: /sticker/i })).toHaveAttribute("src", "data:image/png;base64,tiffpng");
    });
    expect(mockInvoke).toHaveBeenCalledWith("get_media_as_base64", expect.objectContaining({ filename: "scan.tif" }));
  });

  it("renders MediaFallback when every route fails", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_media_as_base64") return Promise.reject(new Error("not found"));
      if (cmd === "check_file_in_zip") return Promise.resolve(false);
      return Promise.resolve(BASE_DIR);
    });

    render(<StickerImage chatId="c1" filename="STK-missing.webp" />);
    fireEvent.error(await screen.findByRole("img", { name: /sticker/i }));

    await waitFor(() => {
      expect(screen.getByText("STK-missing.webp")).toBeInTheDocument();
    });
  });

  it("strips invisible Unicode characters from the filename before building the path", async () => {
    mockCommands();

    const lrm = "\u200E";
    render(<StickerImage chatId="c1" filename={`${lrm}STK-003.webp`} />);

    await waitFor(() => {
      const src = screen.getByRole("img", { name: /sticker/i }).getAttribute("src")!;
      expect(src).toContain(encodeURIComponent("\\STK-003.webp"));
      expect(src).not.toContain(encodeURIComponent(lrm));
    });
  });

  it("strips invisible Unicode characters before invoking the IPC fallback", async () => {
    mockCommands({ get_media_as_base64: () => "data:image/png;base64,xyz" });

    const lrm = "\u200E";
    render(<StickerImage chatId="c1" filename={`${lrm}scan.tif`} />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith(
        "get_media_as_base64",
        expect.objectContaining({ filename: "scan.tif" })
      );
    });
  });
});

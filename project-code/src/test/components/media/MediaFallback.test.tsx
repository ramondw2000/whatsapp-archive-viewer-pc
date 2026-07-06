import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

// Must be set up before importing the component
import "../../mocks/tauri";
import { mockInvoke } from "../../mocks/tauri";
import { MediaFallback } from "../../../components/media/MediaFallback";

describe("MediaFallback", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders the filename", () => {
    render(<MediaFallback filename="photo.jpg" />);
    expect(screen.getByText("photo.jpg")).toBeInTheDocument();
  });

  it("shows 'File not found' when existsInZip is false", () => {
    render(<MediaFallback filename="photo.jpg" existsInZip={false} />);
    expect(screen.getByText("File not found")).toBeInTheDocument();
  });

  it("shows 'File in ZIP (can restore)' when existsInZip is true", () => {
    render(<MediaFallback filename="photo.jpg" existsInZip={true} chatId="c1" />);
    expect(screen.getByText("File in ZIP (can restore)")).toBeInTheDocument();
  });

  it("shows restore button when existsInZip is true and chatId provided", () => {
    render(<MediaFallback filename="photo.jpg" existsInZip={true} chatId="c1" />);
    expect(screen.getByRole("button", { name: /restore from zip/i })).toBeInTheDocument();
  });

  it("does not show restore button when existsInZip is false", () => {
    render(<MediaFallback filename="photo.jpg" existsInZip={false} chatId="c1" />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("does not show restore button when chatId is not provided", () => {
    render(<MediaFallback filename="photo.jpg" existsInZip={true} />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("calls extract_file_from_zip invoke and then onRestored on success", async () => {
    const onRestored = vi.fn();
    mockInvoke.mockResolvedValue(undefined);

    render(
      <MediaFallback filename="photo.jpg" existsInZip={true} chatId="c1" onRestored={onRestored} />
    );
    await userEvent.click(screen.getByRole("button", { name: /restore from zip/i }));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("extract_file_from_zip", {
        chatId: "c1",
        filename: "photo.jpg",
      });
      expect(onRestored).toHaveBeenCalledTimes(1);
    });
  });

  it("shows 'Restoring...' while the invoke is in-flight", async () => {
    // Invoke never resolves during this test
    mockInvoke.mockReturnValue(new Promise(() => {}));

    render(<MediaFallback filename="photo.jpg" existsInZip={true} chatId="c1" />);
    await userEvent.click(screen.getByRole("button", { name: /restore from zip/i }));

    expect(screen.getByRole("button", { name: /restoring/i })).toBeDisabled();
  });
});

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";

import "../../mocks/tauri";
import { mockInvoke } from "../../mocks/tauri";
import { ProfileImage } from "../../../components/ProfileImage";

describe("ProfileImage", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders nothing when photoPath is null", () => {
    const { container } = render(<ProfileImage photoPath={null} alt="user" />);
    expect(container.firstChild).toBeNull();
  });

  it("renders nothing when photoPath is undefined", () => {
    const { container } = render(<ProfileImage photoPath={undefined} alt="user" />);
    expect(container.firstChild).toBeNull();
  });

  it("renders nothing when photoPath is empty string", async () => {
    const { container } = render(<ProfileImage photoPath="" alt="user" />);
    // Empty string is falsy, so useEffect short-circuits to setSrc(null)
    await waitFor(() => {
      expect(container.firstChild).toBeNull();
    });
  });

  it("renders an img tag with base64 src when invoke succeeds", async () => {
    mockInvoke.mockResolvedValue("data:image/jpeg;base64,abc123");

    render(<ProfileImage photoPath="/some/path/photo.jpg" alt="Alice" />);

    await waitFor(() => {
      const img = screen.getByRole("img", { name: "Alice" });
      expect(img).toBeInTheDocument();
      expect(img).toHaveAttribute("src", "data:image/jpeg;base64,abc123");
    });
  });

  it("renders nothing when invoke rejects", async () => {
    mockInvoke.mockRejectedValue(new Error("not found"));

    const { container } = render(<ProfileImage photoPath="/missing/photo.jpg" alt="Bob" />);

    await waitFor(() => {
      expect(container.firstChild).toBeNull();
    });
  });

  it("applies className to the img element", async () => {
    mockInvoke.mockResolvedValue("data:image/jpeg;base64,xyz");

    render(<ProfileImage photoPath="/photo.jpg" alt="Carol" className="profile-pic" />);

    await waitFor(() => {
      expect(screen.getByRole("img")).toHaveClass("profile-pic");
    });
  });

  it("calls read_file_as_base64 with the correct path", async () => {
    mockInvoke.mockResolvedValue("data:image/jpeg;base64,xyz");

    render(<ProfileImage photoPath="/data/photo.jpg" alt="Dave" />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("read_file_as_base64", {
        path: "/data/photo.jpg",
      });
    });
  });
});

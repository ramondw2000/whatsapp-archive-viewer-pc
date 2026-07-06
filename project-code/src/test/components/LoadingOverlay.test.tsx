import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { LoadingOverlay } from "../../components/LoadingOverlay";

describe("LoadingOverlay", () => {
  it("renders the message text", () => {
    render(<LoadingOverlay message="Importing chat..." />);
    expect(screen.getByText("Importing chat...")).toBeInTheDocument();
  });

  it("renders the detail text when provided", () => {
    render(<LoadingOverlay message="Loading" detail="Please wait" />);
    expect(screen.getByText("Please wait")).toBeInTheDocument();
  });

  it("does not render detail element when detail is not provided", () => {
    render(<LoadingOverlay message="Loading" />);
    expect(screen.queryByText("Please wait")).not.toBeInTheDocument();
  });

  it("renders a spinner element", () => {
    const { container } = render(<LoadingOverlay message="Loading" />);
    expect(container.querySelector(".loading-spinner")).toBeInTheDocument();
  });

  it("renders the progress bar", () => {
    const { container } = render(<LoadingOverlay message="Loading" />);
    expect(container.querySelector(".loading-bar")).toBeInTheDocument();
  });

  it("wraps everything in a loading-overlay div", () => {
    const { container } = render(<LoadingOverlay message="Loading" />);
    expect(container.querySelector(".loading-overlay")).toBeInTheDocument();
  });
});

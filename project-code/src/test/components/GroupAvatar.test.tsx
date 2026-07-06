import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { GroupAvatar } from "../../components/GroupAvatar";

describe("GroupAvatar", () => {
  it("renders initials for a single participant", () => {
    render(<GroupAvatar participants={["Alice"]} />);
    expect(screen.getByText("A")).toBeInTheDocument();
  });

  it("renders two-letter initials for two-word names", () => {
    render(<GroupAvatar participants={["Alice Smith"]} />);
    expect(screen.getByText("AS")).toBeInTheDocument();
  });

  it("renders initials for multiple participants (up to 4)", () => {
    const { container } = render(
      <GroupAvatar participants={["Alice", "Bob", "Carol", "Dave"]} />
    );
    const cells = container.querySelectorAll(".group-avatar-cell:not(.group-avatar-cell--empty)");
    expect(cells.length).toBe(4);
  });

  it("caps displayed participants at 4 even when more are provided", () => {
    const { container } = render(
      <GroupAvatar participants={["A", "B", "C", "D", "E", "F"]} />
    );
    // Cells should be at most 4 non-empty slots
    const cells = container.querySelectorAll(".group-avatar-cell:not(.group-avatar-cell--empty)");
    expect(cells.length).toBeLessThanOrEqual(4);
  });

  it("adds an empty cell placeholder when only one participant is given", () => {
    const { container } = render(<GroupAvatar participants={["Alice"]} />);
    expect(container.querySelector(".group-avatar-cell--empty")).toBeInTheDocument();
  });

  it("does not add empty cell when two or more participants are given", () => {
    const { container } = render(<GroupAvatar participants={["Alice", "Bob"]} />);
    expect(container.querySelector(".group-avatar-cell--empty")).not.toBeInTheDocument();
  });

  it("applies the normal size class by default", () => {
    const { container } = render(<GroupAvatar participants={["Alice"]} />);
    expect(container.querySelector(".group-avatar--normal")).toBeInTheDocument();
  });

  it("applies the large size class when size='large'", () => {
    const { container } = render(<GroupAvatar participants={["Alice"]} size="large" />);
    expect(container.querySelector(".group-avatar--large")).toBeInTheDocument();
  });

  it("applies the dialog size class when size='dialog'", () => {
    const { container } = render(<GroupAvatar participants={["Alice"]} size="dialog" />);
    expect(container.querySelector(".group-avatar--dialog")).toBeInTheDocument();
  });

  it("renders nothing visible for an empty participants array", () => {
    const { container } = render(<GroupAvatar participants={[]} />);
    const cells = container.querySelectorAll(".group-avatar-cell:not(.group-avatar-cell--empty)");
    expect(cells.length).toBe(0);
  });
});

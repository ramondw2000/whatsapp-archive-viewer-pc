import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { JumpButton } from "../../components/JumpButton";

describe("JumpButton", () => {
  it("renders nothing when visible is false", () => {
    const { container } = render(
      <JumpButton direction="up" onClick={() => {}} visible={false} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders the button when visible is true", () => {
    render(<JumpButton direction="up" onClick={() => {}} visible={true} />);
    expect(screen.getByRole("button")).toBeInTheDocument();
  });

  it("up button shows upward arrow", () => {
    render(<JumpButton direction="up" onClick={() => {}} visible={true} />);
    expect(screen.getByRole("button").textContent).toContain("↑");
  });

  it("down button shows downward arrow", () => {
    render(<JumpButton direction="down" onClick={() => {}} visible={true} />);
    expect(screen.getByRole("button").textContent).toContain("↓");
  });

  it("up button has correct title", () => {
    render(<JumpButton direction="up" onClick={() => {}} visible={true} />);
    expect(screen.getByRole("button")).toHaveAttribute("title", "Jump to top");
  });

  it("down button has correct title", () => {
    render(<JumpButton direction="down" onClick={() => {}} visible={true} />);
    expect(screen.getByRole("button")).toHaveAttribute("title", "Jump to bottom");
  });

  it("calls onClick handler when clicked", async () => {
    const handler = vi.fn();
    render(<JumpButton direction="down" onClick={handler} visible={true} />);
    await userEvent.click(screen.getByRole("button"));
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("up button has correct CSS class", () => {
    render(<JumpButton direction="up" onClick={() => {}} visible={true} />);
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("jump-button--up");
  });

  it("down button has correct CSS class", () => {
    render(<JumpButton direction="down" onClick={() => {}} visible={true} />);
    const btn = screen.getByRole("button");
    expect(btn.className).toContain("jump-button--down");
  });
});

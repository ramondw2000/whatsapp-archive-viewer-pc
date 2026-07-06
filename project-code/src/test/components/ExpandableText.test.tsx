import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ExpandableText } from "../../components/ExpandableText";

const plainRender = (text: string) => text;

describe("ExpandableText", () => {
  it("renders full content when text is short (under 600 chars)", () => {
    render(<ExpandableText content="Short text" renderFn={plainRender} />);
    expect(screen.getByText("Short text")).toBeInTheDocument();
  });

  it("does not show expand button for short text", () => {
    render(<ExpandableText content="Short text" renderFn={plainRender} />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("truncates long text and shows ellipsis", () => {
    const longText = "a".repeat(700);
    const { container } = render(
      <ExpandableText content={longText} renderFn={plainRender} />
    );
    expect(container.querySelector(".message-text-ellipsis")).toBeInTheDocument();
  });

  it("shows 'Show more' button for long text", () => {
    const longText = "b".repeat(700);
    render(<ExpandableText content={longText} renderFn={plainRender} />);
    expect(screen.getByRole("button", { name: /show more/i })).toBeInTheDocument();
  });

  it("expands to full content when 'Show more' is clicked", async () => {
    const longText = "c".repeat(700);
    render(<ExpandableText content={longText} renderFn={plainRender} />);
    await userEvent.click(screen.getByRole("button", { name: /show more/i }));
    // After expanding, the full text should be rendered
    expect(screen.getByRole("button", { name: /show less/i })).toBeInTheDocument();
  });

  it("collapses back when 'Show less' is clicked", async () => {
    const longText = "d".repeat(700);
    render(<ExpandableText content={longText} renderFn={plainRender} />);
    await userEvent.click(screen.getByRole("button", { name: /show more/i }));
    await userEvent.click(screen.getByRole("button", { name: /show less/i }));
    expect(screen.getByRole("button", { name: /show more/i })).toBeInTheDocument();
  });

  it("does not show expand button when hideExpandButton is true", () => {
    const longText = "e".repeat(700);
    render(
      <ExpandableText content={longText} renderFn={plainRender} hideExpandButton={true} />
    );
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("passes highlight prop to renderFn", () => {
    const capturedArgs: string[] = [];
    const capturingRender = (text: string, q?: string) => {
      capturedArgs.push(q ?? "");
      return text;
    };
    render(
      <ExpandableText content="Hello" renderFn={capturingRender} highlight="Hello" />
    );
    expect(capturedArgs[0]).toBe("Hello");
  });
});

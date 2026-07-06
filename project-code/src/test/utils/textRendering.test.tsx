import { describe, it, expect, vi } from "vitest";
import { render } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { highlightText, createRenderMessageText } from "../../utils/textRendering";

// ── highlightText ─────────────────────────────────────────────────────────────

describe("highlightText", () => {
  it("returns the plain string when query is empty", () => {
    const result = highlightText("Hello World", "");
    expect(result).toBe("Hello World");
  });

  it("returns the plain string when query is only whitespace", () => {
    const result = highlightText("Hello World", "   ");
    expect(result).toBe("Hello World");
  });

  it("wraps matching substring in a <mark> element", () => {
    const { container } = render(<>{highlightText("Hello World", "World")}</>);
    const mark = container.querySelector("mark");
    expect(mark).not.toBeNull();
    expect(mark?.textContent).toBe("World");
    expect(mark?.className).toContain("search-highlight");
  });

  it("match is case-insensitive", () => {
    const { container } = render(<>{highlightText("Hello World", "world")}</>);
    const mark = container.querySelector("mark");
    expect(mark).not.toBeNull();
    expect(mark?.textContent?.toLowerCase()).toBe("world");
  });

  it("highlights multiple occurrences", () => {
    const { container } = render(<>{highlightText("foo bar foo", "foo")}</>);
    const marks = container.querySelectorAll("mark");
    expect(marks.length).toBe(2);
  });

  it("does not add <mark> when query not found in text", () => {
    const { container } = render(<>{highlightText("Hello World", "xyz")}</>);
    expect(container.querySelector("mark")).toBeNull();
  });

  it("escapes regex special characters in query", () => {
    // A query with '.' should not act as a wildcard
    const { container } = render(<>{highlightText("a.b a_b", "a.b")}</>);
    const marks = container.querySelectorAll("mark");
    // Should match only "a.b" (literal dot), not "a_b"
    expect(marks.length).toBe(1);
    expect(marks[0].textContent).toBe("a.b");
  });
});

// ── createRenderMessageText ───────────────────────────────────────────────────

describe("createRenderMessageText", () => {
  const onLinkClick = vi.fn();
  const renderMessageText = createRenderMessageText({ onLinkClick });

  it("returns content containing the text when there are no URLs", () => {
    // renderMessageText always returns a ReactNode array; render and check text
    const { container } = render(<>{renderMessageText("Hello World")}</>);
    expect(container.textContent).toBe("Hello World");
  });

  it("converts an https URL into a clickable <a> element", () => {
    const { container } = render(
      <>{renderMessageText("Visit https://example.com for info")}</>
    );
    const link = container.querySelector("a");
    expect(link).not.toBeNull();
    expect(link?.title).toContain("https://example.com");
    expect(link?.className).toContain("message-link");
  });

  it("prefixes www. links with https://", () => {
    const { container } = render(
      <>{renderMessageText("Go to www.example.com")}</>
    );
    const link = container.querySelector("a");
    expect(link?.title).toBe("https://www.example.com");
  });

  it("calls onLinkClick with the URL when link is clicked", async () => {
    onLinkClick.mockClear();
    const { container } = render(
      <>{renderMessageText("Visit https://example.com")}</>
    );
    const link = container.querySelector("a")!;
    await userEvent.click(link);
    expect(onLinkClick).toHaveBeenCalledWith("https://example.com");
  });

  it("text before and after URL is preserved as plain text", () => {
    const { container } = render(
      <>{renderMessageText("Before https://x.com after")}</>
    );
    expect(container.textContent).toContain("Before");
    expect(container.textContent).toContain("after");
  });

  it("renders multiple URLs as separate links", () => {
    const { container } = render(
      <>{renderMessageText("https://a.com and https://b.com")}</>
    );
    const links = container.querySelectorAll("a");
    expect(links.length).toBe(2);
  });

  it("highlights query matches in plain text segments", () => {
    const { container } = render(
      <>{renderMessageText("Hello World", "World")}</>
    );
    const mark = container.querySelector("mark");
    expect(mark).not.toBeNull();
    expect(mark?.textContent).toBe("World");
  });

  it("returns empty content for an empty string", () => {
    // empty string: parts is empty, function returns the raw string ""
    const result = renderMessageText("");
    expect(result).toBe("");
  });
});

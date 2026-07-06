import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LinkConfirmModal } from "../../components/LinkConfirmModal";

describe("LinkConfirmModal", () => {
  it("renders the URL", () => {
    render(
      <LinkConfirmModal url="https://example.com" onConfirm={() => {}} onCancel={() => {}} />
    );
    expect(screen.getByText("https://example.com")).toBeInTheDocument();
  });

  it("renders the heading 'Open link?'", () => {
    render(
      <LinkConfirmModal url="https://example.com" onConfirm={() => {}} onCancel={() => {}} />
    );
    expect(screen.getByRole("heading", { name: /open link/i })).toBeInTheDocument();
  });

  it("calls onConfirm when Open button is clicked", async () => {
    const onConfirm = vi.fn();
    render(
      <LinkConfirmModal url="https://example.com" onConfirm={onConfirm} onCancel={() => {}} />
    );
    await userEvent.click(screen.getByRole("button", { name: /open/i }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("calls onCancel when Cancel button is clicked", async () => {
    const onCancel = vi.fn();
    render(
      <LinkConfirmModal url="https://example.com" onConfirm={() => {}} onCancel={onCancel} />
    );
    await userEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("calls onCancel when the overlay backdrop is clicked", async () => {
    const onCancel = vi.fn();
    const { container } = render(
      <LinkConfirmModal url="https://example.com" onConfirm={() => {}} onCancel={onCancel} />
    );
    const overlay = container.querySelector(".dialog-overlay")!;
    await userEvent.click(overlay);
    expect(onCancel).toHaveBeenCalled();
  });

  it("does not call onCancel when clicking inside the dialog content", async () => {
    const onCancel = vi.fn();
    const { container } = render(
      <LinkConfirmModal url="https://example.com" onConfirm={() => {}} onCancel={onCancel} />
    );
    const content = container.querySelector(".dialog-content")!;
    await userEvent.click(content);
    expect(onCancel).not.toHaveBeenCalled();
  });
});

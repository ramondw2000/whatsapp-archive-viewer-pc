import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DeleteConfirmationDialog } from "../../../components/dialogs/DeleteConfirmationDialog";

const defaults = {
  chatId: "chat-1",
  chatName: "Alice",
  onSave: vi.fn(),
  onDelete: vi.fn(),
  onCancel: vi.fn(),
  hasChanges: true,
};

describe("DeleteConfirmationDialog", () => {
  it("renders the chat name in the confirmation text", () => {
    render(<DeleteConfirmationDialog {...defaults} />);
    expect(screen.getByText(/Alice/)).toBeInTheDocument();
  });

  it("calls onDelete when 'Delete Everything' is clicked", async () => {
    const onDelete = vi.fn();
    render(<DeleteConfirmationDialog {...defaults} onDelete={onDelete} />);
    await userEvent.click(screen.getByRole("button", { name: /delete everything/i }));
    expect(onDelete).toHaveBeenCalledTimes(1);
  });

  it("calls onSave when 'Keep Changes & Delete' is clicked and hasChanges is true", async () => {
    const onSave = vi.fn();
    render(<DeleteConfirmationDialog {...defaults} onSave={onSave} hasChanges={true} />);
    await userEvent.click(screen.getByRole("button", { name: /keep changes/i }));
    expect(onSave).toHaveBeenCalledTimes(1);
  });

  it("disables 'Keep Changes & Delete' button when hasChanges is false", () => {
    render(<DeleteConfirmationDialog {...defaults} hasChanges={false} />);
    const btn = screen.getByRole("button", { name: /keep changes/i });
    expect(btn).toBeDisabled();
  });

  it("calls onCancel when the close button is clicked", async () => {
    const onCancel = vi.fn();
    render(<DeleteConfirmationDialog {...defaults} onCancel={onCancel} />);
    await userEvent.click(screen.getByRole("button", { name: /×/i }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });
});

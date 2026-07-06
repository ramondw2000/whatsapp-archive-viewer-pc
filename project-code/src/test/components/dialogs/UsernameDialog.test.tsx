import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { UsernameDialog } from "../../../components/dialogs/UsernameDialog";

describe("UsernameDialog", () => {
  it("renders the welcome heading", () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={false} />);
    expect(screen.getByRole("heading")).toBeInTheDocument();
  });

  it("renders the text input", () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={false} />);
    expect(screen.getByRole("textbox")).toBeInTheDocument();
  });

  it("Continue button is disabled when input is empty", () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={false} />);
    expect(screen.getByRole("button", { name: /continue/i })).toBeDisabled();
  });

  it("Continue button enables after typing a name", async () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={false} />);
    await userEvent.type(screen.getByRole("textbox"), "Alice");
    expect(screen.getByRole("button", { name: /continue/i })).not.toBeDisabled();
  });

  it("calls onSubmit with trimmed name on form submission", async () => {
    const onSubmit = vi.fn();
    render(<UsernameDialog onSubmit={onSubmit} onCancel={() => {}} hasExistingName={false} />);
    await userEvent.type(screen.getByRole("textbox"), "  Alice  ");
    await userEvent.click(screen.getByRole("button", { name: /continue/i }));
    expect(onSubmit).toHaveBeenCalledWith("Alice");
  });

  it("Cancel button is disabled when hasExistingName is false", () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={false} />);
    expect(screen.getByRole("button", { name: /cancel/i })).toBeDisabled();
  });

  it("Cancel button is enabled when hasExistingName is true", () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={true} />);
    expect(screen.getByRole("button", { name: /cancel/i })).not.toBeDisabled();
  });

  it("calls onCancel when Cancel is clicked (with existing name)", async () => {
    const onCancel = vi.fn();
    render(<UsernameDialog onSubmit={() => {}} onCancel={onCancel} hasExistingName={true} />);
    await userEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(onCancel).toHaveBeenCalledTimes(1);
  });

  it("shows character counter that decrements as you type", async () => {
    render(<UsernameDialog onSubmit={() => {}} onCancel={() => {}} hasExistingName={false} />);
    expect(screen.getByText(/25 characters remaining/i)).toBeInTheDocument();
    await userEvent.type(screen.getByRole("textbox"), "Alice");
    expect(screen.getByText(/20 characters remaining/i)).toBeInTheDocument();
  });
});

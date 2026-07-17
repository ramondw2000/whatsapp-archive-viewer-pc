import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import "../../mocks/tauri";
import { mockInvoke } from "../../mocks/tauri";
import { LockScreen } from "../../../components/dialogs/LockScreen";

describe("LockScreen", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("renders the Enter PIN heading", () => {
    render(<LockScreen onUnlock={() => {}} onPinReset={() => {}} />);
    expect(screen.getByRole("heading", { name: /enter pin/i })).toBeInTheDocument();
  });

  it("Unlock button is disabled when input is empty", () => {
    render(<LockScreen onUnlock={() => {}} onPinReset={() => {}} />);
    expect(screen.getByRole("button", { name: /unlock/i })).toBeDisabled();
  });

  it("calls verify_pin with the entered PIN and onUnlock when it resolves true", async () => {
    mockInvoke.mockResolvedValue(true);
    const onUnlock = vi.fn();
    render(<LockScreen onUnlock={onUnlock} onPinReset={() => {}} />);
    const input = document.querySelector('input[type="password"]')!;
    await userEvent.type(input, "1234");
    await userEvent.click(screen.getByRole("button", { name: /unlock/i }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("verify_pin", { pin: "1234" });
      expect(onUnlock).toHaveBeenCalledTimes(1);
    });
  });

  it("shows an error and clears the input when verify_pin resolves false", async () => {
    mockInvoke.mockResolvedValue(false);
    const onUnlock = vi.fn();
    render(<LockScreen onUnlock={onUnlock} onPinReset={() => {}} />);
    const input = document.querySelector('input[type="password"]')! as HTMLInputElement;
    await userEvent.type(input, "9999");
    await userEvent.click(screen.getByRole("button", { name: /unlock/i }));
    await waitFor(() => {
      expect(screen.getByText(/incorrect pin/i)).toBeInTheDocument();
    });
    expect(input.value).toBe("");
    expect(onUnlock).not.toHaveBeenCalled();
  });

  it("fetches and shows the recovery question after clicking Forgot PIN?", async () => {
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === "get_recovery_question" ? Promise.resolve("What's your pet's name?") : Promise.resolve(undefined)
    );
    render(<LockScreen onUnlock={() => {}} onPinReset={() => {}} />);
    await userEvent.click(screen.getByText(/forgot pin/i));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_recovery_question");
      expect(screen.getByText(/what's your pet's name/i)).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: /reset lock/i })).toBeDisabled();
  });

  it("shows a no-recovery message instead of a reset form when get_recovery_question resolves null", async () => {
    mockInvoke.mockImplementation((cmd: string) =>
      cmd === "get_recovery_question" ? Promise.resolve(null) : Promise.resolve(undefined)
    );
    render(<LockScreen onUnlock={() => {}} onPinReset={() => {}} />);
    await userEvent.click(screen.getByText(/forgot pin/i));
    await waitFor(() => {
      expect(screen.getByText(/no recovery question is set/i)).toBeInTheDocument();
    });
    expect(screen.queryByRole("button", { name: /reset lock/i })).not.toBeInTheDocument();
  });

  it("submitting the correct recovery answer calls reset_pin_with_recovery_answer, then onPinReset, then onUnlock", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_recovery_question") return Promise.resolve("What's your pet's name?");
      if (cmd === "reset_pin_with_recovery_answer") return Promise.resolve(undefined);
      return Promise.resolve(undefined);
    });
    const onUnlock = vi.fn();
    const onPinReset = vi.fn();
    render(<LockScreen onUnlock={onUnlock} onPinReset={onPinReset} />);
    await userEvent.click(screen.getByText(/forgot pin/i));
    await waitFor(() => expect(screen.getByRole("button", { name: /reset lock/i })).toBeInTheDocument());
    const answerInput = screen.getByPlaceholderText("Answer");
    await userEvent.type(answerInput, "Rex");
    await userEvent.click(screen.getByRole("button", { name: /reset lock/i }));
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("reset_pin_with_recovery_answer", { answer: "Rex" });
      expect(onPinReset).toHaveBeenCalledTimes(1);
      expect(onUnlock).toHaveBeenCalledTimes(1);
    });
  });

  it("shows an error and does not unlock when the recovery answer is wrong", async () => {
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_recovery_question") return Promise.resolve("What's your pet's name?");
      if (cmd === "reset_pin_with_recovery_answer") return Promise.reject("Incorrect answer");
      return Promise.resolve(undefined);
    });
    const onUnlock = vi.fn();
    render(<LockScreen onUnlock={onUnlock} onPinReset={() => {}} />);
    await userEvent.click(screen.getByText(/forgot pin/i));
    await waitFor(() => expect(screen.getByRole("button", { name: /reset lock/i })).toBeInTheDocument());
    await userEvent.type(screen.getByPlaceholderText("Answer"), "wrong");
    await userEvent.click(screen.getByRole("button", { name: /reset lock/i }));
    await waitFor(() => {
      expect(screen.getByText(/incorrect answer/i)).toBeInTheDocument();
    });
    expect(onUnlock).not.toHaveBeenCalled();
  });
});

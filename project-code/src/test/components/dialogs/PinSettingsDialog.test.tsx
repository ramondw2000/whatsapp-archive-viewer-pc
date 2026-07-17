import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import "../../mocks/tauri";
import { mockInvoke } from "../../mocks/tauri";
import { PinSettingsDialog } from "../../../components/dialogs/PinSettingsDialog";

function passwordInputs(): HTMLInputElement[] {
  return Array.from(document.querySelectorAll('input[type="password"]'));
}

function submitButton(): HTMLButtonElement {
  return document.querySelector('button[type="submit"]')!;
}

function mockRecoveryData(question: string | null, answer: string | null, extra?: (cmd: string, args?: any) => any) {
  mockInvoke.mockImplementation((cmd: string, args?: any) => {
    if (cmd === "get_recovery_question") return Promise.resolve(question);
    if (cmd === "get_recovery_answer") return Promise.resolve(answer);
    if (extra) {
      const result = extra(cmd, args);
      if (result !== undefined) return result;
    }
    return Promise.resolve(undefined);
  });
}

describe("PinSettingsDialog", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("Set PIN mode shows a new-PIN field plus two blank recovery fields when nothing exists yet, and calls set_pin", async () => {
    mockRecoveryData(null, null);
    const onChanged = vi.fn();
    const onClose = vi.fn();
    render(<PinSettingsDialog hasPin={false} onClose={onClose} onChanged={onChanged} />);
    expect(screen.queryByRole("button", { name: /change pin/i })).not.toBeInTheDocument();
    const questionInput = await screen.findByPlaceholderText(/recovery question/i) as HTMLInputElement;
    const answerInput = screen.getByPlaceholderText("Answer") as HTMLInputElement;
    expect(questionInput.value).toBe("");
    expect(answerInput.value).toBe("");
    await userEvent.type(passwordInputs()[0], "1234");
    await userEvent.type(questionInput, "Pet's name?");
    await userEvent.type(answerInput, "Rex");
    await userEvent.click(submitButton());
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_pin", { pin: "1234", recoveryQuestion: "Pet's name?", recoveryAnswer: "Rex" });
      expect(onChanged).toHaveBeenCalledTimes(1);
      expect(onClose).toHaveBeenCalledTimes(1);
    });
  });

  it("Set PIN mode pre-fills both fields with a question/answer left over from a prior PIN, directly editable", async () => {
    mockRecoveryData("Pet's name?", "Rex");
    const onChanged = vi.fn();
    render(<PinSettingsDialog hasPin={false} onClose={() => {}} onChanged={onChanged} />);
    const questionInput = await screen.findByPlaceholderText(/recovery question/i) as HTMLInputElement;
    const answerInput = screen.getByPlaceholderText("Answer") as HTMLInputElement;
    await waitFor(() => {
      expect(questionInput.value).toBe("Pet's name?");
      expect(answerInput.value).toBe("Rex");
    });
    await userEvent.type(passwordInputs()[0], "1234");
    await userEvent.click(submitButton());
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("set_pin", { pin: "1234", recoveryQuestion: "Pet's name?", recoveryAnswer: "Rex" });
      expect(onChanged).toHaveBeenCalledTimes(1);
    });
  });

  it("defaults to Change PIN mode when a PIN already exists, showing three tabs", async () => {
    mockRecoveryData(null, null);
    render(<PinSettingsDialog hasPin={true} onClose={() => {}} onChanged={() => {}} />);
    const changePinTab = screen.getAllByRole("button", { name: /^change pin$/i })[0];
    expect(changePinTab).toHaveClass("active");
    expect(screen.getByRole("button", { name: /security question/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^remove pin$/i })).toBeInTheDocument();
    // Change PIN mode: current + new PIN, no recovery fields
    expect(passwordInputs().length).toBe(2);
    expect(screen.queryByPlaceholderText(/recovery question/i)).not.toBeInTheDocument();
  });

  it("Change PIN mode calls change_pin with current and new PIN, without touching the recovery question", async () => {
    mockRecoveryData(null, null);
    const onChanged = vi.fn();
    render(<PinSettingsDialog hasPin={true} onClose={() => {}} onChanged={onChanged} />);
    const [current, next] = passwordInputs();
    await userEvent.type(current, "1111");
    await userEvent.type(next, "2222");
    await userEvent.click(submitButton());
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("change_pin", { currentPin: "1111", newPin: "2222" });
      expect(mockInvoke).not.toHaveBeenCalledWith("set_pin", expect.anything());
      expect(onChanged).toHaveBeenCalledTimes(1);
    });
  });

  it("Change PIN mode shows an error and does not call onChanged when change_pin rejects", async () => {
    mockInvoke.mockRejectedValue("Incorrect current PIN");
    const onChanged = vi.fn();
    render(<PinSettingsDialog hasPin={true} onClose={() => {}} onChanged={onChanged} />);
    const [current, next] = passwordInputs();
    await userEvent.type(current, "9999");
    await userEvent.type(next, "2222");
    await userEvent.click(submitButton());
    await waitFor(() => {
      expect(screen.getByText(/incorrect current pin/i)).toBeInTheDocument();
    });
    expect(onChanged).not.toHaveBeenCalled();
  });

  it("Security Question tab keeps the recovery fields fully hidden until the current PIN actually verifies, then pre-fills both", async () => {
    mockRecoveryData("Pet's name?", "Rex", (cmd, args) => {
      if (cmd === "verify_pin") return Promise.resolve(args?.pin === "1111");
    });
    const onChanged = vi.fn();
    render(<PinSettingsDialog hasPin={true} onClose={() => {}} onChanged={onChanged} />);
    await userEvent.click(screen.getByRole("button", { name: /security question/i }));
    expect(screen.queryByPlaceholderText("Recovery question")).not.toBeInTheDocument();
    expect(screen.getByText(/enter your current pin above/i)).toBeInTheDocument();

    // Wrong PIN: fields stay hidden, an "Incorrect PIN" message shows instead
    await userEvent.type(passwordInputs()[0], "9999");
    await waitFor(() => expect(screen.getByText(/incorrect pin/i)).toBeInTheDocument());
    expect(screen.queryByPlaceholderText("Recovery question")).not.toBeInTheDocument();

    // Correct PIN: both fields appear, pre-filled with the current question and answer
    await userEvent.clear(passwordInputs()[0]);
    await userEvent.type(passwordInputs()[0], "1111");
    const questionInput = await screen.findByPlaceholderText("Recovery question") as HTMLInputElement;
    const answerInput = screen.getByPlaceholderText("Answer") as HTMLInputElement;
    await waitFor(() => {
      expect(questionInput.value).toBe("Pet's name?");
      expect(answerInput.value).toBe("Rex");
    });

    await userEvent.clear(questionInput);
    await userEvent.type(questionInput, "Favorite color?");
    await userEvent.clear(answerInput);
    await userEvent.type(answerInput, "Blue");
    await userEvent.click(submitButton());
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("change_recovery_question", { currentPin: "1111", recoveryQuestion: "Favorite color?", recoveryAnswer: "Blue" });
      expect(mockInvoke).not.toHaveBeenCalledWith("change_pin", expect.anything());
      expect(onChanged).toHaveBeenCalledTimes(1);
    });
  });

  it("Security Question tab leaves both fields blank when nothing is currently set", async () => {
    mockRecoveryData(null, null, (cmd, args) => {
      if (cmd === "verify_pin") return Promise.resolve(args?.pin === "1111");
    });
    render(<PinSettingsDialog hasPin={true} onClose={() => {}} onChanged={() => {}} />);
    await userEvent.click(screen.getByRole("button", { name: /security question/i }));
    await userEvent.type(passwordInputs()[0], "1111");
    const questionInput = await screen.findByPlaceholderText("Recovery question") as HTMLInputElement;
    const answerInput = screen.getByPlaceholderText("Answer") as HTMLInputElement;
    expect(questionInput.value).toBe("");
    expect(answerInput.value).toBe("");
  });

  it("Remove PIN tab calls clear_pin with the current PIN and does not require recovery fields", async () => {
    mockRecoveryData(null, null);
    const onChanged = vi.fn();
    render(<PinSettingsDialog hasPin={true} onClose={() => {}} onChanged={onChanged} />);
    await userEvent.click(screen.getByRole("button", { name: /^remove pin$/i }));
    expect(screen.queryByPlaceholderText(/recovery question/i)).not.toBeInTheDocument();
    const [current] = passwordInputs();
    await userEvent.type(current, "1111");
    await userEvent.click(submitButton());
    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("clear_pin", { currentPin: "1111" });
      expect(onChanged).toHaveBeenCalledTimes(1);
    });
  });
});

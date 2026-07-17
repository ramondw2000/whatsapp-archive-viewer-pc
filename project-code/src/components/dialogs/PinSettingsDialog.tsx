import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface PinSettingsDialogProps {
  hasPin: boolean;
  onClose: () => void;
  onChanged: () => void;
}

type Mode = "set" | "changePin" | "changeQuestion" | "remove";
type PinCheckState = "idle" | "checking" | "valid" | "invalid";

// Applied once the current PIN is verified and the recovery fields become visible/editable —
// a permanent teal border (not just on :focus), so it's visually obvious they're now usable.
const unlockedInputStyle = { borderColor: "var(--wa-teal)" };

export function PinSettingsDialog({ hasPin, onClose, onChanged }: PinSettingsDialogProps) {
  const [mode, setMode] = useState<Mode>(hasPin ? "changePin" : "set");
  const [currentPin, setCurrentPin] = useState("");
  const [newPin, setNewPin] = useState("");
  const [recoveryQuestion, setRecoveryQuestion] = useState("");
  const [recoveryAnswer, setRecoveryAnswer] = useState("");
  const [error, setError] = useState<string | null>(null);

  // Fetched on mount regardless of hasPin, and used to pre-fill both recovery fields — a
  // single question input and a single answer input, always showing (and directly editable
  // over) whatever's currently on file, no separate "old" vs "new" fields.
  const [existingRecoveryQuestion, setExistingRecoveryQuestion] = useState<string | null | undefined>(undefined);
  const [existingRecoveryAnswer, setExistingRecoveryAnswer] = useState<string | null | undefined>(undefined);
  const recoveryDataReady = existingRecoveryQuestion !== undefined && existingRecoveryAnswer !== undefined;

  // The Security Question tab's recovery fields stay fully hidden until the typed PIN is
  // actually verified against the backend (not just "non-empty") — debounced so we're not
  // firing verify_pin on every single keystroke.
  const [pinCheckState, setPinCheckState] = useState<PinCheckState>("idle");
  const pinCheckDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    invoke<string | null>("get_recovery_question").then(setExistingRecoveryQuestion).catch(() => setExistingRecoveryQuestion(null));
    invoke<string | null>("get_recovery_answer").then(setExistingRecoveryAnswer).catch(() => setExistingRecoveryAnswer(null));
  }, []);

  useEffect(() => {
    if (mode !== "changeQuestion") return;
    if (pinCheckDebounceRef.current) clearTimeout(pinCheckDebounceRef.current);
    if (!currentPin.trim()) {
      setPinCheckState("idle");
      return;
    }
    setPinCheckState("checking");
    pinCheckDebounceRef.current = setTimeout(async () => {
      try {
        const ok: boolean = await invoke("verify_pin", { pin: currentPin });
        setPinCheckState(ok ? "valid" : "invalid");
      } catch {
        setPinCheckState("invalid");
      }
    }, 400);
    return () => {
      if (pinCheckDebounceRef.current) clearTimeout(pinCheckDebounceRef.current);
    };
  }, [currentPin, mode]);

  // Pre-fill both fields once — for "set" mode as soon as the existing data is fetched, for
  // "changeQuestion" mode only once the PIN has verified (fields are hidden until then).
  useEffect(() => {
    const canPrefill = mode === "set" ? recoveryDataReady : mode === "changeQuestion" && pinCheckState === "valid" && recoveryDataReady;
    if (!canPrefill) return;
    if (!recoveryQuestion && existingRecoveryQuestion) setRecoveryQuestion(existingRecoveryQuestion);
    if (!recoveryAnswer && existingRecoveryAnswer) setRecoveryAnswer(existingRecoveryAnswer);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [mode, pinCheckState, recoveryDataReady, existingRecoveryQuestion, existingRecoveryAnswer]);

  function switchMode(next: Mode) {
    setMode(next);
    setError(null);
    setCurrentPin("");
    setNewPin("");
    setRecoveryQuestion("");
    setRecoveryAnswer("");
    setPinCheckState("idle");
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      if (mode === "set") {
        await invoke("set_pin", { pin: newPin, recoveryQuestion, recoveryAnswer });
      } else if (mode === "changePin") {
        await invoke("change_pin", { currentPin, newPin });
      } else if (mode === "changeQuestion") {
        await invoke("change_recovery_question", { currentPin, recoveryQuestion, recoveryAnswer });
      } else {
        await invoke("clear_pin", { currentPin });
      }
      onChanged();
      onClose();
    } catch (err) {
      setError(typeof err === "string" ? err : "Something went wrong");
    }
  }

  const submitDisabled =
    mode === "set" ? !newPin.trim() || !recoveryQuestion.trim() || !recoveryAnswer.trim() :
    mode === "changePin" ? !currentPin.trim() || !newPin.trim() :
    mode === "changeQuestion" ? pinCheckState !== "valid" || !recoveryQuestion.trim() || !recoveryAnswer.trim() :
    !currentPin.trim();

  const submitLabel =
    mode === "set" ? "Set PIN" :
    mode === "changePin" ? "Change PIN" :
    mode === "changeQuestion" ? "Update Security Question" :
    "Remove PIN";

  return (
    <div className="dialog-overlay">
      <div className="dialog-content pin-protection-dialog">
        <h3>App Lock Protection</h3>
        {hasPin && (
          <div className="chat-type-filter pin-protection-tabs">
            <button
              type="button"
              className={`chat-filter-btn${mode === "changePin" ? " active" : ""}`}
              onClick={() => switchMode("changePin")}
            >Change PIN</button>
            <button
              type="button"
              className={`chat-filter-btn${mode === "changeQuestion" ? " active" : ""}`}
              onClick={() => switchMode("changeQuestion")}
            >Security Question</button>
            <button
              type="button"
              className={`chat-filter-btn${mode === "remove" ? " active" : ""}`}
              onClick={() => switchMode("remove")}
            >Remove PIN</button>
          </div>
        )}
        <form onSubmit={handleSubmit}>
          {(mode === "changePin" || mode === "changeQuestion" || mode === "remove") && (
            <input
              type="password"
              inputMode="numeric"
              placeholder="Current PIN"
              value={currentPin}
              onChange={(e) => setCurrentPin(e.target.value)}
              autoFocus
            />
          )}
          {(mode === "set" || mode === "changePin") && (
            <input
              type="password"
              inputMode="numeric"
              placeholder="New PIN"
              value={newPin}
              onChange={(e) => setNewPin(e.target.value)}
              autoFocus={mode === "set"}
            />
          )}
          {mode === "set" && !recoveryDataReady && (
            <div className="character-counter">Checking for an existing recovery question…</div>
          )}
          {mode === "set" && recoveryDataReady && (
            <>
              <input
                type="text"
                placeholder="Recovery question (e.g. What's your pet's name?)"
                value={recoveryQuestion}
                onChange={(e) => setRecoveryQuestion(e.target.value)}
              />
              <input
                type="text"
                placeholder="Answer"
                value={recoveryAnswer}
                onChange={(e) => setRecoveryAnswer(e.target.value)}
              />
            </>
          )}
          {mode === "changeQuestion" && (
            <div className="character-counter" style={{ textAlign: "left", marginBottom: 16 }}>
              {pinCheckState === "idle" && "Enter your current PIN above to edit your recovery question and answer."}
              {pinCheckState === "checking" && "Checking PIN…"}
              {pinCheckState === "invalid" && "Incorrect PIN."}
              {pinCheckState === "valid" && !recoveryDataReady && "Loading your current recovery question and answer…"}
            </div>
          )}
          {mode === "changeQuestion" && pinCheckState === "valid" && recoveryDataReady && (
            <>
              <input
                type="text"
                placeholder="Recovery question"
                value={recoveryQuestion}
                onChange={(e) => setRecoveryQuestion(e.target.value)}
                style={unlockedInputStyle}
                autoFocus
              />
              <input
                type="text"
                placeholder="Answer"
                value={recoveryAnswer}
                onChange={(e) => setRecoveryAnswer(e.target.value)}
                style={unlockedInputStyle}
              />
            </>
          )}
          {error && <div className="search-no-results">{error}</div>}
          <div className="dialog-buttons">
            <button type="button" onClick={onClose}>
              Cancel
            </button>
            <button type="submit" disabled={submitDisabled}>
              {submitLabel}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface LockScreenProps {
  onUnlock: () => void;
  onPinReset: () => void;
}

export function LockScreen({ onUnlock, onPinReset }: LockScreenProps) {
  const [pin, setPin] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [recoveryQuestion, setRecoveryQuestion] = useState<string | null | undefined>(undefined);
  const [recoveryAnswer, setRecoveryAnswer] = useState("");
  const [showRecovery, setShowRecovery] = useState(false);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      const ok: boolean = await invoke("verify_pin", { pin });
      if (ok) {
        onUnlock();
      } else {
        setError("Incorrect PIN.");
        setPin("");
      }
    } catch {
      setError("Something went wrong");
    }
  }

  async function handleShowRecovery() {
    setShowRecovery(true);
    setError(null);
    try {
      const question: string | null = await invoke("get_recovery_question");
      setRecoveryQuestion(question);
    } catch {
      setRecoveryQuestion(null);
    }
  }

  async function handleSubmitRecovery(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    try {
      await invoke("reset_pin_with_recovery_answer", { answer: recoveryAnswer });
      onPinReset();
      onUnlock();
    } catch (err) {
      setError(typeof err === "string" ? err : "Incorrect answer");
      setRecoveryAnswer("");
    }
  }

  return (
    <div className="dialog-overlay">
      <div className="dialog-content">
        <h3>Enter PIN</h3>
        <form onSubmit={handleSubmit}>
          <input
            type="password"
            inputMode="numeric"
            value={pin}
            onChange={(e) => { setPin(e.target.value); setError(null); }}
            autoFocus
          />
          {!showRecovery && error && <div className="search-no-results">{error}</div>}
          <div className="dialog-buttons">
            <button type="submit" disabled={!pin}>
              Unlock
            </button>
          </div>
        </form>
        {!showRecovery ? (
          <p className="character-counter">
            <a className="lock-screen-link" href="#" onClick={(e) => { e.preventDefault(); handleShowRecovery(); }}>
              Forgot PIN?
            </a>
          </p>
        ) : recoveryQuestion === undefined ? (
          <p className="character-counter">Loading recovery question…</p>
        ) : recoveryQuestion === null ? (
          <div>
            <p className="character-counter">
              No recovery question is set for this PIN, so it can't be reset from here. Your
              chat data isn't encrypted by this lock — deleting the app's database (Delete All,
              or removing the app data folder) is the only remaining way in, but that erases
              your archive.
            </p>
            <div className="dialog-buttons">
              <button type="button" onClick={() => setShowRecovery(false)}>
                Back
              </button>
            </div>
          </div>
        ) : (
          <form onSubmit={handleSubmitRecovery}>
            <p className="character-counter" style={{ textAlign: "left" }}>{recoveryQuestion}</p>
            <input
              type="text"
              placeholder="Answer"
              value={recoveryAnswer}
              onChange={(e) => { setRecoveryAnswer(e.target.value); setError(null); }}
              autoFocus
            />
            {error && <div className="search-no-results">{error}</div>}
            <div className="dialog-buttons">
              <button type="button" onClick={() => { setShowRecovery(false); setRecoveryAnswer(""); setError(null); }}>
                Cancel
              </button>
              <button type="submit" disabled={!recoveryAnswer.trim()}>
                Reset lock
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}

import { useState } from 'react';

interface UsernameDialogProps {
  onSubmit: (name: string) => void;
  onCancel: () => void;
  hasExistingName: boolean;
}

export function UsernameDialog({ onSubmit, onCancel, hasExistingName }: UsernameDialogProps) {
  const [name, setName] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (name.trim()) {
      onSubmit(name.trim());
    }
  };

  return (
    <div className="dialog-overlay">
      <div className="dialog-content">
        <h3>Welcome to WhatsApp Archive Viewer</h3>
        <p>Please enter your WhatsApp name to personalize your experience:</p>
        <form onSubmit={handleSubmit}>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Your WhatsApp name"
            maxLength={25}
            autoFocus
          />
          <div className="character-counter">
            {25 - name.length} characters remaining
          </div>
          <div className="dialog-buttons">
            <button type="button" onClick={onCancel} disabled={!hasExistingName}>
              Cancel
            </button>
            <button type="submit" disabled={!name.trim()}>
              Continue
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}

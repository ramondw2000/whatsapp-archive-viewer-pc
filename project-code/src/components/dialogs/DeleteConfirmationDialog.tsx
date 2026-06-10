interface DeleteConfirmationDialogProps {
  chatId: string;
  chatName: string;
  onSave: () => void;
  onDelete: () => void;
  onCancel: () => void;
  hasChanges: boolean;
}

export function DeleteConfirmationDialog({ chatId: _chatId, chatName, onSave, onDelete, onCancel, hasChanges }: DeleteConfirmationDialogProps) {
  return (
    <div className="dialog-overlay">
      <div className="dialog-content">
        <button className="dialog-close-btn" onClick={onCancel}>
          ×
        </button>
        <h3>⚠️ Delete Chat</h3>
        <p><strong>Are you sure you want to delete "{chatName}"?</strong></p>
        <p>This action cannot be undone. Choose your option:</p>
        <div className="dialog-buttons">
          <button
            className={`dialog-btn-primary ${!hasChanges ? 'disabled' : ''}`}
            onClick={onSave}
            disabled={!hasChanges}
            title={!hasChanges ? "No changes found" : ""}
          >
            Keep Changes & Delete
          </button>
          <button className="dialog-btn-danger" onClick={onDelete}>
            Delete Everything
          </button>
        </div>
      </div>
    </div>
  );
}

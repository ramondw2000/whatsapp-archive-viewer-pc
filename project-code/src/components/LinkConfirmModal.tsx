interface LinkConfirmModalProps {
  url: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function LinkConfirmModal({ url, onConfirm, onCancel }: LinkConfirmModalProps) {
  return (
    <div className="dialog-overlay" onClick={onCancel}>
      <div className="dialog-content link-confirm-modal" onClick={e => e.stopPropagation()}>
        <h3>Open link?</h3>
        <p className="link-confirm-url">{url}</p>
        <div className="dialog-actions">
          <button className="btn-secondary" onClick={onCancel}>Cancel</button>
          <button className="btn-primary" onClick={onConfirm}>Open</button>
        </div>
      </div>
    </div>
  );
}

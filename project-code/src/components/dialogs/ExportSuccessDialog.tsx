interface ExportSuccessDialogProps {
  message: string;
  onClose: () => void;
}

export function ExportSuccessDialog({ message, onClose }: ExportSuccessDialogProps) {
  return (
    <div className="dialog-overlay">
      <div className="dialog-content">
        <button className="dialog-close-btn" onClick={onClose}>
          ×
        </button>
        <div className="success-icon" style={{
          width: "48px",
          height: "48px",
          borderRadius: "50%",
          background: "var(--wa-teal, #00a884)",
          color: "white",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          fontSize: "24px",
          margin: "0 auto 16px"
        }}>
          ✓
        </div>
        <h3 style={{ textAlign: "center", marginBottom: "12px" }}>Export Complete</h3>
        <p style={{ textAlign: "center", color: "var(--text-secondary, #666)", marginBottom: "24px" }}>
          {message}
        </p>
        <div className="dialog-buttons" style={{ justifyContent: "center" }}>
          <button className="dialog-btn-primary" onClick={onClose}>
            OK
          </button>
        </div>
      </div>
    </div>
  );
}

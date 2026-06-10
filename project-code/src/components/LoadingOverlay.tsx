interface LoadingOverlayProps {
  message: string;
  detail?: string;
}

export function LoadingOverlay({ message, detail }: LoadingOverlayProps) {
  return (
    <div className="loading-overlay">
      <div className="loading-spinner"></div>
      <p className="loading-text">{message}</p>
      {detail && <p className="loading-detail">{detail}</p>}
      <div className="loading-progress">
        <div className="loading-bar"></div>
      </div>
    </div>
  );
}

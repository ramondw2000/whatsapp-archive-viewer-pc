interface JumpButtonProps {
  direction: 'up' | 'down';
  onClick: () => void;
  visible: boolean;
}

export function JumpButton({ direction, onClick, visible }: JumpButtonProps) {
  if (!visible) return null;

  return (
    <button
      className={`jump-button jump-button--${direction}`}
      onClick={onClick}
      title={`Jump to ${direction === 'up' ? 'top' : 'bottom'}`}
    >
      {direction === 'up' ? '↑' : '↓'}
    </button>
  );
}

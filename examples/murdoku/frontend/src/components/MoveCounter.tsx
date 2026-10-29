import { Dices } from 'lucide-react';

interface MoveCounterProps {
  moveCount: number;
}

export function MoveCounter({ moveCount }: MoveCounterProps) {
  return (
    <div
      aria-label={`${moveCount} moves made`}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: '0.5rem',
        padding: '0.375rem 0.75rem',
        background: 'var(--noir-surface)',
        border: '1px solid var(--noir-border)',
        borderRadius: 4,
      }}
    >
      <Dices size={13} style={{ color: 'var(--accent-gold)' }} aria-hidden="true" />
      <span
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '0.8125rem',
          color: 'var(--noir-text)',
        }}
      >
        {moveCount} {moveCount === 1 ? 'move' : 'moves'}
      </span>
    </div>
  );
}

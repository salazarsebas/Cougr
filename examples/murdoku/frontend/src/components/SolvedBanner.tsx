import { Trophy, X } from 'lucide-react';

interface SolvedBannerProps {
  puzzleTitle: string;
  moveCount: number;
  onDismiss: () => void;
}

export function SolvedBanner({ puzzleTitle, moveCount, onDismiss }: SolvedBannerProps) {
  return (
    <div
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="solved-banner-title"
      aria-describedby="solved-banner-desc"
      className="animate-fade-overlay"
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0,0,0,0.75)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        padding: '1rem',
        zIndex: 100,
        backdropFilter: 'blur(4px)',
      }}
    >
      <div
        className="animate-fade-in"
        style={{
          background: 'var(--noir-surface)',
          border: '1px solid var(--accent-green)',
          borderRadius: 8,
          padding: '2.5rem 2rem',
          maxWidth: 440,
          width: '100%',
          textAlign: 'center',
          position: 'relative',
          boxShadow: '0 0 40px rgba(39,174,96,0.2)',
        }}
      >
        {/* Dismiss */}
        <button
          id="btn-dismiss-banner"
          onClick={onDismiss}
          aria-label="Dismiss solved banner"
          style={{
            position: 'absolute',
            top: '0.75rem',
            right: '0.75rem',
            background: 'none',
            border: 'none',
            color: 'var(--noir-muted)',
            cursor: 'pointer',
            padding: '0.25rem',
            borderRadius: 4,
          }}
        >
          <X size={16} />
        </button>

        {/* Trophy icon */}
        <div
          aria-hidden="true"
          style={{
            width: 56, height: 56,
            borderRadius: '50%',
            background: 'rgba(39,174,96,0.15)',
            border: '1px solid var(--accent-green)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            margin: '0 auto 1.25rem',
          }}
        >
          <Trophy size={24} style={{ color: 'var(--accent-green)' }} />
        </div>

        {/* Title */}
        <h2
          id="solved-banner-title"
          style={{
            fontFamily: 'var(--font-serif)',
            fontSize: '1.75rem',
            fontWeight: 700,
            color: 'var(--accent-green)',
            marginBottom: '0.5rem',
          }}
        >
          Case Closed
        </h2>

        {/* Description */}
        <p
          id="solved-banner-desc"
          style={{
            fontFamily: 'var(--font-serif)',
            fontStyle: 'italic',
            color: 'var(--noir-muted)',
            fontSize: '0.9375rem',
            marginBottom: '1.5rem',
            lineHeight: 1.6,
          }}
        >
          You cracked <em style={{ color: 'var(--noir-text)' }}>"{puzzleTitle}"</em>{' '}
          in {moveCount} {moveCount === 1 ? 'move' : 'moves'}. The truth always finds the light.
        </p>

        {/* Stats */}
        <div
          style={{
            display: 'flex',
            justifyContent: 'center',
            gap: '2rem',
            marginBottom: '1.75rem',
          }}
        >
          <div>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '1.5rem', color: 'var(--accent-gold)', fontWeight: 600 }}>
              {moveCount}
            </p>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.6875rem', color: 'var(--noir-muted)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>
              Moves
            </p>
          </div>
          <div style={{ width: 1, background: 'var(--noir-border)' }} />
          <div>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '1.5rem', color: 'var(--accent-green)', fontWeight: 600 }}>
              ✓
            </p>
            <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.6875rem', color: 'var(--noir-muted)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>
              Solved
            </p>
          </div>
        </div>

        <button
          id="btn-play-another"
          className="btn-gold"
          onClick={onDismiss}
          style={{ width: '100%', justifyContent: 'center' }}
        >
          Play Another Case
        </button>
      </div>
    </div>
  );
}

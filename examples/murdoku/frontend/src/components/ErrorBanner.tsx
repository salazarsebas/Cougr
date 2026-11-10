import { AlertTriangle, X } from 'lucide-react';

interface ErrorBannerProps {
  message: string;
  onDismiss: () => void;
}

export function ErrorBanner({ message, onDismiss }: ErrorBannerProps) {
  if (!message) return null;

  return (
    <div
      role="alert"
      aria-live="assertive"
      style={{
        display: 'flex',
        alignItems: 'flex-start',
        gap: '0.625rem',
        padding: '0.75rem 1rem',
        background: 'rgba(192,57,43,0.12)',
        border: '1px solid var(--accent-red)',
        borderRadius: 6,
        maxWidth: 440,
        margin: '0.75rem auto',
      }}
    >
      <AlertTriangle size={16} style={{ color: 'var(--accent-red)', flexShrink: 0, marginTop: 1 }} />
      <p
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '0.8125rem',
          color: 'var(--accent-red)',
          flex: 1,
          lineHeight: 1.4,
        }}
      >
        {message}
      </p>
      <button
        onClick={onDismiss}
        aria-label="Dismiss error"
        style={{
          background: 'none',
          border: 'none',
          color: 'var(--accent-red)',
          cursor: 'pointer',
          padding: '0.125rem',
          flexShrink: 0,
        }}
      >
        <X size={14} />
      </button>
    </div>
  );
}

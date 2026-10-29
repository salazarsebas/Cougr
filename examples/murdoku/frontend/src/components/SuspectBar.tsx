import type { Suspect } from '../types';

interface SuspectBarProps {
  suspects: Suspect[];
  selectedSuspectId: number | null;
  placedSuspectIds: Set<number>;
  onSelect: (id: number) => void;
  /** 'horizontal' on mobile, 'vertical' on desktop sidebar */
  orientation?: 'horizontal' | 'vertical';
}

export function SuspectBar({
  suspects,
  selectedSuspectId,
  placedSuspectIds,
  onSelect,
  orientation = 'vertical',
}: SuspectBarProps) {
  const isHorizontal = orientation === 'horizontal';

  return (
    <aside
      aria-label="Suspect selection"
      style={{
        display: 'flex',
        flexDirection: isHorizontal ? 'row' : 'column',
        gap: '0.5rem',
        flexWrap: isHorizontal ? 'wrap' : 'nowrap',
        padding: isHorizontal ? '0.75rem 0' : '0.5rem',
      }}
    >
      <p
        id="suspect-bar-label"
        style={{
          fontFamily: 'var(--font-mono)',
          fontSize: '0.6875rem',
          color: 'var(--noir-muted)',
          textTransform: 'uppercase',
          letterSpacing: '0.08em',
          alignSelf: isHorizontal ? 'center' : 'auto',
          flexShrink: 0,
        }}
      >
        Suspects
      </p>

      {suspects.map((suspect) => {
        const isPlaced = placedSuspectIds.has(suspect.id);
        const isSelected = selectedSuspectId === suspect.id;

        return (
          <button
            key={suspect.id}
            id={`suspect-btn-${suspect.id}`}
            role="option"
            aria-selected={isSelected}
            aria-label={`${suspect.name}${isPlaced ? ', already placed' : ''}${isSelected ? ', selected' : ''}`}
            disabled={isPlaced}
            onClick={() => !isPlaced && onSelect(suspect.id)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '0.625rem',
              padding: '0.5rem 0.75rem',
              background: isSelected
                ? 'rgba(201,168,76,0.1)'
                : 'var(--noir-surface)',
              border: `1px solid ${isSelected ? 'var(--accent-gold)' : 'var(--noir-border)'}`,
              borderLeft: `3px solid ${suspect.color}`,
              borderRadius: 4,
              cursor: isPlaced ? 'not-allowed' : 'pointer',
              opacity: isPlaced ? 0.4 : 1,
              transition: 'border-color 150ms, opacity 150ms, background 150ms',
              textAlign: 'left',
              width: isHorizontal ? 'auto' : '100%',
              flexShrink: 0,
            }}
          >
            {/* Color swatch */}
            <span
              aria-hidden="true"
              style={{
                width: 10, height: 10,
                borderRadius: '50%',
                background: suspect.color,
                flexShrink: 0,
                boxShadow: isSelected ? `0 0 8px ${suspect.color}80` : 'none',
              }}
            />

            <span
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '0.8125rem',
                color: isPlaced ? 'var(--noir-muted)' : 'var(--noir-text)',
                whiteSpace: 'nowrap',
              }}
            >
              {suspect.name}
            </span>

            {/* Placed indicator */}
            {isPlaced && (
              <span
                aria-hidden="true"
                style={{
                  marginLeft: 'auto',
                  fontSize: '0.6875rem',
                  color: 'var(--noir-muted)',
                }}
              >
                placed
              </span>
            )}

            {/* Selected indicator */}
            {isSelected && !isPlaced && (
              <span
                aria-hidden="true"
                style={{
                  marginLeft: 'auto',
                  width: 6, height: 6,
                  borderRadius: '50%',
                  background: 'var(--accent-gold)',
                  boxShadow: '0 0 6px var(--accent-gold)',
                }}
              />
            )}
          </button>
        );
      })}
    </aside>
  );
}

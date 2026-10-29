import { useState } from 'react';
import {
  ChevronDown, ChevronUp,
  Rows3, Columns3, MapPin, Ban, AlignHorizontalDistributeCenter,
} from 'lucide-react';
import type { Clue, ClueType, Suspect } from '../types';

interface CluePanelProps {
  clues: Clue[];
  suspects: Suspect[];
}

const CLUE_ICONS: Record<ClueType, React.ReactNode> = {
  not_same_row:  <Rows3    size={13} aria-hidden="true" />,
  not_same_col:  <Columns3 size={13} aria-hidden="true" />,
  adjacent:      <AlignHorizontalDistributeCenter size={13} aria-hidden="true" />,
  not_adjacent:  <Ban      size={13} aria-hidden="true" />,
  same_row:      <Rows3    size={13} aria-hidden="true" />,
  same_col:      <Columns3 size={13} aria-hidden="true" />,
};

const CLUE_COLORS: Record<ClueType, string> = {
  not_same_row:  'var(--accent-red)',
  not_same_col:  'var(--accent-red)',
  adjacent:      'var(--accent-gold)',
  not_adjacent:  'var(--accent-red)',
  same_row:      'var(--accent-blue)',
  same_col:      'var(--accent-blue)',
};

function getSuspect(suspects: Suspect[], id: number) {
  return suspects.find((s) => s.id === id);
}

export function CluePanel({ clues, suspects }: CluePanelProps) {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <section
      aria-label="Puzzle clues"
      style={{
        background: 'var(--noir-surface)',
        border: '1px solid var(--noir-border)',
        borderRadius: 6,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        maxHeight: collapsed ? 52 : 600,
        transition: 'max-height 200ms ease-in-out',
      }}
    >
      {/* Header */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '0.75rem 1rem',
          borderBottom: collapsed ? 'none' : '1px solid var(--noir-border)',
          flexShrink: 0,
        }}
      >
        <h2
          style={{
            fontFamily: 'var(--font-serif)',
            fontSize: '1rem',
            fontWeight: 600,
            color: 'var(--noir-text)',
          }}
        >
          <MapPin
            size={14}
            aria-hidden="true"
            style={{ marginRight: '0.4rem', color: 'var(--accent-gold)', verticalAlign: 'middle' }}
          />
          Evidence
          <span
            style={{
              marginLeft: '0.4rem',
              fontFamily: 'var(--font-mono)',
              fontSize: '0.7rem',
              color: 'var(--noir-muted)',
            }}
          >
            ({clues.length})
          </span>
        </h2>

        <button
          id="btn-toggle-clues"
          onClick={() => setCollapsed((c) => !c)}
          aria-expanded={!collapsed}
          aria-controls="clue-list"
          className="btn-outline"
          style={{ padding: '0.25rem 0.5rem', gap: '0.25rem' }}
          aria-label={collapsed ? 'Expand clues panel' : 'Collapse clues panel'}
        >
          {collapsed ? <ChevronDown size={14} /> : <ChevronUp size={14} />}
          <span style={{ fontSize: '0.75rem' }}>{collapsed ? 'Show' : 'Hide'}</span>
        </button>
      </div>

      {/* Clue list */}
      {!collapsed && (
        <ol
          id="clue-list"
          aria-label="List of puzzle clues"
          style={{
            listStyle: 'none',
            overflowY: 'auto',
            flex: 1,
            padding: '0.5rem 0',
          }}
        >
          {clues.map((clue, i) => {
            const suspectA = getSuspect(suspects, clue.suspectAId);
            const suspectB = getSuspect(suspects, clue.suspectBId);
            const iconColor = CLUE_COLORS[clue.type];

            return (
              <li
                key={clue.id}
                id={`clue-${clue.id}`}
                style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: '0.625rem',
                  padding: '0.625rem 1rem',
                  borderBottom: '1px solid var(--noir-border)',
                }}
              >
                {/* Clue type icon */}
                <span
                  aria-hidden="true"
                  style={{
                    color: iconColor,
                    flexShrink: 0,
                    marginTop: 2,
                  }}
                >
                  {CLUE_ICONS[clue.type]}
                </span>

                <div style={{ flex: 1, minWidth: 0 }}>
                  {/* Suspect tags */}
                  <div style={{ display: 'flex', gap: '0.375rem', marginBottom: '0.3rem', flexWrap: 'wrap' }}>
                    {suspectA && (
                      <span
                        style={{
                          fontFamily: 'var(--font-mono)',
                          fontSize: '0.65rem',
                          padding: '0.1rem 0.4rem',
                          borderRadius: 3,
                          background: `${suspectA.color}30`,
                          color: suspectA.color,
                          border: `1px solid ${suspectA.color}50`,
                        }}
                      >
                        {suspectA.initials}
                      </span>
                    )}
                    {suspectB && (
                      <span
                        style={{
                          fontFamily: 'var(--font-mono)',
                          fontSize: '0.65rem',
                          padding: '0.1rem 0.4rem',
                          borderRadius: 3,
                          background: `${suspectB.color}30`,
                          color: suspectB.color,
                          border: `1px solid ${suspectB.color}50`,
                        }}
                      >
                        {suspectB.initials}
                      </span>
                    )}
                  </div>

                  {/* Clue text */}
                  <p
                    style={{
                      fontFamily: 'var(--font-serif)',
                      fontStyle: 'italic',
                      fontSize: '0.875rem',
                      color: 'var(--noir-muted)',
                      lineHeight: 1.5,
                    }}
                  >
                    <span
                      aria-hidden="true"
                      style={{
                        fontFamily: 'var(--font-mono)',
                        fontSize: '0.65rem',
                        color: 'var(--noir-border)',
                        marginRight: '0.4rem',
                      }}
                    >
                      {String(i + 1).padStart(2, '0')}.
                    </span>
                    {clue.description}
                  </p>
                </div>
              </li>
            );
          })}
        </ol>
      )}
    </section>
  );
}

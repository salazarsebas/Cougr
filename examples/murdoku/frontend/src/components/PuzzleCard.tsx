import { Grid2x2, User, Trophy } from 'lucide-react';
import type { PuzzleSummary, Difficulty } from '../types';

interface PuzzleCardProps {
  puzzle: PuzzleSummary;
  totalSolvers?: number;
  onClick: (id: string) => void;
}

const DIFFICULTY_STYLES: Record<Difficulty, { color: string; label: string }> = {
  Easy:   { color: 'var(--accent-green)', label: 'Easy' },
  Medium: { color: 'var(--accent-gold)',  label: 'Medium' },
  Expert: { color: 'var(--accent-red)',   label: 'Expert' },
};

function shortenAddress(addr: string) {
  if (addr.length <= 12) return addr;
  return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
}

export function PuzzleCard({ puzzle, totalSolvers, onClick }: PuzzleCardProps) {
  const diff = DIFFICULTY_STYLES[puzzle.difficulty];

  return (
    <article
      className="card"
      id={`puzzle-card-${puzzle.id}`}
      role="button"
      tabIndex={0}
      aria-label={`Play puzzle: ${puzzle.title}, ${puzzle.difficulty} difficulty, ${puzzle.gridSize}×${puzzle.gridSize} grid`}
      onClick={() => onClick(puzzle.id)}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') onClick(puzzle.id); }}
      style={{
        padding: '1.25rem',
        cursor: 'pointer',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.875rem',
        userSelect: 'none',
      }}
    >
      {/* Top row: grid badge + difficulty */}
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '0.5rem' }}>
        <span
          className="badge"
          aria-label={`${puzzle.gridSize}×${puzzle.gridSize} grid`}
          style={{ background: 'rgba(201,168,76,0.12)', color: 'var(--accent-gold)', gap: '0.3rem' }}
        >
          <Grid2x2 size={11} aria-hidden="true" />
          {puzzle.gridSize}×{puzzle.gridSize}
        </span>

        <span
          className="badge"
          aria-label={`Difficulty: ${puzzle.difficulty}`}
          style={{ color: diff.color, background: 'transparent', fontWeight: 600 }}
        >
          <span
            aria-hidden="true"
            style={{
              width: 6, height: 6, borderRadius: '50%',
              background: diff.color, display: 'inline-block',
            }}
          />
          {diff.label}
        </span>
      </div>

      {/* Title */}
      <h2
        style={{
          fontFamily: 'var(--font-serif)',
          fontSize: '1.125rem',
          fontWeight: 600,
          color: 'var(--noir-text)',
          lineHeight: 1.3,
        }}
      >
        {puzzle.title}
      </h2>

      {/* Footer: clue count + solvers + creator */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginTop: 'auto',
          fontFamily: 'var(--font-mono)',
          fontSize: '0.75rem',
          color: 'var(--noir-muted)',
          gap: '0.5rem',
        }}
      >
        <span>{puzzle.clueCount} clues</span>
        {totalSolvers !== undefined && (
          <span style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
            <Trophy size={10} aria-hidden="true" style={{ color: 'var(--accent-gold)' }} />
            {totalSolvers}
          </span>
        )}
        <span style={{ display: 'flex', alignItems: 'center', gap: '0.3rem' }}>
          <User size={11} aria-hidden="true" />
          {shortenAddress(puzzle.creatorAddress)}
        </span>
      </div>
    </article>
  );
}

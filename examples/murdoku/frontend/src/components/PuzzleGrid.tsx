import type { Suspect, Cell } from '../types';

interface CellLoadingState {
  [key: number]: boolean;
}

interface PuzzleGridProps {
  cells: Cell[];
  gridSize: 4 | 5;
  suspects: Suspect[];
  isSolved: boolean;
  onCellClick: (index: number) => void;
  loadingCells?: CellLoadingState;
}

function getSuspectById(suspects: Suspect[], id: number | null): Suspect | undefined {
  if (id === null) return undefined;
  return suspects.find((s) => s.id === id);
}

function hexToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r},${g},${b},${alpha})`;
}

function checkConflicts(board: Cell[], size: number): Set<number> {
  const conflicts = new Set<number>();
  for (let i = 0; i < board.length; i++) {
    if (board[i].suspectId === null) continue;
    const val = board[i].suspectId;
    const row = Math.floor(i / size);
    const col = i % size;
    let hasConflict = false;
    for (let c = 0; c < size && !hasConflict; c++) {
      const j = row * size + c;
      if (j !== i && board[j].suspectId === val) hasConflict = true;
    }
    for (let r = 0; r < size && !hasConflict; r++) {
      const j = r * size + col;
      if (j !== i && board[j].suspectId === val) hasConflict = true;
    }
    if (hasConflict) conflicts.add(i);
  }
  return conflicts;
}

export function PuzzleGrid({ cells, gridSize, suspects, isSolved, onCellClick, loadingCells = {} }: PuzzleGridProps) {
  const conflicts = isSolved ? new Set<number>() : checkConflicts(cells, gridSize);

  return (
    <div
      role="grid"
      aria-label={`${gridSize}×${gridSize} puzzle grid`}
      aria-describedby="grid-instructions"
      style={{
        display: 'grid',
        gridTemplateColumns: `repeat(${gridSize}, 1fr)`,
        gap: 2,
        background: 'var(--noir-border)',
        border: '2px solid var(--noir-border)',
        borderRadius: 6,
        overflow: 'hidden',
        width: '100%',
        maxWidth: gridSize === 4 ? 360 : 440,
        margin: '0 auto',
      }}
    >
      <p id="grid-instructions" className="sr-only">
        Select a suspect from the suspect bar, then click a cell to place them. Click an occupied cell to remove the suspect. Conflicting cells are highlighted in red.
      </p>

      {cells.map((cell, idx) => {
        const row = Math.floor(idx / gridSize);
        const col = idx % gridSize;
        const suspect = getSuspectById(suspects, cell.suspectId);
        const isConflict = conflicts.has(idx);
        const isEmpty = cell.suspectId === null;
        const isLoading = loadingCells[idx] === true;

        let bgColor = 'var(--noir-surface)';
        if (isConflict) bgColor = 'rgba(192,57,43,0.22)';
        else if (isSolved) bgColor = 'rgba(39,174,96,0.18)';
        else if (suspect) bgColor = hexToRgba(suspect.color, 0.18);

        let borderColor = 'transparent';
        if (isConflict) borderColor = 'var(--accent-red)';

        return (
          <div
            key={idx}
            role="gridcell"
            id={`cell-${row}-${col}`}
            aria-label={`Row ${row + 1}, Column ${col + 1}: ${suspect ? suspect.name : 'empty'}${isConflict ? ', conflict' : ''}`}
            aria-selected={false}
            tabIndex={0}
            onClick={() => !isLoading && onCellClick(idx)}
            onKeyDown={(e) => { if (!isLoading && (e.key === 'Enter' || e.key === ' ')) onCellClick(idx); }}
            style={{
              background: bgColor,
              border: `2px solid ${borderColor}`,
              aspectRatio: '1',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexDirection: 'column',
              cursor: isLoading ? 'wait' : (isEmpty ? 'pointer' : 'pointer'),
              transition: isConflict
                ? 'background-color 0ms'
                : isSolved
                ? 'background-color 400ms ease-in-out'
                : 'background-color 150ms ease-in-out',
              position: 'relative',
              padding: '0.25rem',
              opacity: isLoading ? 0.7 : 1,
            }}
          >
            {isLoading && (
              <div
                style={{
                  position: 'absolute',
                  inset: 0,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  background: 'rgba(0,0,0,0.3)',
                  zIndex: 1,
                }}
              >
                <div
                  style={{
                    width: 20,
                    height: 20,
                    border: '2px solid var(--noir-border)',
                    borderTopColor: 'var(--accent-gold)',
                    borderRadius: '50%',
                    animation: 'spin 0.8s linear infinite',
                  }}
                />
              </div>
            )}

            {suspect && (
              <>
                <span
                  aria-hidden="true"
                  style={{
                    width: 8, height: 8,
                    borderRadius: '50%',
                    background: suspect.color,
                    marginBottom: 4,
                    boxShadow: `0 0 6px ${suspect.color}80`,
                    flexShrink: 0,
                  }}
                />
                <span
                  style={{
                    fontFamily: 'var(--font-mono)',
                    fontSize: gridSize === 5 ? '0.6rem' : '0.7rem',
                    fontWeight: 600,
                    color: isConflict ? 'var(--accent-red)' : 'var(--noir-text)',
                    textAlign: 'center',
                    lineHeight: 1.2,
                    wordBreak: 'break-word',
                    maxWidth: '100%',
                  }}
                >
                  {suspect.initials}
                </span>
              </>
            )}

            {isSolved && suspect && (
              <span
                aria-hidden="true"
                style={{
                  position: 'absolute',
                  top: 2, right: 4,
                  fontSize: '0.6rem',
                  color: 'var(--accent-green)',
                  zIndex: 2,
                }}
              >
                ✓
              </span>
            )}
          </div>
        );
      })}

      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}

import { useState, useCallback, useEffect, useMemo } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, LayoutPanelLeft, Loader2, Play } from 'lucide-react';
import { PuzzleGrid } from '../components/PuzzleGrid';
import { SuspectBar } from '../components/SuspectBar';
import { CluePanel } from '../components/CluePanel';
import { SolvedBanner } from '../components/SolvedBanner';
import { MoveCounter } from '../components/MoveCounter';
import { ErrorBanner } from '../components/ErrorBanner';
import {
  usePuzzle,
  usePlayerState,
  useStartGame,
  usePlaceSuspect,
  useRemoveSuspect,
} from '../hooks/useMurdoku';
import type { Cell } from '../types';

function buildCellsFromContract(
  contractCells: { suspectId: number | null }[],
  _gridSize: number,
): Cell[] {
  return contractCells.map((c) => ({
    suspectId: c.suspectId,
    status: c.suspectId === null ? 'empty' : 'filled',
  }));
}

function buildInitialCells(gridSize: number): Cell[] {
  return Array.from({ length: gridSize * gridSize }, () => ({
    suspectId: null,
    status: 'empty',
  }));
}

function getPlacedSuspectIds(cells: Cell[]): Set<number> {
  const ids = new Set<number>();
  cells.forEach((c) => {
    if (c.suspectId !== null) ids.add(c.suspectId);
  });
  return ids;
}

function getMoveResultMessage(result: string): string {
  switch (result) {
    case 'RowConflict':
      return 'This suspect already appears in that row.';
    case 'ColConflict':
      return 'This suspect already appears in that column.';
    case 'CellOccupied':
      return 'This cell is already occupied.';
    case 'GameAlreadySolved':
      return 'This puzzle is already solved.';
    case 'InvalidCoordinates':
      return 'Invalid cell coordinates.';
    default:
      return 'The move was rejected by the chain.';
  }
}

interface PlayPageProps {
  walletConnected: boolean;
}

export function PlayPage({ walletConnected }: PlayPageProps) {
  const { puzzleId } = useParams<{ puzzleId: string }>();
  const navigate = useNavigate();
  const [cluePanelOpen, setCluePanelOpen] = useState(true);
  const [selectedSuspectId, setSelectedSuspectId] = useState<number | null>(null);
  const [showSolvedBanner, setShowSolvedBanner] = useState(false);
  const [cells, setCells] = useState<Cell[]>([]);
  const [moveCount, setMoveCount] = useState(0);
  const [isSolved, setIsSolved] = useState(false);
  const [loadingCells, setLoadingCells] = useState<Record<number, boolean>>({});
  const [moveError, setMoveError] = useState<string | null>(null);
  const [hasSession, setHasSession] = useState(false);

  const { puzzle, loading: puzzleLoading, error: puzzleError } = usePuzzle(puzzleId || '');
  const {
    playerState,
    hasSession: contractHasSession,
    refresh: refreshPlayerState,
  } = usePlayerState(puzzleId || '', puzzle?.gridSize ?? 4);

  const { startGame, loading: startLoading, error: startError } = useStartGame(
    puzzleId || '',
    puzzle?.gridSize ?? 4,
  );

  const { placeSuspect, loading: placeLoading } = usePlaceSuspect(
    puzzleId || '',
    puzzle?.gridSize ?? 4,
  );

  const { removeSuspect, loading: removeLoading } = useRemoveSuspect(
    puzzleId || '',
    puzzle?.gridSize ?? 4,
  );

  useEffect(() => {
    if (!walletConnected) {
      navigate('/');
    }
  }, [walletConnected, navigate]);

  useEffect(() => {
    if (contractHasSession !== hasSession) {
      setHasSession(contractHasSession);
    }
  }, [contractHasSession, hasSession]);

  useEffect(() => {
    if (playerState && puzzle) {
      setCells(buildCellsFromContract(playerState.cells, puzzle.gridSize));
      setMoveCount(playerState.moveCount);
      setIsSolved(playerState.solved);
      setHasSession(true);
    }
  }, [playerState, puzzle]);

  useEffect(() => {
    if (isSolved && puzzle) {
      setShowSolvedBanner(true);
    }
  }, [isSolved, puzzle]);

  const handleStartGame = useCallback(async () => {
    if (!puzzle) return;
    try {
      await startGame();
      setHasSession(true);
      setCells(buildInitialCells(puzzle.gridSize));
      setMoveCount(0);
      setIsSolved(false);
    } catch (e) {
      // Error handled by hook
    }
  }, [puzzle, startGame]);

  const handleCellClick = useCallback(
    async (idx: number) => {
      if (!puzzle || !hasSession || isSolved) return;
      if (placeLoading || removeLoading) return;

      const cell = cells[idx];
      const gridSize = puzzle.gridSize;
      const row = Math.floor(idx / gridSize);
      const col = idx % gridSize;

      setMoveError(null);

      if (cell.suspectId !== null) {
        setLoadingCells((prev) => ({ ...prev, [idx]: true }));

        try {
          const result = await removeSuspect(row, col);
          if (result !== 'Ok') {
            setMoveError(getMoveResultMessage(result));
            setLoadingCells((prev) => ({ ...prev, [idx]: false }));
            return;
          }

          setCells((prev) =>
            prev.map((c, i) =>
              i === idx ? { suspectId: null, status: 'empty' } : c,
            ),
          );
          setMoveCount((prev) => prev + 1);
          setIsSolved(false);
          refreshPlayerState();
        } catch (e) {
          setMoveError(e instanceof Error ? e.message : 'Failed to remove suspect');
        } finally {
          setLoadingCells((prev) => ({ ...prev, [idx]: false }));
        }
      } else {
        if (selectedSuspectId === null) return;

        setLoadingCells((prev) => ({ ...prev, [idx]: true }));

        const prevCellState = { ...cell };

        setCells((prev) =>
          prev.map((c, i) =>
            i === idx ? { suspectId: selectedSuspectId, status: 'filled' } : c,
          ),
        );

        try {
          const result = await placeSuspect(row, col, selectedSuspectId);

          if (result !== 'Ok') {
            setCells((prev) =>
              prev.map((c, i) => (i === idx ? prevCellState : c)),
            );
            setMoveError(getMoveResultMessage(result));
            setLoadingCells((prev) => ({ ...prev, [idx]: false }));
            return;
          }

          setMoveCount((prev) => prev + 1);
          setSelectedSuspectId(null);
          refreshPlayerState();
        } catch (e) {
          setCells((prev) =>
            prev.map((c, i) => (i === idx ? prevCellState : c)),
          );
          setMoveError(e instanceof Error ? e.message : 'Failed to place suspect');
        } finally {
          setLoadingCells((prev) => ({ ...prev, [idx]: false }));
        }
      }
    },
    [
      puzzle,
      hasSession,
      isSolved,
      cells,
      selectedSuspectId,
      placeSuspect,
      removeSuspect,
      placeLoading,
      removeLoading,
      refreshPlayerState,
    ],
  );

  const handleSelectSuspect = useCallback((id: number) => {
    setSelectedSuspectId((prev) => (prev === id ? null : id));
  }, []);

  const placedSuspectIds = useMemo(() => getPlacedSuspectIds(cells), [cells]);
  const selectedSuspect = puzzle?.suspects.find((s) => s.id === selectedSuspectId);

  if (!walletConnected) {
    return null;
  }

  if (puzzleLoading) {
    return (
      <main
        style={{
          flex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '2rem',
        }}
      >
        <div style={{ textAlign: 'center' }}>
          <Loader2 size={32} style={{ color: 'var(--accent-gold)' }} className="animate-spin" />
          <p
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.875rem',
              color: 'var(--noir-muted)',
              marginTop: '1rem',
            }}
          >
            Loading case file...
          </p>
        </div>
      </main>
    );
  }

  if (puzzleError || !puzzle) {
    return (
      <main
        style={{
          flex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '2rem',
        }}
      >
        <div style={{ textAlign: 'center' }}>
          <h1
            style={{
              fontFamily: 'var(--font-serif)',
              color: 'var(--accent-red)',
              marginBottom: '1rem',
            }}
          >
            Case Not Found
          </h1>
          <p style={{ color: 'var(--noir-muted)', marginBottom: '1.5rem' }}>
            No puzzle matches this case ID.
          </p>
          <button className="btn-outline" onClick={() => navigate('/')}>
            <ArrowLeft size={14} /> Back to Case Files
          </button>
        </div>
      </main>
    );
  }

  if (!hasSession) {
    return (
      <main
        style={{
          flex: 1,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '2rem',
        }}
      >
        <div
          style={{
            textAlign: 'center',
            maxWidth: 420,
            background: 'var(--noir-surface)',
            border: '1px solid var(--noir-border)',
            borderRadius: 8,
            padding: '2.5rem 2rem',
          }}
        >
          <div
            style={{
              width: 56,
              height: 56,
              borderRadius: '50%',
              background: 'rgba(201,168,76,0.12)',
              border: '1px solid var(--accent-gold)',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              margin: '0 auto 1.25rem',
            }}
          >
            <Play size={22} style={{ color: 'var(--accent-gold)' }} />
          </div>
          <h1
            style={{
              fontFamily: 'var(--font-serif)',
              fontSize: '1.75rem',
              color: 'var(--noir-text)',
              marginBottom: '0.5rem',
            }}
          >
            {puzzle.title}
          </h1>
          <p
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.75rem',
              color: 'var(--noir-muted)',
              marginBottom: '0.75rem',
            }}
          >
            {puzzle.gridSize}×{puzzle.gridSize} · {puzzle.difficulty}
          </p>
          <p
            style={{
              fontFamily: 'var(--font-serif)',
              fontStyle: 'italic',
              color: 'var(--noir-muted)',
              marginBottom: '1.75rem',
              lineHeight: 1.6,
            }}
          >
            {puzzle.description}
          </p>
          {startError && (
            <p
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: '0.8125rem',
                color: 'var(--accent-red)',
                marginBottom: '1rem',
              }}
            >
              {startError}
            </p>
          )}
          <button
            className="btn-gold"
            onClick={handleStartGame}
            disabled={startLoading}
            style={{
              margin: '0 auto',
              opacity: startLoading ? 0.6 : 1,
              cursor: startLoading ? 'not-allowed' : 'pointer',
            }}
          >
            {startLoading ? (
              <>
                <Loader2 size={14} className="animate-spin" aria-hidden="true" />
                Starting...
              </>
            ) : (
              <>
                <Play size={14} aria-hidden="true" />
                Start Investigation
              </>
            )}
          </button>
          <button
            className="btn-outline"
            onClick={() => navigate('/')}
            style={{ marginTop: '0.75rem' }}
          >
            <ArrowLeft size={14} aria-hidden="true" />
            Back to Case Files
          </button>
        </div>
      </main>
    );
  }

  return (
    <main id="main-content" style={{ flex: 1, display: 'flex', flexDirection: 'column' }}>
      {/* Header bar */}
      <div
        style={{
          padding: '0.875rem 1.25rem',
          borderBottom: '1px solid var(--noir-border)',
          display: 'flex',
          alignItems: 'center',
          gap: '1rem',
          flexWrap: 'wrap',
        }}
      >
        <button
          id="btn-back"
          className="btn-outline"
          onClick={() => navigate('/')}
          aria-label="Back to case files"
        >
          <ArrowLeft size={14} aria-hidden="true" /> Back
        </button>
        <div style={{ flex: 1, minWidth: 0 }}>
          <h1
            style={{
              fontFamily: 'var(--font-serif)',
              fontSize: 'clamp(1rem, 3vw, 1.375rem)',
              fontWeight: 700,
              color: 'var(--noir-text)',
              whiteSpace: 'nowrap',
              overflow: 'hidden',
              textOverflow: 'ellipsis',
            }}
          >
            {puzzle.title}
          </h1>
          <p
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.6875rem',
              color: 'var(--noir-muted)',
              marginTop: 2,
            }}
          >
            {puzzle.gridSize}×{puzzle.gridSize} · {puzzle.difficulty}
          </p>
        </div>
        <MoveCounter moveCount={moveCount} />
        <button
          id="btn-toggle-clues-mobile"
          className="btn-outline mobile-only"
          onClick={() => setCluePanelOpen((o) => !o)}
          aria-expanded={cluePanelOpen}
          aria-label={cluePanelOpen ? 'Hide clues' : 'Show clues'}
        >
          <LayoutPanelLeft size={14} aria-hidden="true" />
          <span style={{ fontSize: '0.8125rem' }}>Clues</span>
        </button>
      </div>

      {/* Error banner */}
      {moveError && (
        <ErrorBanner message={moveError} onDismiss={() => setMoveError(null)} />
      )}

      {/* Play area */}
      <div
        className="play-layout"
        style={{
          flex: 1,
          display: 'grid',
          gap: '1rem',
          padding: '1.25rem',
          maxWidth: 1280,
          margin: '0 auto',
          width: '100%',
          alignItems: 'start',
        }}
      >
        {/* Clue panel */}
        <div className="play-clues" style={{ display: cluePanelOpen ? 'block' : 'none' }}>
          <CluePanel clues={puzzle.clues} suspects={puzzle.suspects} />
        </div>

        {/* Grid column */}
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            gap: '0.875rem',
          }}
        >
          {isSolved && (
            <p
              role="status"
              aria-live="polite"
              style={{
                fontFamily: 'var(--font-serif)',
                fontStyle: 'italic',
                color: 'var(--accent-green)',
                fontSize: '0.9375rem',
                textAlign: 'center',
              }}
            >
              ✓ Case solved - all suspects correctly placed.
            </p>
          )}
          <PuzzleGrid
            cells={cells}
            gridSize={puzzle.gridSize}
            suspects={puzzle.suspects}
            isSolved={isSolved}
            onCellClick={handleCellClick}
            loadingCells={loadingCells}
          />
          <p
            aria-live="polite"
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.75rem',
              textAlign: 'center',
              color: selectedSuspect ? 'var(--accent-gold)' : 'var(--noir-muted)',
              minHeight: '1.2em',
            }}
          >
            {selectedSuspect
              ? `Click a cell to place ${selectedSuspect.name}`
              : isSolved
              ? ''
              : 'Select a suspect, then click a cell. Click an occupied cell to remove.'}
          </p>
        </div>

        {/* Suspect bar */}
        <div className="play-suspects">
          <SuspectBar
            suspects={puzzle.suspects}
            selectedSuspectId={selectedSuspectId}
            placedSuspectIds={placedSuspectIds}
            onSelect={handleSelectSuspect}
            orientation="vertical"
          />
        </div>
      </div>

      {showSolvedBanner && (
        <SolvedBanner
          puzzleTitle={puzzle.title}
          moveCount={moveCount}
          onDismiss={() => {
            setShowSolvedBanner(false);
            navigate('/');
          }}
        />
      )}

      <style>{`
        .play-layout {
          grid-template-columns: min(280px, 30%) 1fr min(220px, 25%);
        }
        @media (max-width: 1023px) {
          .play-layout {
            grid-template-columns: 1fr !important;
          }
          .play-clues {
            order: 3;
          }
          .play-suspects {
            order: 2;
          }
        }
        .mobile-only {
          display: none;
        }
        @media (max-width: 1023px) {
          .mobile-only {
            display: inline-flex;
          }
        }
        @media (min-width: 1024px) {
          .play-clues {
            display: block !important;
          }
        }
        @keyframes spin {
          to {
            transform: rotate(360deg);
          }
        }
        .animate-spin {
          animation: spin 1s linear infinite;
        }
      `}</style>
    </main>
  );
}

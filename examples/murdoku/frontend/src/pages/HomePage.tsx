import { useNavigate } from 'react-router-dom';
import { BookOpen, Plus, Loader2, Inbox } from 'lucide-react';
import { PuzzleCard } from '../components/PuzzleCard';
import { usePuzzleList } from '../hooks/useMurdoku';
import type { PuzzleSummary, PuzzleSummaryWithSolvers } from '../types';

function mapToPuzzleSummary(p: PuzzleSummaryWithSolvers): PuzzleSummary & { totalSolvers: number } {
  return {
    id: p.id,
    title: p.title,
    gridSize: p.gridSize,
    difficulty: p.difficulty,
    clueCount: 0,
    creatorAddress: p.creatorAddress,
    totalSolvers: p.totalSolvers,
  };
}

export function HomePage() {
  const navigate = useNavigate();
  const { puzzles, loading, error, hasMore, loadMore } = usePuzzleList(6);

  return (
    <main
      id="main-content"
      style={{ flex: 1, padding: '2rem 1.25rem', maxWidth: 1280, margin: '0 auto', width: '100%' }}
    >
      {/* Hero */}
      <section
        aria-labelledby="hero-heading"
        style={{
          textAlign: 'center',
          padding: '3rem 1rem 2.5rem',
          marginBottom: '2.5rem',
          borderBottom: '1px solid var(--noir-border)',
        }}
      >
        <div
          aria-hidden="true"
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '0.6875rem',
            color: 'var(--accent-gold)',
            letterSpacing: '0.2em',
            textTransform: 'uppercase',
            marginBottom: '0.75rem',
          }}
        >
          On-Chain Murder Mystery
        </div>

        <h1
          id="hero-heading"
          style={{
            fontFamily: 'var(--font-serif)',
            fontSize: 'clamp(2rem, 5vw, 3.5rem)',
            fontWeight: 700,
            color: 'var(--noir-text)',
            lineHeight: 1.1,
            marginBottom: '1rem',
          }}
        >
          The Case Files
        </h1>

        <p
          style={{
            fontFamily: 'var(--font-serif)',
            fontStyle: 'italic',
            color: 'var(--noir-muted)',
            fontSize: '1.0625rem',
            maxWidth: 520,
            margin: '0 auto 1.75rem',
            lineHeight: 1.6,
          }}
        >
          Every puzzle is a locked room. Every clue is a lie - or is it?
          Place the suspects. Solve the grid. The truth is on-chain.
        </p>

        <button
          id="btn-create-puzzle"
          className="btn-outline"
          onClick={() => navigate('/create')}
          aria-label="Create a new puzzle"
          style={{ borderColor: 'var(--accent-gold)', color: 'var(--accent-gold)' }}
        >
          <Plus size={14} aria-hidden="true" />
          Create a Case
        </button>
      </section>

      {/* Catalog heading */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: '1.25rem',
          gap: '1rem',
        }}
      >
        <h2
          style={{
            fontFamily: 'var(--font-serif)',
            fontSize: '1.375rem',
            color: 'var(--noir-text)',
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
          }}
        >
          <BookOpen size={18} style={{ color: 'var(--accent-gold)' }} aria-hidden="true" />
          Open Cases
        </h2>
        {!loading && puzzles.length > 0 && (
          <span
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.75rem',
              color: 'var(--noir-muted)',
            }}
          >
            {puzzles.length} available
          </span>
        )}
      </div>

      {/* Loading state */}
      {loading && puzzles.length === 0 && (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '4rem 2rem',
            gap: '1rem',
          }}
        >
          <Loader2 size={32} style={{ color: 'var(--accent-gold)' }} className="animate-spin" />
          <p
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.875rem',
              color: 'var(--noir-muted)',
            }}
          >
            Retrieving case files...
          </p>
        </div>
      )}

      {/* Error state */}
      {error && (
        <div
          style={{
            textAlign: 'center',
            padding: '3rem 1rem',
            color: 'var(--accent-red)',
          }}
        >
          <p style={{ fontFamily: 'var(--font-serif)', fontSize: '1.125rem', marginBottom: '0.5rem' }}>
            Unable to load cases
          </p>
          <p style={{ fontFamily: 'var(--font-mono)', fontSize: '0.8125rem', color: 'var(--noir-muted)' }}>
            {error}
          </p>
        </div>
      )}

      {/* Empty state */}
      {!loading && !error && puzzles.length === 0 && (
        <div
          style={{
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            padding: '4rem 2rem',
            gap: '1rem',
          }}
        >
          <Inbox size={40} style={{ color: 'var(--noir-muted)' }} />
          <p
            style={{
              fontFamily: 'var(--font-serif)',
              fontStyle: 'italic',
              fontSize: '1.125rem',
              color: 'var(--noir-muted)',
            }}
          >
            No open cases yet. Be the first to file one.
          </p>
          <button
            className="btn-gold"
            onClick={() => navigate('/create')}
          >
            <Plus size={14} aria-hidden="true" />
            Create a Case
          </button>
        </div>
      )}

      {/* Puzzle grid */}
      {puzzles.length > 0 && (
        <>
          <div
            role="list"
            aria-label="Available puzzles"
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))',
              gap: '1rem',
            }}
          >
            {puzzles.map((puzzle) => {
              const mapped = mapToPuzzleSummary(puzzle);
              return (
                <div key={puzzle.id} role="listitem">
                  <PuzzleCard
                    puzzle={mapped}
                    totalSolvers={mapped.totalSolvers}
                    onClick={(id) => navigate(`/play/${id}`)}
                  />
                </div>
              );
            })}
          </div>

          {/* Load more */}
          {hasMore && (
            <div style={{ textAlign: 'center', marginTop: '2rem' }}>
              <button
                id="btn-load-more"
                className="btn-outline"
                onClick={loadMore}
                disabled={loading}
                style={{
                  opacity: loading ? 0.6 : 1,
                  cursor: loading ? 'not-allowed' : 'pointer',
                }}
              >
                {loading ? (
                  <>
                    <Loader2 size={14} className="animate-spin" aria-hidden="true" />
                    Loading...
                  </>
                ) : (
                  'Load More Cases'
                )}
              </button>
            </div>
          )}
        </>
      )}

      <style>{`
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
        .animate-spin {
          animation: spin 1s linear infinite;
        }
      `}</style>
    </main>
  );
}

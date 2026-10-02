import { parseStateHash, statusLabel, type MatchState } from "../game";

export interface MatchPanelProps { state: MatchState | null; busy: boolean; onRefresh: () => void; }

/**
 * Read-only view of the on-chain match: everything the checkpoint template
 * stores, nothing about the off-chain simulation.
 */
export function MatchPanel({ state, busy, onRefresh }: MatchPanelProps) {
  return (
    <div className="game">
      <div className="game-heading">
        <div>
          <span className="label">Match state</span>
          <strong>{state ? statusLabel(state.status) : "No match loaded"}</strong>
        </div>
        <button type="button" onClick={onRefresh} disabled={busy}>Refresh</button>
      </div>
      {state ? (
        <>
          <dl className="fields">
            <div><dt>Player</dt><dd><code>{state.player}</code></dd></div>
            <div><dt>Last tick</dt><dd>{state.last_tick}</dd></div>
            <div><dt>Last hash</dt><dd><code>{hashDisplay(state.last_hash)}</code></dd></div>
            <div><dt>Last score</dt><dd>{state.last_score}</dd></div>
          </dl>
          {state.status !== 0 && <p className="result" role="status">{statusLabel(state.status)}</p>}
        </>
      ) : (
        <div className="empty">No checkpoint match loaded. Start one or refresh once the contract is configured.</div>
      )}
    </div>
  );
}

function hashDisplay(hash: MatchState["last_hash"]): string {
  if (hash === 0 || hash === 0n || hash === "0") return "none";
  const parsed = parseStateHash(hash);
  return parsed ? parsed.label : String(hash);
}

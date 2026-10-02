import { useCallback, useEffect, useState } from "react";
import {
  CONTRACT_ID,
  ClientError,
  getState,
  initMatch,
  makeMove,
  requireTestnet
} from "./contract";
import { EntityCard } from "./components/EntityCard";
import {
  entityForAddress,
  matchStatusLabel,
  type MatchState
} from "./game";

export default function App() {
  const [address, setAddress]     = useState("");
  const [opponent, setOpponent]   = useState("");
  const [state, setState]         = useState<MatchState | null>(null);
  const [busy, setBusy]           = useState(false);
  const [error, setError]         = useState("");
  const [notice, setNotice]       = useState(
    "Connect Freighter on Stellar Testnet to begin."
  );

  // ── Connect ──────────────────────────────────────────────────────────────

  const connect = useCallback(async () => {
    setBusy(true);
    setError("");
    try {
      const wallet = await requireTestnet();
      setAddress(wallet);
      setNotice("Connected to Stellar Testnet.");
      try {
        setState(await getState());
      } catch {
        setState(null);
      }
    } catch (cause) {
      setError(
        cause instanceof ClientError
          ? cause.message
          : "Unable to connect to Freighter."
      );
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => { void connect(); }, [connect]);

  // ── Refresh ───────────────────────────────────────────────────────────────

  const refresh = async () => {
    setBusy(true);
    setError("");
    try {
      setState(await getState());
      setNotice("State refreshed from Soroban.");
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : "Unable to load match state."
      );
    } finally {
      setBusy(false);
    }
  };

  // ── Init match ────────────────────────────────────────────────────────────

  const start = async () => {
    setBusy(true);
    setError("");
    try {
      const hash = await initMatch(opponent);
      setState(await getState());
      setNotice(`Match started. Transaction ${hash.slice(0, 8)}…`);
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : "Unable to start the match."
      );
    } finally {
      setBusy(false);
    }
  };

  // ── Make move ─────────────────────────────────────────────────────────────

  const play = async (entityId: number) => {
    if (!state || !state.active) return;
    const myEntity = entityForAddress(state.entities, address);
    if (!myEntity || myEntity.id !== entityId) return;
    setBusy(true);
    setError("");
    try {
      const hash = await makeMove(entityId);
      setState(await getState());
      setNotice(`Move submitted. Transaction ${hash.slice(0, 8)}…`);
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : "Unable to submit the move."
      );
    } finally {
      setBusy(false);
    }
  };

  // ── Derived display values ────────────────────────────────────────────────

  const statusText = state ? matchStatusLabel(state) : "No match loaded";

  return (
    <main className="shell">
      <section className="card">
        <p className="eyebrow">Cougr · storageworld reference client</p>
        <h1>StorageWorld match</h1>
        <p className="muted">
          Direct RPC client for the storageworld template. Freighter signs
          wallet transactions. Both entities are rendered so a partial update
          (one entity's value changes, the other's does not) is visible after
          each move.
        </p>

        {/* ── Wallet panel ─────────────────────────────────────────── */}
        <div className="panel">
          <div className="row">
            <div>
              <span className="label">Wallet</span>
              <code>
                {address
                  ? `${address.slice(0, 8)}…${address.slice(-6)}`
                  : "Not connected"}
              </code>
            </div>
            <button
              type="button"
              onClick={() => void connect()}
              disabled={busy}
            >
              {address ? "Reconnect" : "Connect Freighter"}
            </button>
          </div>
          <p className="network">Network: Stellar Testnet only</p>
        </div>

        {/* ── Init panel ───────────────────────────────────────────── */}
        <div className="panel">
          <label htmlFor="opponent">Second player address</label>
          <input
            id="opponent"
            value={opponent}
            onChange={(e) => setOpponent(e.target.value)}
            placeholder="G..."
            spellCheck={false}
            disabled={busy}
          />
          <button
            type="button"
            onClick={() => void start()}
            disabled={busy || !address || !opponent}
          >
            {busy ? "Working…" : "Start match"}
          </button>
          {!CONTRACT_ID && (
            <p className="warning">VITE_CONTRACT_ID is not configured.</p>
          )}
        </div>

        {/* ── Error banner ─────────────────────────────────────────── */}
        {error && (
          <div className="error" role="alert">
            {error}
          </div>
        )}

        {/* ── Match state ──────────────────────────────────────────── */}
        <div className="game">
          <div className="game-heading">
            <div>
              <span className="label">Match state</span>
              <strong>{statusText}</strong>
            </div>
            <button
              type="button"
              onClick={() => void refresh()}
              disabled={busy || !address}
            >
              Refresh
            </button>
          </div>

          {state ? (
            <>
              <div className="entities">
                {state.entities.map((entity) => (
                  <EntityCard
                    key={entity.id}
                    entity={entity}
                    isOwn={entity.owner === address}
                    disabled={busy || !state.active}
                    onMove={(id) => void play(id)}
                  />
                ))}
              </div>
              <p className="muted">
                {state.move_count} move{state.move_count === 1 ? "" : "s"}{" "}
                recorded · StorageWorld only rewrites the entity that changed
              </p>
            </>
          ) : (
            <div className="empty">{notice}</div>
          )}
        </div>

        <footer>
          Contract: <code>{CONTRACT_ID || "not configured"}</code>
        </footer>
      </section>
    </main>
  );
}

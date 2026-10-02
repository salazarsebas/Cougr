import { useCallback, useEffect, useState } from "react";
import {
  CONTRACT_ID,
  ClientError,
  getState,
  moveEntity,
  requireTestnet,
} from "./contract";
import { EntityCard } from "./components/EntityCard";
import { turnLabel, type MatchState, type Direction } from "./game";

export default function App() {
  const [address, setAddress] = useState("");
  const [state, setState] = useState<MatchState | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState(
    "Connect Freighter on Stellar Testnet to begin."
  );

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

  useEffect(() => {
    void connect();
  }, [connect]);

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

  const move = async (entityId: number, direction: Direction) => {
    if (!state) return;
    // Only allow the move for the currently active entity
    if (state.active_entity !== entityId) return;
    setBusy(true);
    setError("");
    try {
      const hash = await moveEntity(entityId, direction);
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

  const turnText = !state
    ? "No match loaded"
    : turnLabel(state);

  return (
    <main className="shell">
      <section className="card">
        <p className="eyebrow">Cougr · storageworld reference client</p>
        <h1>StorageWorld match</h1>
        <p className="muted">
          Direct RPC client for the storageworld template. Move one entity and
          watch the other remain unchanged — StorageWorld only writes back the
          fields that changed.
        </p>

        {/* Wallet panel */}
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

        {!CONTRACT_ID && (
          <p className="warning">VITE_CONTRACT_ID is not configured.</p>
        )}

        {error && (
          <div className="error" role="alert">
            {error}
          </div>
        )}

        {/* Match state */}
        <div className="game">
          <div className="game-heading">
            <div>
              <span className="label">Match state</span>
              <strong>{turnText}</strong>
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
              <p className="muted">
                Move count: {state.move_count}
              </p>
              {/* Render both entities side by side so partial updates are visible */}
              <div className="entities">
                <EntityCard
                  state={state.entity_0}
                  isActive={state.active_entity === 0}
                  disabled={busy || !address}
                  onMove={(dir) => void move(0, dir)}
                />
                <EntityCard
                  state={state.entity_1}
                  isActive={state.active_entity === 1}
                  disabled={busy || !address}
                  onMove={(dir) => void move(1, dir)}
                />
              </div>
              <p className="notice muted">{notice}</p>
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

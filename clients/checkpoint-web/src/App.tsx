import { useCallback, useEffect, useState } from "react";
import { CONTRACT_ID, ClientError, commitCheckpoint, disputeCheckpoint, finalizeMatch, getState, requireTestnet, startMatch } from "./contract";
import { MatchPanel } from "./components/MatchPanel";
import { actionRejection } from "./game";
import type { ActionResult, MatchState } from "./game";

export default function App() {
  const [address, setAddress] = useState("");
  const [player, setPlayer] = useState("");
  const [tick, setTick] = useState("100");
  const [stateHash, setStateHash] = useState("48879");
  const [score, setScore] = useState("0");
  const [claimedHash, setClaimedHash] = useState("48880");
  const [state, setState] = useState<MatchState | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("Connect Freighter on Stellar Testnet to begin.");

  const connect = useCallback(async () => {
    setBusy(true); setError("");
    try {
      const wallet = await requireTestnet();
      setAddress(wallet);
      setNotice("Connected to Stellar Testnet.");
      try { setState(await getState()); setPlayer(wallet); } catch { setState(null); }
    } catch (cause) {
      setError(cause instanceof ClientError ? cause.message : "Unable to connect to Freighter.");
    } finally { setBusy(false); }
  }, []);

  useEffect(() => { void connect(); }, [connect]);

  const refresh = async () => {
    setBusy(true); setError("");
    try { setState(await getState()); setNotice("State refreshed from Soroban."); }
    catch (cause) { setError(cause instanceof Error ? cause.message : "Unable to load match state."); }
    finally { setBusy(false); }
  };

  // The template returns ActionResult { success: false, message } for a
  // rejected action instead of trapping, so a late dispute is a successful
  // transaction carrying a rejection. Surface it as an error, not a crash.
  const run = async (action: () => Promise<{ hash: string; result: ActionResult }>, done: string) => {
    setBusy(true); setError("");
    try {
      const invocation = await action();
      setState(invocation.result.match_state);
      if (invocation.result.success) {
        setNotice(`${done} Transaction ${invocation.hash.slice(0, 8)}…`);
      } else {
        setError(`The contract rejected the action: ${actionRejection(invocation.result.message)}`);
      }
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : "The transaction did not go through.");
    } finally { setBusy(false); }
  };

  const requirePlayer = (): string | null => (player.trim() ? null : "Enter the player or challenger address first.");

  const start = () => { const problem = requirePlayer(); if (problem) { setError(problem); return; } void run(() => startMatch(player), "Match started."); };
  const commit = () => { const problem = requirePlayer(); if (problem) { setError(problem); return; } void run(() => commitCheckpoint(player, tick, stateHash, score), "Checkpoint committed."); };
  const dispute = () => { const problem = requirePlayer(); if (problem) { setError(problem); return; } void run(() => disputeCheckpoint(player, tick, claimedHash), "Dispute submitted."); };
  const finalize = () => { const problem = requirePlayer(); if (problem) { setError(problem); return; } void run(() => finalizeMatch(player), "Match finalised."); };

  return <main className="shell"><section className="card">
    <p className="eyebrow">Cougr · checkpoint reference client</p>
    <h1>Checkpoint match</h1>
    <p className="muted">Direct RPC client for the checkpoint template. Ticks stay off chain; the contract stores a tick index, state hash, and score per checkpoint, with a dispute window.</p>

    <div className="panel"><div className="row"><div><span className="label">Wallet</span><code>{address ? `${address.slice(0, 8)}…${address.slice(-6)}` : "Not connected"}</code></div><button type="button" onClick={() => void connect()} disabled={busy}>{address ? "Reconnect" : "Connect Freighter"}</button></div><p className="network">Network: Stellar Testnet only</p></div>

    <div className="panel">
      <label htmlFor="player">Player or challenger address</label>
      <input id="player" value={player} onChange={(e) => setPlayer(e.target.value)} placeholder="G..." spellCheck={false} disabled={busy} />
      <div className="grid">
        <div><label htmlFor="tick">Tick index</label><input id="tick" inputMode="numeric" value={tick} onChange={(e) => setTick(e.target.value)} disabled={busy} /></div>
        <div><label htmlFor="hash">State hash (u64)</label><input id="hash" inputMode="numeric" value={stateHash} onChange={(e) => setStateHash(e.target.value)} disabled={busy} /></div>
        <div><label htmlFor="score">Score</label><input id="score" inputMode="numeric" value={score} onChange={(e) => setScore(e.target.value)} disabled={busy} /></div>
        <div><label htmlFor="claimed">Dispute claimed hash (u64)</label><input id="claimed" inputMode="numeric" value={claimedHash} onChange={(e) => setClaimedHash(e.target.value)} disabled={busy} /></div>
      </div>
      <div className="actions">
        <button type="button" onClick={start} disabled={busy || !address}>Start match</button>
        <button type="button" onClick={commit} disabled={busy || !address}>Commit checkpoint</button>
        <button type="button" onClick={dispute} disabled={busy || !address}>Dispute checkpoint</button>
        <button type="button" onClick={finalize} disabled={busy || !address}>Finalise match</button>
      </div>
      {!CONTRACT_ID && <p className="warning">VITE_CONTRACT_ID is not configured.</p>}
      <p className="muted">Tick, hash, and score feed commit_checkpoint. Tick and the claimed hash feed dispute_checkpoint: use the committed tick and a hash that differs from the committed one, inside the window, or the contract rejects the dispute.</p>
    </div>

    {error && <div className="error" role="alert">{error}</div>}
    {!error && notice && <div className="muted" role="status">{notice}</div>}
    <MatchPanel state={state} busy={busy} onRefresh={() => void refresh()} />
    <footer>Contract: <code>{CONTRACT_ID || "not configured"}</code></footer>
  </section></main>;
}

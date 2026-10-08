import { useCallback, useEffect, useState } from "react";
import { CONTRACT_ID, ClientError, getState, initGame, makeMove, requireTestnet } from "./contract";
import { GameBoard } from "./components/GameBoard";
import { statusLabel, type GameState } from "./game";
import { bytesToHex, mapBrowserCredentialToSecp256r1Bytes } from "./passkey";

export default function App() {
  const [address, setAddress] = useState("");
  const [opponent, setOpponent] = useState("");
  const [state, setState] = useState<GameState | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("Connect Freighter on Stellar Testnet to begin.");
  const [passkeyStatus, setPasskeyStatus] = useState("No passkey generated yet.");
  const [passkeyHex, setPasskeyHex] = useState("");

  const connect = useCallback(async () => {
    setBusy(true); setError("");
    try {
      const wallet = await requireTestnet();
      setAddress(wallet);
      setNotice("Connected to Stellar Testnet.");
      try { setState(await getState()); } catch { setState(null); }
    } catch (cause) {
      setError(cause instanceof ClientError ? cause.message : "Unable to connect to Freighter.");
    } finally { setBusy(false); }
  }, []);

  useEffect(() => { void connect(); }, [connect]);

  const refresh = async () => {
    setBusy(true); setError("");
    try { setState(await getState()); setNotice("State refreshed from Soroban."); }
    catch (cause) { setError(cause instanceof Error ? cause.message : "Unable to load game state."); }
    finally { setBusy(false); }
  };

  const start = async () => {
    setBusy(true); setError("");
    try {
      const hash = await initGame(opponent);
      setState(await getState());
      setNotice(`Match started. Transaction ${hash.slice(0, 8)}…`);
    } catch (cause) { setError(cause instanceof Error ? cause.message : "Unable to start the match."); }
    finally { setBusy(false); }
  };

  const play = async (position: number) => {
    if (!state) return;
    const myTurn = state.is_x_turn === (address === state.player_x);
    if (!myTurn || state.status !== 0) return;
    setBusy(true); setError("");
    try {
      const hash = await makeMove(position);
      setState(await getState());
      setNotice(`Move submitted. Transaction ${hash.slice(0, 8)}…`);
    } catch (cause) { setError(cause instanceof Error ? cause.message : "Unable to submit the move."); }
    finally { setBusy(false); }
  };

  const createPasskey = async () => {
    if (!window.isSecureContext && window.location.hostname !== "localhost") {
      setPasskeyStatus("WebAuthn requires HTTPS or localhost. Use the browser demo on a secure origin.");
      return;
    }

    setBusy(true); setError("");
    try {
      const challenge = new Uint8Array(32);
      crypto.getRandomValues(challenge);

      const credential = await navigator.credentials.create({
        publicKey: {
          challenge,
          rp: { name: "Stellar Drips Wave", id: window.location.hostname === "localhost" ? "localhost" : window.location.hostname },
          user: {
            id: new TextEncoder().encode("stellar-drips-wave-demo"),
            name: "stellar-drips-wave-demo",
            displayName: "Stellar Drips Wave Demo"
          },
          pubKeyCredParams: [{ alg: -7, type: "public-key" }],
          authenticatorSelection: { authenticatorAttachment: "platform", userVerification: "preferred" },
          timeout: 60000,
          attestation: "none"
        }
      });

      if (!credential) {
        throw new Error("The browser did not create a WebAuthn credential.");
      }

      const keyBytes = mapBrowserCredentialToSecp256r1Bytes(credential as { response?: { getPublicKey?: () => ArrayBuffer | ArrayBufferView | null | undefined } });
      setPasskeyHex(bytesToHex(keyBytes));
      setPasskeyStatus("Passkey created and mapped to the secp256r1 bytes that tap_battle expects.");
      setNotice("Browser passkey created. This is the mapper proof used before registration on tap_battle.");
    } catch (cause) {
      setPasskeyStatus(cause instanceof Error ? cause.message : "Unable to create a browser passkey.");
    } finally { setBusy(false); }
  };

  const myTurn = state ? state.is_x_turn === (address === state.player_x) : false;
  const turnText = !state ? "No match loaded" : state.status !== 0 ? statusLabel(state.status) : myTurn ? "Your turn" : "Opponent's turn";

  return <main className="shell"><section className="card">
    <p className="eyebrow">Cougr · Stellar Drips Wave</p>
    <h1>WebAuthn passkey demo</h1>
    <p className="muted">This browser flow creates a passkey, maps it through the secp256r1 byte shim that matches the tap_battle contract, and proves the browser and contract agree.</p>

    <div className="panel"><div className="row"><div><span className="label">Wallet</span><code>{address ? `${address.slice(0, 8)}…${address.slice(-6)}` : "Not connected"}</code></div><button type="button" onClick={() => void connect()} disabled={busy}>{address ? "Reconnect" : "Connect Freighter"}</button></div><p className="network">Network: Stellar Testnet only</p></div>

    <div className="panel"><div className="row"><div><span className="label">Browser passkey</span><strong>{passkeyStatus}</strong></div><button type="button" onClick={() => void createPasskey()} disabled={busy}>Create passkey</button></div>{passkeyHex && <code className="passkey-code">{passkeyHex}</code>}</div>

    <div className="panel"><label htmlFor="opponent">Player O address</label><input id="opponent" value={opponent} onChange={(e) => setOpponent(e.target.value)} placeholder="G..." spellCheck={false} disabled={busy}/><button type="button" onClick={() => void start()} disabled={busy || !address || !opponent}>{busy ? "Working…" : "Start match"}</button>{!CONTRACT_ID && <p className="warning">VITE_CONTRACT_ID is not configured.</p>}</div>

    {error && <div className="error" role="alert">{error}</div>}
    <div className="game"><div className="game-heading"><div><span className="label">Match state</span><strong>{turnText}</strong></div><button type="button" onClick={() => void refresh()} disabled={busy || !address}>Refresh</button></div>{state ? <><GameBoard state={state} disabled={busy || !myTurn} onMove={(position) => void play(position)}/>{state.status !== 0 && <p className="result" role="status">{statusLabel(state.status)}</p>}<p className="muted">{state.move_count}/9 moves played</p></> : <div className="empty">{notice}</div>}</div>
    <footer>Contract: <code>{CONTRACT_ID || "not configured"}</code></footer>
  </section></main>;
}

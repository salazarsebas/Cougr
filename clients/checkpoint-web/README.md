# Cougr checkpoint web client

Small browser reference client for the `cougr new --template checkpoint` contract.

It connects to Freighter, refuses non-Testnet wallets, reads `get_state` directly from Soroban RPC, and signs `commit_checkpoint`, `dispute_checkpoint`, and `finalize_match` with the connected wallet. It does not use custodial keys or `packages/sdk-core`.

The checkpoint template implements the off-chain-ticks, on-chain-checkpoints model described in issue #329: the client (or game) simulates ticks locally, and the contract only records a tick index, a state hash, and a score per checkpoint, plus a dispute window before the result is finalised.

## Configure

Create `clients/checkpoint-web/.env.local`:

    VITE_CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

Use the contract instance ID returned by the Stellar CLI after deploying the generated checkpoint contract.

## Deploy the generated contract

From the generated project:

    cougr new my-game --template checkpoint
    cd my-game
    cargo test
    stellar contract build
    stellar contract deploy --wasm target/wasm32v1-none/release/my_game.wasm --source-account alice --network testnet

Fund the Testnet wallet used by Freighter before submitting transactions. The template's `DISPUTE_WINDOW_LEDGERS` constant (default 1000, roughly 83 minutes at ~5 s/ledger) controls how long a checkpoint can be disputed.

## Contract methods used

The client calls exactly these methods, as documented in issue #329's objective (`commit_checkpoint` with a tick index, state hash, and score, and a dispute path inside a window):

- `start_match(player) -> MatchState`
- `commit_checkpoint(player, tick, state_hash, score) -> ActionResult`
- `dispute_checkpoint(challenger, tick, claimed_hash) -> ActionResult`
- `finalize_match(player) -> ActionResult`
- `get_state() -> MatchState`

`MatchState` carries `player`, `last_tick`, `last_hash` (u64), `last_score`, and `status` (0 running, 1 finalised, 2 disputed).

Rejections come back as `ActionResult { success: false, message }` on an otherwise successful transaction, not as contract traps. `commit_checkpoint` can reject with `notrun` or `oldtick`; `dispute_checkpoint` with `notrun`, `badtick`, `toolate`, or `nohash`; `finalize_match` with `notrun` or `window`. The client maps these codes to safe UI messages, so a late dispute surfaces as a rejection, not a crash.

`commit_checkpoint`, `dispute_checkpoint`, and `finalize_match` are signed by the connected Freighter wallet. The wallet is the transaction source and no secret key is stored by the client.

## Interface source

This client was built against the interface documented in issue #329, with method names and argument order aligned to the checkpoint template proposed in PR #357. Once that template lands, `cli/templates/checkpoint/README.md` becomes the authoritative reference; align any drifted names there.

## Development

    npm install
    npm run dev
    npm test
    npm run build

The unit tests use fixture state only. They do not connect to Freighter, a live wallet, or a deployed contract.

## Scope

The client intentionally stays limited to the checkpoint template. It does not implement custodial keys, Studio, Godot, a physics engine or frame-by-frame move UI, a general template-agnostic renderer, the turn-based or storageworld clients, or mainnet. Mainnet is refused in the UI.

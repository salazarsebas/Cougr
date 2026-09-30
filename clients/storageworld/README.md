# Cougr StorageWorld web client

Browser reference client for the `cougr new --template storageworld` contract.

> **Template naming note** – The CLI template this client targets is tracked in
> issue [#330](https://github.com/salazarsebas/Cougr/issues/330) and PR
> [#358](https://github.com/salazarsebas/Cougr/pull/358).  At the time this
> client was written (built against the documented interface for #330,
> commit `main` 2026-09-30), the working template name is **storageworld** and
> the expected CLI flag is `--template storageworld`.  If #330 lands under a
> different name (the issue body notes the name *may change if it collides*),
> this client's directory, package name, and the contract method names below
> will need to be aligned with whatever actually merged.

It connects to Freighter, refuses non-Testnet wallets, reads `get_state`
directly from Soroban RPC, and signs `move_entity` with the connected wallet.
It does **not** use custodial keys or `packages/sdk-core`.

Two entities are always rendered side-by-side so the partial-update behaviour
of StorageWorld is directly visible: moving entity 0 leaves entity 1's position
and step counter completely unchanged, because StorageWorld only writes back the
fields that actually changed.

## Configure

Create `clients/storageworld/.env.local`:

```
VITE_CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

Use the contract instance ID returned by `stellar contract deploy` after
deploying the generated storageworld contract.

## Deploy the generated contract

From a generated storageworld project (once PR #358 merges):

```bash
cougr new my-storage-game --template storageworld
cd my-storage-game
cargo test
stellar contract build
stellar contract deploy \
  --wasm target/wasm32v1-none/release/my_storage_game.wasm \
  --source-account alice \
  --network testnet
```

Fund the Testnet wallet used by Freighter before submitting transactions.

## Contract methods used

The client calls exactly these methods from the storageworld template
(working names per issue #330 / PR #358 interface):

| Method | Signature | Notes |
|--------|-----------|-------|
| `get_state` | `get_state() → MatchState` | Read-only; called via `simulateTransaction` |
| `move_entity` | `move_entity(player, entity_id, dir) → MatchState` | Signed by the connected Freighter wallet |

`MatchState` contains two `EntityState` values (`entity_0`, `entity_1`),
`active_entity` (0 or 1), and `move_count`.

`move_entity` is signed by the connected Freighter wallet. The wallet is the
transaction source and no secret key is stored by the client.

## Development

```bash
npm install
npm run dev
npm test
npm run build
```

Unit tests use fixture state only. They do not connect to Freighter, a live
wallet, or a deployed contract.

## Scope

The client is intentionally limited to the storageworld contract:

- No custodial keys
- No `packages/sdk-core` dependency
- No mainnet (the UI refuses any non-Testnet network passphrase)
- No Studio or Godot
- No general template-agnostic renderer

## Security

- No secret or private key is committed or referenced.
- Mainnet (`Networks.PUBLIC`) is refused at connect time.
- Contract ID is supplied via environment variable only (`.env.local`, never
  committed).

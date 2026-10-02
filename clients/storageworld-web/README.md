# clients/storageworld-web

Freighter web client for the **storageworld** Cougr template.

Reads a StorageWorld-backed match directly from Soroban RPC and submits moves
signed by the connected Freighter wallet. Two entities are rendered so that a
**partial update** — one entity's field changes, the other's does not — is
visible in the UI after each move.

---

## Template name caveat

This client is built against the interface documented in
[issue #330](https://github.com/salazarsebas/Cougr/issues/330) and
[PR #358](https://github.com/salazarsebas/Cougr/pull/358), which have **not**
merged to `main` as of the commit this client was written against. The template
working name is **storageworld**. If PR #358 lands under a different name or
directory, update the following before deploying:

- `VITE_CONTRACT_ID` (your `.env.local`) — re-deploy from the final template
  directory.
- `CONTRACT_METHOD_INIT` / `CONTRACT_METHOD_MOVE` / `CONTRACT_METHOD_GET_STATE`
  in `src/contract.ts` — if method names differ from `init_match`, `make_move`,
  `get_state`.
- The package name in `package.json` and this README.

---

## Contract methods used

| Method | Called by | Arguments |
|---|---|---|
| `init_match` | `initMatch()` in `contract.ts` | `(first_player: Address, second_player: Address)` |
| `make_move` | `makeMove()` in `contract.ts` | `(player: Address, entity_id: u32)` |
| `get_state` | `getState()` in `contract.ts` | *(none)* |

These names follow the interface specified in issue #330. Confirm against the
final template README when PR #358 merges.

---

## Quick start

### 1. Deploy the contract

Once PR #358 merges and the `storageworld` template is available on `main`:

```bash
# Scaffold a new game from the template
cargo install cougr-cli
cougr new my-storageworld-game --template storageworld
cd my-storageworld-game

# Build and deploy to Testnet
cargo test
stellar contract build
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_storageworld_game.wasm \
  --source alice \
  --network testnet
# Copy the printed contract ID.
```

### 2. Configure the client

```bash
cd clients/storageworld-web
echo 'VITE_CONTRACT_ID=<paste-contract-id-here>' > .env.local
```

**Never commit `.env.local`.** It is listed in `.gitignore` by convention; no
secret or testnet key is committed in this repository.

### 3. Run locally

```bash
npm install
npm run dev
# Open http://localhost:5173
```

### 4. Connect Freighter

- Install the [Freighter browser extension](https://www.freighter.app/).
- Switch to **Stellar Testnet** in Freighter. The client refuses Mainnet and
  shows an error if the connected network is not Testnet.
- Fund both player addresses on Testnet via
  [Friendbot](https://friendbot.stellar.org/).

### 5. Start a match

Paste the second player's Testnet address into the "Second player address" field
and click **Start match**. The connected wallet becomes the first entity owner;
the second address becomes the second.

### 6. Make a move

Each entity card shows a **Make move** button only for the entity you own.
Clicking it calls `make_move` on-chain. After the transaction confirms the UI
re-reads `get_state`; the other entity's value does not change — that is the
partial update StorageWorld exists to demonstrate.

---

## Building for production

```bash
npm run build
# Output is in dist/
```

---

## Tests

Unit tests cover fixture state validation, the partial-update invariant (a move
on entity A leaves entity B unchanged), and EntityCard rendering for own vs.
opponent entities. No live wallet or live contract is required.

```bash
npm test
```

Expected output:

```
✓ src/game.test.ts          (20 tests)
✓ src/components/EntityCard.test.tsx  (9 tests)

 Test Files  2 passed (2)
      Tests  29 passed (29)
```

---

## CI

The workflow at
[`.github/workflows/storageworld-client.yml`](../../.github/workflows/storageworld-client.yml)
runs only when files under `clients/storageworld-web/**` change. It does **not**
run the Rust `clippy -D warnings` gate; Rust-only changes do not trigger it.

---

## Network policy

Mainnet is refused at the Freighter layer (`requireTestnet()` in
`src/contract.ts`). If Freighter reports any network passphrase other than
`Networks.TESTNET` the client surfaces an error and does not submit any
transaction.

---

## Directory layout

```
clients/storageworld-web/
├── index.html
├── package.json
├── vite.config.ts          # also serves as vitest config
├── tsconfig.json
├── tsconfig.app.json
├── tsconfig.node.json
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── contract.ts         # Freighter connect, testnet guard, RPC calls
    ├── game.ts             # types, guards, pure helpers
    ├── styles.css
    ├── game.test.ts
    └── components/
        ├── EntityCard.tsx
        └── EntityCard.test.tsx
```

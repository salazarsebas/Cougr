# Murdoku Frontend

The frontend for the Murdoku game is a Vite-powered React web application that provides the user interface for browsing, playing, and creating murder mystery logic puzzles.

## Responsibilities & Scope

### What the Frontend Does
* **User Authentication**: Integrates the Pollar SDK (`@pollar/react`) for social/email login and embedded Stellar wallet provisioning.
* **Session Key Management**: Requests matches/actions-scoped temporary session keys via `SessionBuilder` so players can play seamlessly without repeated wallet popups.
* **UI Rendering**: Displays the game board, active suspects, clue card sidebar, puzzle catalog, and a step-by-step puzzle creator.
* **Transaction Construction**: Formulates and broadcasts transaction payloads (e.g. calling `place_suspect` and `submit_puzzle`) using the Stellar/Soroban JS SDK.

### What the Frontend Does NOT Do
* **Game Logic Enforcement**: The frontend does not validate whether a suspect placement is valid, whether row/column duplicates exist, or whether clues are satisfied. All rules are strictly checked and enforced on-chain by the Soroban contract systems.
* **Win/Solve Verification**: The frontend does not decide if a puzzle is solved. It queries the contract's `is_solved` state to determine whether to trigger a victory state.

---

## Environment Variables

Copy `.env.example` to `.env` in the frontend directory:
```bash
cp .env.example .env
```

Define the following environment variables:

| Variable | Description | Where to Get |
|---|---|---|
| `VITE_POLLAR_API_KEY` | The publishable API Key for the Pollar SDK. | Log into the [Pollar Developer Dashboard](https://dashboard.pollar.xyz), create a new application, and retrieve the public client key. |
| `VITE_CONTRACT_ID` | The hex-encoded ID of the deployed Murdoku Soroban contract. | Obtained from your terminal output after executing `stellar contract deploy` on Stellar Testnet. |

---

## Development & Build Commands

Run the following commands inside the `examples/murdoku/frontend` directory:

```bash
# Install dependencies
npm install

# Start local development server (with hot module replacement)
npm run dev

# Build the production bundle
npm run build

# Preview the production build locally
npm run preview
```

---

## Targeting a Custom Contract

To target a different instance of the Murdoku smart contract (for example, a locally running Soroban RPC node or your own deployed testnet instance):
1. Deploy your contract instance using the `stellar contract deploy` CLI command.
2. Open the frontend `.env` file.
3. Update the `VITE_CONTRACT_ID` variable with your new contract address:
   ```env
   VITE_CONTRACT_ID=CC...
   ```
4. Restart your development server (`npm run dev`) or rebuild the bundle (`npm run build`) to apply the changes.

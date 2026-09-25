# Cougr turn-based web client

Small browser reference client for the `cougr new --template turn-based` contract.

It connects to Freighter, refuses non-Testnet wallets, reads `get_state` directly from Soroban RPC, starts a match with `init_game`, and signs `make_move` with the connected wallet. It does not use custodial keys or `packages/sdk-core`.

## Configure

Create `clients/web/.env.local`:

    VITE_CONTRACT_ID=CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

Use the contract instance ID returned by the Stellar CLI after deploying the generated turn-based contract.

## Deploy the generated contract

From the generated project:

    cougr new my-game --template turn-based
    cd my-game
    cargo test
    stellar contract build
    stellar contract deploy --wasm target/wasm32v1-none/release/my_game.wasm --source-account alice --network testnet

The generated template documents the required WASM target and Stellar CLI prerequisites. Fund the Testnet wallet used by Freighter before submitting transactions.

## Contract methods used

The client calls exactly these methods from `cli/templates/turn-based/README.md`:

- `init_game(player_x, player_o) -> GameState`
- `make_move(player, position) -> MoveResult`
- `get_state() -> GameState`

`make_move` is signed by the connected Freighter wallet. The wallet is the transaction source and no secret key is stored by the client.

## Development

    npm install
    npm run dev
    npm test
    npm run build

The unit tests use fixture state only. They do not connect to Freighter, a live wallet, or a deployed contract.

## Scope

The client intentionally stays limited to the current 3×3 turn-based contract. It does not implement custodial keys, Studio, Godot, a general component renderer, configurable board sizes, or mainnet.

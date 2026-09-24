# Cougr Turn-Based Godot Addon

This is a Godot 4 addon for interacting with the Cougr turn-based smart contract template on Stellar testnet.

## Setup
1. Import this addon into your Godot project by copying the `addons/cougr_turn_based` directory.
2. Enable the addon in Project Settings -> Plugins.
3. Configure the following environment variables or project settings:
   - `COUGR_CONTRACT_ID`: The deployed contract ID.
   - `COUGR_RPC_URL`: The Stellar RPC URL (e.g. `https://soroban-testnet.stellar.org:443`).
   - `COUGR_TESTNET_KEY`: A testnet secret key for signing transactions (starting with S).

## Sample Scene
A sample 3x3 board scene is provided in `sample_scene.tscn`. You can run it directly to play a game against the contract on testnet.


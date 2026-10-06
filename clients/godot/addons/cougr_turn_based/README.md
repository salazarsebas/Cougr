# Cougr Turn-Based Godot Addon

Godot 4 addon for Cougr's turn-based template. It allows your Godot game to read the board state and submit `make_move` transactions on Stellar testnet.

## Import the addon

1. Copy `addons/cougr_turn_based` into your project's `addons` directory.
2. Enable **Cougr Turn-Based** in Project Settings > Plugins.

The client is a `Node`. You can add it to your scene:

```gdscript
const Client := preload("res://addons/cougr_turn_based/turn_based_client.gd")

func _ready() -> void:
    var client := Client.new()
    add_child(client)
    var state: Dictionary = await client.read_state()
    if state["success"]:
        print(state["result_scval"])
```

## Configure

The client reads these environment variables:

- `COUGR_RPC_URL`: Soroban RPC endpoint. Defaults to testnet.
- `COUGR_CONTRACT_ID`: The deployed contract, in `C...` form.
- `COUGR_SOURCE_ACCOUNT`: The account that signs, in `G...` form.
- `COUGR_SIGNING_KEY`: Stellar CLI identity or testnet secret key.

```sh
export COUGR_RPC_URL=https://soroban-testnet.stellar.org
export COUGR_CONTRACT_ID=C...
export COUGR_SOURCE_ACCOUNT=G...
export COUGR_SIGNING_KEY=cougr-testnet
```

## Provide the key via env

The addon delegates signing to the Stellar CLI. The CLI must be on `PATH`.
Set `COUGR_SIGNING_KEY` to an identity or secret key (`S...`), and it will be passed to `stellar tx sign`. The key stays out of git.

```sh
stellar keys generate cougr-testnet --network testnet
export COUGR_SIGNING_KEY=cougr-testnet
```

## Run the sample scene

A 3x3 tic-tac-toe sample scene is provided in `clients/godot/sample_scene.tscn`. It shows how to read the state and submit moves. 

```sh
cd clients/godot
godot --path .
```

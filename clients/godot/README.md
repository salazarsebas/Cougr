# Cougr checkpoint Godot client

Godot 4 addon for the Cougr `checkpoint` contract template. Real-time games
cannot put every frame on chain, so the template has a client simulate off chain
and commit a state hash and score at periodic checkpoints, with a dispute window
before a result is finalised. This addon is the Godot half of that: it reads the
match state and submits `commit_checkpoint` and `dispute_checkpoint` on Stellar
testnet.

It does not implement a scene editor, it does not talk to Freighter, and it does
not hold a key.

## Layout

| Path | What it is |
|---|---|
| `addons/cougr_checkpoint/checkpoint_client.gd` | RPC client: `read_state`, `commit_checkpoint`, `raise_dispute` |
| `addons/cougr_checkpoint/checkpoint_payload.gd` | Invocation and envelope byte layout |
| `addons/cougr_checkpoint/checkpoint_state.gd` | Decodes the `MatchState` a `get_state` returns |
| `addons/cougr_checkpoint/soroban_xdr.gd` | XDR, strkey, and hashing primitives |
| `addons/cougr_checkpoint/stellar_cli_signer.gd` | Signs through `stellar tx sign` |
| `addons/cougr_checkpoint/tests/run_tests.gd` | Headless vector check |
| `checkpoint_vectors/` | Rust `soroban_sdk` rebuild of the same payloads |
| `sample_checkpoint_scene.tscn` | Sample scene: commit a checkpoint, then dispute it |

## Import the addon

Copy `addons/cougr_checkpoint` into your project's `addons` directory, then enable
**Cougr Checkpoint** in Project Settings > Plugins. The addon adds no editor UI.

The client is a `Node`, because it owns an `HTTPRequest`. Add it to the scene that
drives the match:

```gdscript
const Client := preload("res://addons/cougr_checkpoint/checkpoint_client.gd")

func _ready() -> void:
    var client := Client.new()
    add_child(client)
    var state: Dictionary = await client.read_state()
    if state["success"]:
        print(state["state"]["last_tick"], state["state"]["last_score"])
```

Every method is a coroutine, so callers have to `await` them.

## Configure

The client reads the environment in `_ready()`, so nothing has to be committed to
point a build at a contract. Exported properties on the node win over the
environment when they are set.

| Variable | Meaning |
|---|---|
| `COUGR_RPC_URL` | Soroban RPC endpoint. Defaults to `https://soroban-testnet.stellar.org` |
| `COUGR_CONTRACT_ID` | The deployed contract, in `C...` form |
| `COUGR_SOURCE_ACCOUNT` | The account that signs, in `G...` form |
| `COUGR_SIGNING_KEY` | Stellar CLI identity or testnet secret key. Read by the signer, never by the client |
| `COUGR_NETWORK_PASSPHRASE` | Overrides the passphrase the signing payload is bound to |

`read_state` needs only `COUGR_RPC_URL` and `COUGR_CONTRACT_ID`. It simulates
`get_state`, which needs no signature, so it works before any wallet is set up.
A write needs all four of the first four.

```sh
export COUGR_RPC_URL=https://soroban-testnet.stellar.org
export COUGR_CONTRACT_ID=C...
export COUGR_SOURCE_ACCOUNT=G...
export COUGR_SIGNING_KEY=cougr-testnet
```

## Provide a key without committing it

The addon never holds a key and never derives one. Godot 4 has no ed25519, and
its hashing contexts stop at SHA-256, so instead of reimplementing ed25519 the
signer hands the unsigned envelope to `stellar tx sign`, which already has the
identity used to deploy the contract:

```sh
stellar keys generate cougr-testnet --network testnet
stellar keys fund cougr-testnet --network testnet
export COUGR_SIGNING_KEY=cougr-testnet
```

`COUGR_SIGNING_KEY` may name an identity, as above, or hold a testnet secret key
starting with `S`. The value is expanded by the shell inside the signing command,
and `clients/godot/.gitignore` ignores `env.sh` and `*.key`, so the key never
enters a file that Godot reads. The Stellar CLI has to be installed and on
`PATH`.

To sign somewhere else, including a hardware wallet or a remote signer, assign
any object with a matching method:

```gdscript
client.signer = MySigner.new()  # sign(envelope: PackedByteArray, passphrase: String) -> PackedByteArray
```

## Run the sample scene

```sh
cd clients/godot
godot --path .
```

The scene creates the client, prints the configuration it found, and offers three
buttons: **Read state**, **Commit checkpoint**, and **Raise dispute**. Commit uses
tick `1024`, hash `0123456789abcdef`, and score `4200`, then advances the tick so
a second commit is still monotone. Dispute targets the tick that was just
committed, with a different hash, which is what the contract requires to accept
it. Each result is logged in the window and printed to the console.

## Point it at a testnet contract

Generate and deploy a checkpoint contract, then open a match with the account the
addon will sign as:

```sh
cougr new my-game --template checkpoint
cd my-game
cargo test
stellar contract build
stellar contract deploy \
  --wasm target/wasm32v1-none/release/my_game.wasm \
  --source-account cougr-testnet \
  --network testnet
```

The deploy prints the contract id. Use it for `COUGR_CONTRACT_ID`, and set
`COUGR_SOURCE_ACCOUNT` to the address of `cougr-testnet`. Then open the match
from that same account, because `commit_checkpoint` requires the configured
player and rejects anyone else:

```sh
stellar contract invoke \
  --id "$COUGR_CONTRACT_ID" \
  --source-account cougr-testnet \
  --network testnet \
  -- start_match --player "$COUGR_SOURCE_ACCOUNT"
```

## Contract methods used

The client calls exactly these methods:

- `get_state() -> MatchState`
- `commit_checkpoint(player, tick, state_hash, score) -> ActionResult`
- `dispute_checkpoint(challenger, tick, claimed_hash) -> ActionResult`

`commit_checkpoint` requires a tick strictly greater than the last committed one.
`dispute_checkpoint` requires the tick of the last committed checkpoint, a hash
different from the committed one, and a ledger inside the dispute window. The
client always sends the transaction from `COUGR_SOURCE_ACCOUNT`, so that account
is the `player` on a commit and the `challenger` on a dispute.

`ActionResult.message` is not decoded yet: rejections surface as the RPC's
simulation error, which names the failing host call. Decoding `message` into the
contract's own reason codes (`notplay`, `oldtick`, `toolate`, `nohash`) is not
part of this addon.

## Byte layout and vectors

The two invocations have to be byte-identical to what the contract expects, and
neither side can be trusted to check itself, so the layout is pinned twice: once
by a Rust build of the same invocation through `soroban_sdk`, and once by
GDScript. Each side asserts the same hex, so moving either one fails CI.

```sh
cd clients/godot/checkpoint_vectors
cargo test -- --nocapture
```

```sh
cd clients/godot
godot --headless --path . --script res://addons/cougr_checkpoint/tests/run_tests.gd
```

Both cover, for `commit_checkpoint` and for the dispute: the argument list, the
unsigned transaction envelope, and the signing payload the client authorises. The
Rust side also decodes the pinned addresses with `soroban_sdk` and checks they
agree with the local strkey decoder, and the GDScript side decodes a `MatchState`
that `soroban_sdk` built.

`cargo test` in the crate also serves as the record of what the vectors are, so
run it before changing any argument. `.github/workflows/godot-checkpoint-client.yml`
runs both halves for changes under `clients/godot`.

## Limits

- **Signing needs the Stellar CLI.** Godot cannot sign ed25519, so a deployment
  that signs on the device would need a GDExtension, not this addon.
- **No unsigned 64-bit integers in GDScript.** `state_hash` and `claimed_hash` are
  accepted as an `int` below 2^63, or as a 16 character hex string for the full
  range, which is what the dispute test vector uses. `read_state` reports
  `last_hash` as 16 hex characters for the same reason.
- **One match per contract instance**, which is the template's design.
- **The submit path is not covered by CI.** The vector check is offline, and the
  RPC path needs a deployed contract and a funded key. The account sequence read,
  the request and response shapes, and the envelope layout were checked against
  Stellar testnet by hand, but no automated test submits a transaction.

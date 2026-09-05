# Murdoku

Murdoku is a murder mystery logic puzzle built with the [Cougr](../../README.md) ECS framework on Stellar Soroban. It demonstrates how to combine Entity Component System (ECS) game state, Pollar social logins, session key authorization, and zero-knowledge proof verification for privacy-preserving puzzle solving.

## Purpose and Pattern

Murdoku is an on-chain logic puzzle that combines Sudoku-style Latin square constraints with murder mystery storytelling. Players place suspects (e.g., characters, weapons, rooms) on a grid while satisfying constraints provided by clues (e.g., "The suspect with the knife is not in the same row as the butler"). 

This example showcases the canonical reference architecture for a full-stack game within the Cougr ecosystem:
* **Entity Component System (ECS)**: Separation of game state, input validation, puzzle constraint checks, and win conditions using the Cougr ECS engine.
* **Social Authentication & Session Keys**: Account abstraction utilizing Pollar for embedded wallet logins and scoped session key creation (`SessionBuilder`) to support non-intrusive gameplay transactions.
* **ZK Privacy Layer**: The solution is never stored in plaintext on-chain. Instead, puzzle creators submit a cryptographic commitment (Poseidon2 hash of the solution + salt) and a Groth16 verifier key. Players prove they know a valid solution by submitting a Groth16 zero-knowledge proof, without revealing the solution itself.

The contract supports two build modes controlled by the `zk` Cargo feature:
- **Default (v1)**: Plaintext solution stored on-chain. Simpler but exposes the solution to on-chain observers.
- **`zk` feature enabled**: Solution commitment + Groth16 proof verification. The solution remains private; the contract only stores a hash and validates ZK proofs.

---

## Architecture Overview

Murdoku follows a decoupled design where all gameplay rules and state changes are handled by the Soroban contract, while the user interface and wallet management are facilitated by a React web app.

```
Browser (Vite + React)
  └── Pollar (@pollar/react) - embedded wallet, social login
  └── contract.ts - Stellar SDK contract client
        │
        ▼
  Soroban Contract (examples/murdoku/src/)
  ├── lib.rs - entrypoints & GameApp wiring
  ├── components.rs - ECS components (Board, Suspect, GameState)
  ├── systems.rs - validation, move execution, and completion systems
  ├── types.rs - domain types (Clues, Suspects, GridConfiguration)
  └── auth.rs - session keys, CougrAccount wiring
        │
        ▼
  Stellar Testnet (Soroban RPC)
```

The React frontend embeds the Pollar SDK for OAuth/social logins. Pollar creates a temporary keypair representing a session key. The frontend uses the session key to sign game moves (e.g., `place_suspect`). The Soroban contract verifies that the session key is valid and authorized to act on behalf of the player using the `authorize_with_fallback` module.

---

## Public Contract API

Below is the entrypoint surface defined in the `#[contractimpl]` block of the Murdoku smart contract. Some entrypoints are only available with the `zk` feature enabled.

| Function | Parameters | Return Type | Description |
|---|---|---|---|
| `init_game` | `env: Env`, `admin: Address` | `()` | Initializes the contract state and registers the game administrator. |
| `submit_puzzle` (v1) | `env: Env`, `creator: Address`, `size: u32`, `solution: Vec<u32>`, `clues: Vec<Clue>` | `u32` | Registers a new puzzle in the catalog with plaintext solution. Validates Latin square constraints. |
| `submit_puzzle` (ZK) | `env: Env`, `creator: Address`, `grid_size: u32`, `suspects: Vec<String>`, `clues: Vec<Clue>`, `solution_commitment: BytesN<32>`, `verifier_key: Bytes`, `metadata: PuzzleMetadata` | `u32` | Registers a new puzzle with a solution commitment and Groth16 verifier key. No plaintext solution is stored. |
| `list_puzzles` | `env: Env`, `offset: u32`, `limit: u32` | `Vec<PuzzleSummary>` | Returns a paginated list of summaries of all registered puzzles. |
| `get_puzzle` | `env: Env`, `puzzle_id: u32` | `Puzzle` | Retrieves puzzle details. In ZK mode, returns the solution commitment (not the plaintext solution). |
| `deactivate_puzzle` | `env: Env`, `caller: Address`, `puzzle_id: u32` | `()` | Deactivates a puzzle. Only the creator can deactivate. |
| `submit_proof` (ZK) | `env: Env`, `player: Address`, `puzzle_id: u32`, `proof: Bytes`, `public_inputs: Vec<BytesN<32>>` | `()` | Submits a Groth16 proof of a valid solution. Marks the puzzle as solved for the player on success. |
| `is_solved` (ZK) | `env: Env`, `puzzle_id: u32`, `player: Address` | `bool` | Returns `true` if the player has submitted a valid proof for the puzzle. |

---

## Storage Model

Murdoku uses Soroban's state storage model (Instance, Persistent, and Temporary) to optimize gas costs and storage lifetimes.

| Storage Type | Data Kept | Lifetime | Rationale |
|---|---|---|---|---|
| **Persistent** | Registered Puzzle catalog (by ID), Creator profiles, Passkey credentials, Verifier keys (ZK), Solver state (ZK) | Indefinite | Puzzle templates and user credentials must persist forever across sessions and are read-heavy. |
| **Instance** | Active Player Game session, ECS World state (`SimpleWorld`), Admin configs | Extended (Renewed on play) | The active board state must persist during active play but can be garbage-collected or archived if the player abandons the game. |
| **Temporary** | Active Session Key tokens, cryptographic challenges | Short-term (Expires in blocks) | Session authorization keys only need to last for the duration of the play session and should expire automatically to free state space. |

In ZK mode, the following additional storage keys are used:

| Key Pattern | Type | Description |
|---|---|---|
| `(Symbol("VKEY"), puzzle_id)` | `Bytes` | XDR-serialized Groth16 `VerificationKey` |
| `(Symbol("SOLVED"), puzzle_id, player)` | `bool` | Whether a player has solved this puzzle |
| `(Symbol("SOLVER_CNT"), puzzle_id)` | `u32` | Number of unique solvers for this puzzle |

---

## Main Gameplay Flow

### Play Flow
1. **Connect Wallet**: The player logs into the frontend via Pollar using an email or social login. Pollar provisions a Stellar account and prepares a local session key.
2. **Browse Catalog**: The frontend calls `list_puzzles` to fetch and render the list of available murder mystery puzzles.
3. **Open Puzzle**: The player selects a puzzle. The frontend calls `get_puzzle` to retrieve the clues and grid dimensions, then calls `start_game` to initialize the ECS world state for this puzzle.
4. **Place Suspects**: The player drags and drops suspects (e.g., characters, weapons) onto grid cells. Each placement calls `place_suspect`. This transaction is signed by the local session key and bypasses manual wallet confirmations.
5. **On-chain Validation**: The contract executes the validation system:
   - Verifies the cell is editable.
   - Enforces Latin square constraints (no duplicate suspects in the same row or column).
   - Validates placement against the puzzle's clues.
6. **Completion**: If all cells are filled correctly, the contract marks the game status as solved. `is_solved` returns `true`, and the frontend displays a victory screen.

### Create Flow
1. **Connect Wallet**: The creator authenticates on the creator portal via Pollar.
2. **Configure Puzzle**: The creator designs a puzzle:
   - Selects grid size (4x4 or 5x5).
   - Enters the full solution grid.
   - Defines the clues (e.g., cell constraints, adjacency rules).
3. **Submit Puzzle** (v1): The creator clicks "Publish", triggering a call to `submit_puzzle` with the plaintext solution.
4. **Submit Puzzle** (ZK): The creator generates a Poseidon2 commitment of the solution and a Groth16 verifier key, then calls `submit_puzzle` with the commitment and key instead of the plaintext.
5. **Contract Validation**:
   - **v1 mode**: The contract validates the Latin square constraints and clue consistency via ECS systems.
   - **ZK mode**: The contract validates puzzle structure (grid size, suspects, clue bounds) but cannot verify solution correctness - the ZK proof catches cheating at solve time.
6. **Publishing**: If checks pass, the contract assigns a puzzle ID, stores the puzzle in persistent storage, and activates it.

---

## Cougr APIs Used

Murdoku is built on the core APIs of the Cougr framework.

| API / Module | Used For | Location |
|---|---|---|
| `GameApp` | Orchestrates systems execution flow during suspect placement and puzzle registration. | `src/lib.rs` |
| `ScheduleStage` | Ensures that validation systems execute strictly before update systems, and completion systems execute last. | `src/lib.rs` |
| `SimpleWorld` | Stores the active ECS state including entity components (e.g. Board, MoveCount). | `src/lib.rs`, `src/systems.rs` |
| `impl_component!` | Macros defining serialized game components like `BoardComponent` and `GameStatusComponent`. | `src/components.rs` |
| `auth::SessionBuilder` | Builds scoped session keys containing specific permissions (e.g., only allowing `place_suspect`). | `src/auth.rs` |
| `auth::authorize_with_fallback` | Authorizes transactions using either the active session key or the player's direct signature. | `src/auth.rs` |
| `ops::Ownable` | Restricts contract initialization and admin parameters to the contract owner. | `src/lib.rs` |

---

## Frontend Setup

### 1. Requirements
* Node.js v18 or later
* Stellar CLI installed and configured

### 2. Run Locally
Navigate to the frontend directory:
```bash
cd examples/murdoku/frontend
```

Copy the environment template:
```bash
cp .env.example .env
```

Open `.env` and set the variables:
* `VITE_POLLAR_API_KEY`: Obtain this from the Pollar Developer Console by registering an application.
* `VITE_CONTRACT_ID`: The deployed contract ID from your testnet deployment (see below).

Install dependencies and run the Vite development server:
```bash
npm install
npm run dev
```

The frontend will run locally on `http://localhost:5173`.

---

## Contract Build and Test Commands

Execute the following commands from the `examples/murdoku` directory to format, lint, test, and build the contract.

```bash
cd examples/murdoku
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features zk   # run ZK-specific tests
stellar contract build
```

To build with the ZK privacy layer enabled:
```bash
cargo build --features zk
cargo test --features zk
```

---

## Deploying to Testnet

To deploy and test the contract manually on Stellar Testnet, use the following commands:

```bash
# 1. Generate an identity for deployment
stellar keys generate murdoku_deployer

# 2. Fund the identity using Friendbot
stellar keys fund murdoku_deployer --network <NETWORK>

# 3. Deploy the compiled WASM to Testnet
CONTRACT_ID=$(stellar contract deploy \
  --wasm target/wasm32v1-none/release/murdoku.wasm \
  --network <NETWORK> \
  --source murdoku_deployer)

# 4. Initialize the contract game registry
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  --source murdoku_deployer \
  -- init_game \
  --admin murdoku_deployer

# 5. Submit a minimal 4x4 placeholder puzzle to the catalog
# (Solution grid contains flat representation of a 4x4 Latin square)
stellar contract invoke \
  --id $CONTRACT_ID \
  --network <NETWORK> \
  --source murdoku_deployer \
  -- submit_puzzle \
  --creator murdoku_deployer \
  --size 4 \
  --solution '[1,2,3,4,2,3,4,1,3,4,1,2,4,1,2,3]' \
  --clues '[]'
```

---

## ZK Privacy Layer

### Circuit Statement

The Groth16 circuit proves the following statement without revealing the solution:

> "I know a set of values `cells` such that:
> (1) `cells` is a valid Latin square of size N,
> (2) Poseidon2(cells || salt) == commitment,
> (3) each cell value is in range 1..=N."

### Public Inputs Format

| Index | Type | Description |
|-------|------|-------------|
| 0 | `BytesN<32>` | Solution commitment (Poseidon2 hash of solution + salt) |
| 1..N | `BytesN<32>` | Additional public inputs (e.g., grid size, clue hashes) |

### Player Solve Flow (ZK mode)

1. **Browse Puzzles**: Frontend calls `list_puzzles` and `get_puzzle` to fetch puzzle metadata (grid size, suspects, clues, solution **commitment** - not the plaintext solution).
2. **Solve Off-Chain**: The player solves the puzzle manually, arriving at a candidate grid arrangement.
3. **Generate Proof**: The frontend (or a backend prover service) generates a Groth16 proof that the player knows a valid arrangement consistent with the commitment.
4. **Submit Proof**: The player calls `submit_proof(player, puzzle_id, proof, public_inputs)`. The contract verifies the Groth16 proof against the stored verifier key.
5. **Mark Solved**: On valid proof, the contract marks the puzzle as solved for that player and increments the solver count.

### Key Files

| File | Purpose |
|------|---------|
| `src/zk.rs` | Groth16 proof verification logic, verifier key storage, solver state tracking |
| `src/components.rs` | `SolutionCommitment` ECS component (ZK mode only) |
| `src/lib.rs` | Feature-gated `Puzzle` struct, `submit_proof` entrypoint, `is_solved` query |

### Backward Compatibility

The v1 plaintext solution path is preserved behind the default (no `zk` feature) build. To switch between modes:

```bash
# v1 mode (default)
cargo build
cargo test

# ZK mode
cargo build --features zk
cargo test --features zk
```

### Known Constraints

- **Trusted Setup**: Groth16 requires a trusted setup ceremony for each circuit. The development verifier keys used in this implementation are **not production-safe**. A proper multi-party ceremony must be completed before mainnet deployment. See the module-level docs in `src/zk.rs` for details.
- **Client-Side Proof Generation**: Generating Groth16 proofs in the browser requires a WASM-compiled prover. This is tracked as a separate enhancement and is not yet implemented in the frontend.
- **Circuit Size**: A 5×5 Murdoku circuit has 25 cells. Each cell requires range checks (1..=N) and Latin square row/column uniqueness constraints. The constraint count should be verified against Stellar's Groth16 verification limits before production use.

---

## Known Limitations

- **ZK trusted setup is development-only.** The Groth16 verifier keys shipped here come from a development setup and are not production-safe. See [Known Constraints](#known-constraints) above for the full ZK caveats (trusted setup, client-side proving, circuit size).
- **Difficulty is unverified creator metadata.** The contract validates Latin-square correctness on submission but does not independently grade or rank difficulty - the `difficulty` field is taken from the creator at face value.
- **No ranked leaderboard or rewards.** A per-puzzle solver count is tracked, but the example intentionally omits scoring, ranked leaderboards, and reward distribution to keep the reference focused on the ECS + auth + ZK pattern.
- **Puzzle catalog grows without an on-chain cap.** Submissions increment `PUZZLE_COUNT` without an upper bound. Reads are paginated via `offset`/`limit`, but production deployments expecting very large catalogs should add submission limits and indexed lookups.

---

## Classification

Marked as **Canonical**. Murdoku is the reference full-stack example for Cougr, showcasing how to build games that combine ECS, session-key-based social logins (via Pollar), and multi-step transaction authorization. Use this as the template for production-grade full-stack game contracts.

# Build Your First Game

Welcome to Cougr! This tutorial will take you from an empty directory to a fully tested, testnet-deployed game. 

This guide is designed for developers who already know **Rust** but have no prior experience with **Cougr**, **Soroban**, or **Stellar**. By the end, you'll understand how Cougr's Entity-Component-System (ECS) architecture translates into safe, efficient smart contracts.

We are going to build a simple 2D grid game where a player can spawn into the world and walk in four directions.

---

## 1. Project Setup

Since we're building a smart contract, we start with a standard Rust library rather than a binary.

```bash
cargo new --lib my_first_game
cd my_first_game
```

Add `cougr-core` and the `soroban-sdk` to your `Cargo.toml`. We enable the `testutils` feature to get access to Cougr's built-in testing harness later.

```toml
[package]
name = "my_first_game"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
soroban-sdk = "25.3.2"
cougr-core = "1.1.0" # or the latest version from crates.io

[features]
testutils = ["soroban-sdk/testutils", "cougr-core/testutils"]
```

---

## 2. The Mental Model: ECS on Soroban

If you've used an ECS in a game engine like Bevy or Unity, you already know the basics: **Entities** are just IDs, **Components** hold data, and **Systems** run logic. 

However, building for a blockchain introduces new constraints you must consider:

> [!WARNING]
> **Soroban-Specific Constraints**
> 
> 1. **Storage is not a normal database:** You cannot freely iterate over millions of rows. State must be loaded into memory, modified, and saved back efficiently.
> 2. **Execution costs money:** Every instruction, memory allocation, and storage write costs "gas". Infinite loops or massive arrays will cause your transaction to exceed resource limits and fail.
> 3. **Instance Storage vs Persistent Storage:** Cougr uses Soroban's "Instance Storage" by default for your hot-loop game state. This means all active gameplay components are loaded in a single read, making operations extremely cheap and fast, but it requires you to be mindful of total state size.

Cougr abstracts the heavy lifting of storage management, but you still need to write code with these constraints in mind.

---

## 3. Defining Components

Let's open `src/lib.rs`. First, we clear out the default code and define our game's data. 

We need two components: a `Position` to track where the player is, and `Moves` to track how many steps they have left.

```rust,ignore
#![no_std]

use cougr_core::game::SorobanGame;
use cougr_core::{impl_component, impl_component_observed, impl_soroban_game};
use soroban_sdk::{contract, contractimpl, contracttype, Env};

// `Position` emits an indexed event on every change (so a UI can watch it)
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl_component_observed!(Position, "position", Table, { x: i32, y: i32 });

// `Moves` is kept private; it doesn't need to emit an event on every step
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct Moves {
    pub remaining: u32,
}
impl_component!(Moves, "moves", Table, { remaining: u32 });
```

Notice the two different macros:
- `impl_component_observed!` tells Cougr to emit a Soroban event every time this component is modified. This is crucial for off-chain clients (like your web frontend) to track movement in real-time without polling the blockchain.
- `impl_component!` is for standard data that doesn't need to be broadcasted to indexers, saving you gas on event emissions.

---

## 4. Writing the Game Contract (Systems)

Next, we define our contract. In Cougr, the contract acts as the outer shell that loads the ECS world, runs your system logic (the functions), and saves the world back.

Add this to the bottom of `src/lib.rs`:

```rust,ignore
#[contract]
#[derive(Clone)]
pub struct MyFirstGame;

// This macro wires up the `load_world` and `save_world` boilerplate.
impl_soroban_game!(MyFirstGame, "world");

#[contractimpl]
impl MyFirstGame {
    
    /// Spawns a new player entity into the world.
    pub fn spawn(env: Env) -> u32 {
        // 1. Load the ECS world from Soroban storage
        let mut world = MyFirstGame::load_world(&env);

        // 2. Spawn an entity and attach our components
        let entity = world.spawn_entity();
        world.set_typed_observed(&env, entity, &Position { x: 0, y: 0 });
        world.set_typed(&env, entity, &Moves { remaining: 10 });

        // 3. Save the ECS world back to Soroban storage
        MyFirstGame::save_world(&env, &world);
        
        entity
    }

    /// Moves a player entity in a given direction (0=North, 1=East, 2=South, 3=West).
    pub fn move_entity(env: Env, entity_id: u32, direction: u32) {
        let mut world = MyFirstGame::load_world(&env);

        let mut pos = world.get_typed::<Position>(&env, entity_id).unwrap();
        let mut moves = world.get_typed::<Moves>(&env, entity_id).unwrap();

        if moves.remaining == 0 {
            panic!("no moves left");
        }

        match direction {
            0 => pos.y += 1, // North
            1 => pos.x += 1, // East
            2 => pos.y -= 1, // South
            3 => pos.x -= 1, // West
            _ => panic!("invalid direction"),
        }
        
        moves.remaining -= 1;

        // Apply changes
        world.set_typed_observed(&env, entity_id, &pos);
        world.set_typed(&env, entity_id, &moves);

        MyFirstGame::save_world(&env, &world);
    }
    
    /// Query a player's current position.
    pub fn get_position(env: Env, entity_id: u32) -> Option<Position> {
        let world = MyFirstGame::load_world(&env);
        world.get_typed::<Position>(&env, entity_id)
    }
}
```

This pattern - **Load World -> Query/Modify -> Save World** - is the backbone of every Cougr contract entry point.

---

## 5. Local Testing

Testing smart contracts on an actual network is slow. Cougr provides a powerful local `GameHarness` to run tests instantly in memory.

Create a new file `src/test.rs` and add it to your module tree by adding `#[cfg(test)] mod test;` to the very bottom of `src/lib.rs`.

In `src/test.rs`:

```rust,ignore
#![cfg(test)]

use super::*;
use cougr_core::test::{GameHarness, Scenario};
use soroban_sdk::Env;

#[test]
fn test_spawn_and_move() {
    let env = Env::default();
    
    // Register our contract using Cougr's test harness
    let harness = GameHarness::new(env, MyFirstGame);

    // The macro generated a "MyFirstGameClient" for us automatically
    let client = MyFirstGameClient::new(harness.env(), harness.contract_id());
    
    // 1. Spawn the entity
    let entity_id = client.spawn();
    
    let pos = client.get_position(&entity_id).unwrap();
    assert_eq!(pos.x, 0);
    assert_eq!(pos.y, 0);

    // 2. Use Cougr's Scenario builder to simulate turns/moves
    Scenario::new("move north")
        .turns(1)
        .run(&harness, |_player, _turn, h| {
            let c = MyFirstGameClient::new(h.env(), h.contract_id());
            
            // Move North (direction = 0)
            c.move_entity(&entity_id, &0);
            
            let pos = c.get_position(&entity_id).unwrap();
            assert_eq!(pos.x, 0);
            assert_eq!(pos.y, 1);
        });
}
```

Run your tests to verify your game works locally:

```bash
cargo test
```

If it passes, you are ready to deploy!

---

## 6. Deploying to Testnet

To deploy, we need to compile our game to a WebAssembly (WASM) binary and use the Stellar CLI.

> [!TIP]
> If you haven't installed the Stellar CLI yet, check out the [Stellar Quickstart](https://developers.stellar.org/docs/build/smart-contracts/getting-started/deploy-to-testnet).

**1. Build the WASM file:**
```bash
cargo build --target wasm32-unknown-unknown --release
```
Your compiled game is now located at `target/wasm32-unknown-unknown/release/my_first_game.wasm`.

**2. Configure your testnet identity:**
```bash
stellar keys generate alice --network testnet
```

**3. Deploy the contract:**
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/my_first_game.wasm \
  --source alice \
  --network testnet
```

If successful, the CLI will return a `C...` contract address. **Congratulations!** Your game is live on the Stellar Testnet.

---

## 7. Next Steps

You've built and deployed a basic ECS contract. However, real games require more advanced mechanics like access control, hidden information (Fog of War), or multi-contract plugin architectures.

- **Learn the architecture:** Read [Cougr Patterns](../learn/PATTERNS.md) to understand how to structure larger, production-ready games.
- **Get inspired:** Browse the **Showcase** (like `murdoku` or `battleship`) in the `examples/` directory to see full-stack implementations.

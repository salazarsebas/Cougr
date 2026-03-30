# RPG Arena

A standalone Soroban smart contract example demonstrating a compact turn-based combat loop using the Cougr-Core ECS framework.

## Overview

The RPG Arena example focuses on deterministic turn management, status effect application, and cooldown tracking. It allows two players (Stellar addresses) to engage in a structured battle until one is defeated.

### Key Features

- **Turn System**: Round-based movement with enforcement of turn order.
- **Combat Stats**: Combatants have HP, Defense, and Speed.
- **Actions**:
    - **Attack**: Deals base damage reduced by opponent's defense.
    - **Defent**: Reduces damage taken on the next turn.
    - **Special Ability**: A powerful "Poison Strike" that deals moderate damage and applies a damage-over-time (Poison) status effect.
- **Status Effects**: Supports active status effects (e.g., Poison) that tick at the start of each combatant's turn.
- **Cooldowns**: Special abilities have a turn-based cooldown to prevent repeated use.
- **ECS Design**: Decouples logic into reusable systems:
    - `ActionValidationSystem`: Ensures actions are legal for the current turn.
    - `DamageResolutionSystem`: Calculates and applies damage with modifiers.
    - `StatusEffectSystem`: Updates active effects and applies tick damage.
    - `CooldownSystem`: Decrements cooldown counters.
    - `TurnAdvanceSystem`: Transitions turns and rounds.
    - `EndConditionSystem`: Detects win/loss states.

## Commands

```bash
cd examples/rpg_arena
# Format and lint
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test

# Build contract
stellar contract build
```

## Contract API

- `init_battle(player_one: Address, player_two: Address)`: Initializes a new battle between two players.
- `submit_action(player: Address, action: Action)`: Performs one of the three actions: `Attack`, `Defend`, or `Special`.
- `get_state() -> BattleState`: Returns the current overall battle state.
- `get_combatant(player: Address) -> CombatantState`: Returns the state of a specific combatant.
- `is_finished() -> bool`: Returns true if the battle has concluded.

## Implementation Details

The contract uses the Cougr core components and macros to efficiently store and serialize the game world state. By leveraging ECS patterns, it separates the "what" (data in components) from the "how" (logic in systems), making it highly testable and extensible.

### Example Battle State
```rust
pub struct BattleState {
    pub p1: CombatantState,
    pub p2: CombatantState,
    pub round: u32,
    pub current_turn: Address,
    pub is_finished: bool,
    pub winner: Option<Address>,
}
```

use cougr_core::{impl_component, impl_rich_component};
use soroban_sdk::{contracttype, Address, Env, Vec};

/// Entity id under which the entire game state is stored. Tic-tac-toe has
/// exactly one game per contract instance, so a single fixed entity id is
/// sufficient instead of a dynamic entity population.
pub const GAME_ENTITY: u32 = 1;

// ─── Components ───────────────────────────────────────────────────────────────

/// Board state: 9 cells where 0 = empty, 1 = X, 2 = O.
/// Uses impl_rich_component! because Vec<u32> requires XDR codec.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Board {
    pub cells: Vec<u32>,
}

impl_rich_component!(Board, "board");

impl Board {
    pub fn new(env: &Env) -> Self {
        let mut cells = Vec::new(env);
        for _ in 0..9u32 {
            cells.push_back(0u32);
        }
        Self { cells }
    }
}

/// Both players' wallet addresses.
/// Uses impl_rich_component! because Address requires XDR codec.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Players {
    pub player_x: Address,
    pub player_o: Address,
}

impl_rich_component!(Players, "players");

/// Turn and game-over state — plain fixed-size fields; no XDR needed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnState {
    pub is_x_turn: bool,
    pub move_count: u32,
    pub status: u32, // 0 = in progress, 1 = X wins, 2 = O wins, 3 = draw
}

impl_component!(TurnState, "turnst", Table, {
    is_x_turn: bool,
    move_count: u32,
    status: u32
});

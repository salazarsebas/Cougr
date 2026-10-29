#![no_std]

use cougr_core::component::ComponentTrait;
use cougr_core::component::Position as CorePosition;
use cougr_core::event::Event;
use soroban_sdk::{contract, contractimpl, Env, Vec};

pub mod components;
pub mod maze;
pub mod systems;
pub mod types;

#[cfg(test)]
mod test;

use crate::components::{Ghost, Position};
use crate::maze::create_maze;
use crate::systems::GameSystem;
use crate::types::{
    CellType, DataKey, Direction, GameState, GHOST_ENTITY_ID_START, INITIAL_LIVES, PELLET_POINTS,
    POWER_PELLET_POINTS,
};

#[contract]
pub struct PacManContract;

#[contractimpl]
impl PacManContract {
    pub fn init_game(env: Env) -> GameState {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Game already initialized");
        }

        let maze = create_maze(&env);
        let mut pellet_count: u32 = 0;
        for i in 0..maze.len() {
            let cell = maze.get(i).unwrap();
            if cell == CellType::Pellet || cell == CellType::PowerPellet {
                pellet_count += 1;
            }
        }

        let mut ghosts: Vec<Ghost> = Vec::new(&env);
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START, 4, 4));
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 1, 5, 4));
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 2, 4, 5));
        ghosts.push_back(Ghost::new(GHOST_ENTITY_ID_START + 3, 5, 5));

        let pacman_start = Position::new(1, 1);
        let collision_events: Vec<Event> = Vec::new(&env);

        let state = GameState {
            pacman_pos: pacman_start,
            pacman_dir: Direction::Right,
            pacman_start,
            ghosts,
            maze,
            score: 0,
            lives: INITIAL_LIVES,
            game_over: false,
            won: false,
            power_mode_timer: 0,
            pellets_remaining: pellet_count,
            last_collision_events: collision_events,
        };

        env.storage().instance().set(&DataKey::GameState, &state);
        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().extend_ttl(50000, 100000);

        state
    }

    pub fn change_direction(env: Env, direction: Direction) {
        let mut state = Self::get_state(&env);
        if state.game_over {
            panic!("Game is over");
        }
        state.pacman_dir = direction;
        env.storage().instance().set(&DataKey::GameState, &state);
    }

    pub fn update_tick(env: Env) -> GameState {
        let mut state = Self::get_state(&env);
        if state.game_over {
            panic!("Game is over");
        }

        GameSystem::move_pacman(&env, &mut state);
        GameSystem::check_pellet_collection(&env, &mut state);
        GameSystem::move_ghosts(&env, &mut state);
        GameSystem::check_ghost_collisions(&env, &mut state);

        if state.power_mode_timer > 0 {
            state.power_mode_timer -= 1;
            if state.power_mode_timer == 0 {
                GameSystem::end_frightened_mode(&env, &mut state);
            }
        }

        if state.pellets_remaining == 0 {
            state.game_over = true;
            state.won = true;
        }

        env.storage().instance().set(&DataKey::GameState, &state);
        state
    }

    pub fn eat_pellet(env: Env) -> u32 {
        let mut state = Self::get_state(&env);
        if state.game_over {
            return 0;
        }

        let idx = state.pacman_pos.to_index();
        let cell = state.maze.get(idx).unwrap();

        let points = match cell {
            CellType::Pellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += PELLET_POINTS;
                state.pellets_remaining -= 1;
                PELLET_POINTS
            }
            CellType::PowerPellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += POWER_PELLET_POINTS;
                state.pellets_remaining -= 1;
                GameSystem::activate_power_mode(&env, &mut state);
                POWER_PELLET_POINTS
            }
            _ => 0,
        };

        if points > 0 {
            env.storage().instance().set(&DataKey::GameState, &state);
        }
        points
    }

    pub fn get_score(env: Env) -> u32 {
        Self::get_state(&env).score
    }

    pub fn get_lives(env: Env) -> u32 {
        Self::get_state(&env).lives
    }

    pub fn get_pacman_position(env: Env) -> Position {
        Self::get_state(&env).pacman_pos
    }

    pub fn get_maze(env: Env) -> Vec<CellType> {
        Self::get_state(&env).maze
    }

    pub fn get_game_state(env: Env) -> GameState {
        Self::get_state(&env)
    }

    pub fn check_game_over(env: Env) -> (bool, bool) {
        let state = Self::get_state(&env);
        (state.game_over, state.won)
    }

    pub fn get_collision_events(env: Env) -> Vec<Event> {
        Self::get_state(&env).last_collision_events
    }

    pub fn get_pacman_core_position(env: Env) -> CorePosition {
        let state = Self::get_state(&env);
        state.pacman_pos.to_core_position()
    }

    pub fn get_serialized_pacman_position(env: Env) -> soroban_sdk::Bytes {
        let state = Self::get_state(&env);
        let core_pos = state.pacman_pos.to_core_position();
        core_pos.serialize(&env)
    }

    fn get_state(env: &Env) -> GameState {
        env.storage()
            .instance()
            .get(&DataKey::GameState)
            .expect("Game not initialized")
    }
}

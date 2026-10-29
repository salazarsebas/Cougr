#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;

use components::{
    BoardState, ECSWorldState, GameState, GameStatusComponent, ScoreState, TurnComponent,
    WINNER_DRAW, WINNER_NONE, WORLD_KEY,
};
use components::{BLACK, STATUS_ACTIVE, STATUS_FINISHED, WHITE};
use systems::{
    end_condition_system, flip_resolution_system, init_board, move_validation_system, opponent_of,
    scoring_system, turn_system,
};

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct ReversiContract;

#[contractimpl]
impl ReversiContract {
    pub fn init_game(env: Env, player_one: Address, player_two: Address) {
        if env
            .storage()
            .instance()
            .get::<_, ECSWorldState>(&WORLD_KEY)
            .is_some()
        {
            panic!("Game already initialized");
        }
        let board = init_board(&env);
        let score = scoring_system(&board);
        let world = ECSWorldState {
            board,
            turn: TurnComponent {
                current_player: BLACK,
                pass_count: 0,
            },
            status: GameStatusComponent {
                status: STATUS_ACTIVE,
            },
            score,
            player_one,
            player_two,
        };
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn submit_move(env: Env, player: Address, row: u32, col: u32) {
        player.require_auth();
        let mut world = Self::load_world(&env);
        if world.status.status != STATUS_ACTIVE {
            panic!("Game is finished");
        }
        let player_color = Self::player_color(&world, &player);
        if player_color != world.turn.current_player {
            panic!("Not your turn");
        }
        if !move_validation_system(&world.board, row, col, player_color) {
            panic!("Illegal move");
        }
        let opp = opponent_of(player_color);
        world.board = flip_resolution_system(world.board, row, col, player_color);
        world.score = scoring_system(&world.board);
        world.turn = turn_system(&world.board, player_color, opp);
        world.status = end_condition_system(&world.board, &world.turn);
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn get_state(env: Env) -> GameState {
        let world = Self::load_world(&env);
        GameState {
            current_player: world.turn.current_player,
            pass_count: world.turn.pass_count,
            status: world.status.status,
        }
    }

    pub fn get_board(env: Env) -> BoardState {
        let world = Self::load_world(&env);
        BoardState {
            cells: world.board.cells,
            width: world.board.width,
            height: world.board.height,
        }
    }

    pub fn get_score(env: Env) -> ScoreState {
        let world = Self::load_world(&env);
        let winner = if world.status.status == STATUS_FINISHED {
            if world.score.black_count > world.score.white_count {
                BLACK
            } else if world.score.white_count > world.score.black_count {
                WHITE
            } else {
                WINNER_DRAW
            }
        } else {
            WINNER_NONE
        };
        ScoreState {
            black_count: world.score.black_count,
            white_count: world.score.white_count,
            winner,
        }
    }

    fn load_world(env: &Env) -> ECSWorldState {
        env.storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"))
    }

    fn player_color(world: &ECSWorldState, player: &Address) -> u32 {
        if player == &world.player_one {
            BLACK
        } else if player == &world.player_two {
            WHITE
        } else {
            panic!("Unknown player")
        }
    }
}

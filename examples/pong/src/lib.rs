#![no_std]

mod components;
mod systems;

pub use components::{BallComponent, ECSWorldState, GameState, PaddleComponent, ScoreComponent};
use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};
#[cfg(test)]
pub(crate) use systems::{
    BALL_SPEED, FIELD_HEIGHT, FIELD_WIDTH, PADDLE_HEIGHT, PADDLE_SPEED, WINNING_SCORE,
};

const ECS_WORLD_KEY: Symbol = symbol_short!("ECSWORLD");

#[contract]
pub struct PongContract;

#[contractimpl]
impl PongContract {
    /// Initialize a new game using Cougr-Core ECS component pattern
    /// Demonstrates: Entity creation with components
    pub fn init_game(env: Env) -> GameState {
        // Create ECS world with entities and components
        // Following Cougr-Core pattern: each game object is an entity with components
        let world_state = ECSWorldState {
            // Entity 0: Player 1 Paddle
            player1_paddle: PaddleComponent {
                player_id: 1,
                y_position: systems::FIELD_HEIGHT / 2,
            },
            // Entity 1: Player 2 Paddle
            player2_paddle: PaddleComponent {
                player_id: 2,
                y_position: systems::FIELD_HEIGHT / 2,
            },
            // Entity 2: Ball
            ball: BallComponent {
                x: systems::FIELD_WIDTH / 2,
                y: systems::FIELD_HEIGHT / 2,
                vx: systems::BALL_SPEED,
                vy: systems::BALL_SPEED,
            },
            // Entity 3: Score
            score: ScoreComponent {
                player1_score: 0,
                player2_score: 0,
                game_active: true,
            },
        };

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        systems::world_to_game_state(&world_state)
    }

    /// Move a player's paddle
    /// Demonstrates: Component query and update pattern from Cougr-Core
    pub fn move_paddle(env: Env, player: u32, direction: i32) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !world_state.score.game_active {
            return systems::world_to_game_state(&world_state);
        }

        // Query pattern: Find paddle component by player_id
        // This demonstrates Cougr-Core's component query approach
        let movement = direction * systems::PADDLE_SPEED;

        if world_state.player1_paddle.player_id == player {
            // Update component
            let new_y = world_state.player1_paddle.y_position + movement;
            world_state.player1_paddle.y_position = systems::clamp_paddle_position(new_y);
        } else if world_state.player2_paddle.player_id == player {
            // Update component
            let new_y = world_state.player2_paddle.y_position + movement;
            world_state.player2_paddle.y_position = systems::clamp_paddle_position(new_y);
        }

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        systems::world_to_game_state(&world_state)
    }

    /// Update game tick - demonstrates Cougr-Core System pattern
    /// Systems: PhysicsSystem, CollisionSystem, ScoringSystem
    pub fn update_tick(env: Env) -> GameState {
        let mut world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !world_state.score.game_active {
            return systems::world_to_game_state(&world_state);
        }

        // PhysicsSystem: Update ball position based on velocity
        systems::physics_system(&mut world_state);

        // CollisionSystem: Handle wall and paddle collisions
        systems::collision_system(&mut world_state);

        // ScoringSystem: Check for scoring and update scores
        systems::scoring_system(&mut world_state);

        env.storage().instance().set(&ECS_WORLD_KEY, &world_state);
        systems::world_to_game_state(&world_state)
    }

    /// Get current game state
    pub fn get_game_state(env: Env) -> GameState {
        let world_state: ECSWorldState = env
            .storage()
            .instance()
            .get(&ECS_WORLD_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        systems::world_to_game_state(&world_state)
    }

    /// Reset the game
    pub fn reset_game(env: Env) -> GameState {
        Self::init_game(env)
    }
}

#[cfg(test)]
mod test;

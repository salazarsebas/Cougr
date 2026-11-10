use crate::components::{Ghost, Position};
use crate::types::{
    CellType, Direction, GameState, GhostMode, GHOST_POINTS, MAZE_HEIGHT, MAZE_WIDTH,
    PELLET_POINTS, POWER_MODE_DURATION, POWER_PELLET_POINTS,
};
use cougr_core::event::EventTrait;
use cougr_core::event::{CollisionEvent, Event};
use soroban_sdk::{Env, Vec};

pub struct GameSystem;

impl GameSystem {
    pub fn move_pacman(_env: &Env, state: &mut GameState) {
        let mut new_pos = state.pacman_pos;

        match state.pacman_dir {
            Direction::Up => new_pos.y -= 1,
            Direction::Down => new_pos.y += 1,
            Direction::Left => new_pos.x -= 1,
            Direction::Right => new_pos.x += 1,
        }

        if new_pos.x < 0 {
            new_pos.x = (MAZE_WIDTH - 1) as i32;
        } else if new_pos.x >= MAZE_WIDTH as i32 {
            new_pos.x = 0;
        }
        if new_pos.y < 0 {
            new_pos.y = (MAZE_HEIGHT - 1) as i32;
        } else if new_pos.y >= MAZE_HEIGHT as i32 {
            new_pos.y = 0;
        }

        let idx = new_pos.to_index();
        let cell = state.maze.get(idx).unwrap();

        if cell != CellType::Wall {
            state.pacman_pos = new_pos;
        }
    }

    pub fn check_pellet_collection(env: &Env, state: &mut GameState) {
        let idx = state.pacman_pos.to_index();
        let cell = state.maze.get(idx).unwrap();

        match cell {
            CellType::Pellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += PELLET_POINTS;
                state.pellets_remaining -= 1;
            }
            CellType::PowerPellet => {
                state.maze.set(idx, CellType::Empty);
                state.score += POWER_PELLET_POINTS;
                state.pellets_remaining -= 1;
                Self::activate_power_mode(env, state);
            }
            _ => {}
        }
    }

    pub fn move_ghosts(_env: &Env, state: &mut GameState) {
        let pacman_pos = state.pacman_pos;

        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();

            if ghost.frightened_timer > 0 {
                ghost.frightened_timer -= 1;
                if ghost.frightened_timer == 0 {
                    ghost.mode = GhostMode::Chase;
                }
            }

            let new_dir = Self::calculate_ghost_direction(state, &ghost, pacman_pos);
            ghost.direction = new_dir;

            let mut new_pos = ghost.position;
            match ghost.direction {
                Direction::Up => new_pos.y -= 1,
                Direction::Down => new_pos.y += 1,
                Direction::Left => new_pos.x -= 1,
                Direction::Right => new_pos.x += 1,
            }

            if new_pos.x < 0 {
                new_pos.x = (MAZE_WIDTH - 1) as i32;
            } else if new_pos.x >= MAZE_WIDTH as i32 {
                new_pos.x = 0;
            }
            if new_pos.y < 0 {
                new_pos.y = (MAZE_HEIGHT - 1) as i32;
            } else if new_pos.y >= MAZE_HEIGHT as i32 {
                new_pos.y = 0;
            }

            let idx = new_pos.to_index();
            let cell = state.maze.get(idx).unwrap();

            if cell != CellType::Wall {
                ghost.position = new_pos;
            }

            state.ghosts.set(i, ghost);
        }
    }

    fn calculate_ghost_direction(
        state: &GameState,
        ghost: &Ghost,
        pacman_pos: Position,
    ) -> Direction {
        let directions = [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut best_dir = ghost.direction;
        let mut best_score: i32 = i32::MIN;

        for dir in directions.iter() {
            let mut test_pos = ghost.position;
            match dir {
                Direction::Up => test_pos.y -= 1,
                Direction::Down => test_pos.y += 1,
                Direction::Left => test_pos.x -= 1,
                Direction::Right => test_pos.x += 1,
            }

            if test_pos.x < 0 {
                test_pos.x = (MAZE_WIDTH - 1) as i32;
            } else if test_pos.x >= MAZE_WIDTH as i32 {
                test_pos.x = 0;
            }
            if test_pos.y < 0 {
                test_pos.y = (MAZE_HEIGHT - 1) as i32;
            } else if test_pos.y >= MAZE_HEIGHT as i32 {
                test_pos.y = 0;
            }

            let idx = test_pos.to_index();
            let cell = state.maze.get(idx).unwrap();
            if cell == CellType::Wall {
                continue;
            }

            let new_dx = pacman_pos.x - test_pos.x;
            let new_dy = pacman_pos.y - test_pos.y;
            let distance = new_dx.abs() + new_dy.abs();

            let score = match ghost.mode {
                GhostMode::Chase => -distance,
                GhostMode::Frightened => distance,
            };

            if score > best_score {
                best_score = score;
                best_dir = *dir;
            }
        }

        best_dir
    }

    pub fn check_ghost_collisions(env: &Env, state: &mut GameState) {
        let pacman_pos = state.pacman_pos;
        state.last_collision_events = Vec::new(env);

        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();

            if ghost.position == pacman_pos {
                let collision_event = ghost.create_collision_event();
                let event_data = collision_event.serialize(env);
                let event = Event::new(CollisionEvent::event_type(), event_data);
                state.last_collision_events.push_back(event);

                match ghost.mode {
                    GhostMode::Chase => {
                        state.lives -= 1;
                        if state.lives == 0 {
                            state.game_over = true;
                            state.won = false;
                        } else {
                            state.pacman_pos = state.pacman_start;
                            state.pacman_dir = Direction::Right;
                        }
                    }
                    GhostMode::Frightened => {
                        state.score += GHOST_POINTS;
                        ghost.respawn();
                        state.ghosts.set(i, ghost);
                    }
                }
            }
        }
    }

    pub fn activate_power_mode(_env: &Env, state: &mut GameState) {
        state.power_mode_timer = POWER_MODE_DURATION;
        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();
            ghost.mode = GhostMode::Frightened;
            ghost.frightened_timer = POWER_MODE_DURATION;
            state.ghosts.set(i, ghost);
        }
    }

    pub fn end_frightened_mode(_env: &Env, state: &mut GameState) {
        for i in 0..state.ghosts.len() {
            let mut ghost = state.ghosts.get(i).unwrap();
            ghost.mode = GhostMode::Chase;
            ghost.frightened_timer = 0;
            state.ghosts.set(i, ghost);
        }
    }
}

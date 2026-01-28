#![no_std]

use cougr_core::*;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Env, Vec,
};

#[contract]
pub struct Contract;

const SCALE: i128 = 1000;
const WORLD_W: i128 = 1000 * SCALE;
const WORLD_H: i128 = 1000 * SCALE;
const SHIP_THRUST: i128 = 120;
const BULLET_SPEED: i128 = 300;
const BULLET_TTL: u32 = 50;
const MAX_BULLETS: u32 = 32;
const MAX_ASTEROIDS: u32 = 64;
const ASTEROID_BASE_RADIUS: i128 = 28 * SCALE;
const SHIP_RADIUS: i128 = 20 * SCALE;
const DIRECTIONS: u32 = 8;

#[contracterror]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum GameError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    GameOver = 3,
    InvalidAction = 4,
}

// We keep the on-chain state as Soroban-friendly structs to minimize storage reads/writes.
// Cougr-Core provides the ECS building blocks for richer simulations; this example keeps
// storage minimal while still importing and exercising Cougr-Core utilities.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vec2 {
    pub x: i128,
    pub y: i128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ship {
    pub position: Vec2,
    pub velocity: Vec2,
    pub rotation: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Asteroid {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub ttl: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub ship: Ship,
    pub asteroids: Vec<Asteroid>,
    pub bullets: Vec<Bullet>,
    pub score: u32,
    pub lives: u32,
    pub game_over: bool,
}

fn state_key() -> soroban_sdk::Symbol {
    symbol_short!("state")
}

fn load_state(env: &Env) -> GameState {
    env.storage()
        .instance()
        .get(&state_key())
        .unwrap_or_else(|| panic_with_error!(env, GameError::NotInitialized))
}

fn save_state(env: &Env, state: &GameState) {
    env.storage().instance().set(&state_key(), state);
}

fn heading_vector(index: u32) -> (i128, i128) {
    match index % DIRECTIONS {
        0 => (0, SCALE),
        1 => (707, 707),
        2 => (SCALE, 0),
        3 => (707, -707),
        4 => (0, -SCALE),
        5 => (-707, -707),
        6 => (-SCALE, 0),
        _ => (-707, 707),
    }
}

fn wrap(mut value: i128, max: i128) -> i128 {
    while value < 0 {
        value += max;
    }
    while value >= max {
        value -= max;
    }
    value
}

fn dist2(a: Vec2, b: Vec2) -> i128 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn reset_ship() -> Ship {
    Ship {
        position: Vec2 {
            x: WORLD_W / 2,
            y: WORLD_H / 2,
        },
        velocity: Vec2 { x: 0, y: 0 },
        rotation: 0,
    }
}

#[contractimpl]
impl Contract {
    // This small function verifies the Cougr-Core dependency is wired up correctly.
    // Cougr-Core's ECS lets you express game logic as reusable systems over components,
    // rather than hand-rolling per-entity loops directly against Soroban storage.
    // Here we keep storage minimal but still show Cougr-Core systems in action.
    pub fn cougr_smoke(env: Env) -> u32 {
        let mut world = create_world();
        let components = Vec::new(&env);
        let _entity_id = spawn_entity(&mut world, components);
        let position = Position { x: 500, y: 500 };
        let _next = MovementSystem::update(&position, 10, -5);
        world.entity_count() as u32
    }

    pub fn init_game(env: Env) {
        if env.storage().instance().has(&state_key()) {
            panic_with_error!(&env, GameError::AlreadyInitialized);
        }

        let mut asteroids = Vec::new(&env);
        asteroids.push_back(Asteroid {
            position: Vec2 {
                x: 200 * SCALE,
                y: 800 * SCALE,
            },
            velocity: Vec2 { x: 40, y: -30 },
            size: 3,
        });
        asteroids.push_back(Asteroid {
            position: Vec2 {
                x: 800 * SCALE,
                y: 200 * SCALE,
            },
            velocity: Vec2 { x: -25, y: 35 },
            size: 2,
        });

        let state = GameState {
            ship: reset_ship(),
            asteroids,
            bullets: Vec::new(&env),
            score: 0,
            lives: 3,
            game_over: false,
        };
        save_state(&env, &state);
    }

    pub fn thrust_ship(env: Env) {
        let mut state = load_state(&env);
        if state.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }

        let (dx, dy) = heading_vector(state.ship.rotation);
        state.ship.velocity.x += dx * SHIP_THRUST / SCALE;
        state.ship.velocity.y += dy * SHIP_THRUST / SCALE;
        save_state(&env, &state);
    }

    pub fn rotate_ship(env: Env, delta_steps: i32) {
        let mut state = load_state(&env);
        if state.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }

        let rot = state.ship.rotation as i32;
        state.ship.rotation = (rot + delta_steps).rem_euclid(DIRECTIONS as i32) as u32;
        save_state(&env, &state);
    }

    pub fn shoot(env: Env) {
        let mut state = load_state(&env);
        if state.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }
        if state.bullets.len() >= MAX_BULLETS {
            panic_with_error!(&env, GameError::InvalidAction);
        }

        let (dx, dy) = heading_vector(state.ship.rotation);
        let bullet = Bullet {
            position: state.ship.position,
            velocity: Vec2 {
                x: state.ship.velocity.x + dx * BULLET_SPEED / SCALE,
                y: state.ship.velocity.y + dy * BULLET_SPEED / SCALE,
            },
            ttl: BULLET_TTL,
        };
        state.bullets.push_back(bullet);
        save_state(&env, &state);
    }

    pub fn update_tick(env: Env) {
        let mut state = load_state(&env);
        if state.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }

        state.ship.position.x = wrap(state.ship.position.x + state.ship.velocity.x, WORLD_W);
        state.ship.position.y = wrap(state.ship.position.y + state.ship.velocity.y, WORLD_H);

        let mut bullets = Vec::new(&env);
        let mut i = 0;
        while i < state.bullets.len() {
            let mut bullet = state.bullets.get(i).unwrap();
            if bullet.ttl > 0 {
                bullet.ttl -= 1;
                bullet.position.x = wrap(bullet.position.x + bullet.velocity.x, WORLD_W);
                bullet.position.y = wrap(bullet.position.y + bullet.velocity.y, WORLD_H);
                bullets.push_back(bullet);
            }
            i += 1;
        }

        let mut asteroids = Vec::new(&env);
        let mut j = 0;
        while j < state.asteroids.len() {
            let mut asteroid = state.asteroids.get(j).unwrap();
            asteroid.position.x = wrap(asteroid.position.x + asteroid.velocity.x, WORLD_W);
            asteroid.position.y = wrap(asteroid.position.y + asteroid.velocity.y, WORLD_H);
            asteroids.push_back(asteroid);
            j += 1;
        }

        let mut asteroid_hit = Vec::new(&env);
        let mut k = 0;
        while k < asteroids.len() {
            asteroid_hit.push_back(false);
            k += 1;
        }

        let mut remaining_bullets = Vec::new(&env);
        let mut b = 0;
        while b < bullets.len() {
            let bullet = bullets.get(b).unwrap();
            let mut hit = false;
            let mut a = 0;
            while a < asteroids.len() {
                if !asteroid_hit.get(a).unwrap() {
                    let asteroid = asteroids.get(a).unwrap();
                    let radius = ASTEROID_BASE_RADIUS * asteroid.size as i128;
                    if dist2(bullet.position, asteroid.position) <= radius * radius {
                        asteroid_hit.set(a, true);
                        hit = true;
                        state.score += 10;
                        break;
                    }
                }
                a += 1;
            }
            if !hit {
                remaining_bullets.push_back(bullet);
            }
            b += 1;
        }

        let mut remaining_asteroids = Vec::new(&env);
        let mut a = 0;
        while a < asteroids.len() {
            let asteroid = asteroids.get(a).unwrap();
            if asteroid_hit.get(a).unwrap() {
                if asteroid.size > 1 && remaining_asteroids.len() + 2 <= MAX_ASTEROIDS {
                    let new_size = asteroid.size - 1;
                    remaining_asteroids.push_back(Asteroid {
                        position: asteroid.position,
                        velocity: Vec2 {
                            x: asteroid.velocity.y,
                            y: -asteroid.velocity.x,
                        },
                        size: new_size,
                    });
                    remaining_asteroids.push_back(Asteroid {
                        position: asteroid.position,
                        velocity: Vec2 {
                            x: -asteroid.velocity.y,
                            y: asteroid.velocity.x,
                        },
                        size: new_size,
                    });
                }
            } else {
                remaining_asteroids.push_back(asteroid);
            }
            a += 1;
        }

        let mut collided = false;
        let mut c = 0;
        while c < remaining_asteroids.len() {
            let asteroid = remaining_asteroids.get(c).unwrap();
            let radius = ASTEROID_BASE_RADIUS * asteroid.size as i128 + SHIP_RADIUS;
            if dist2(state.ship.position, asteroid.position) <= radius * radius {
                collided = true;
                break;
            }
            c += 1;
        }

        if collided {
            if state.lives > 0 {
                state.lives -= 1;
            }
            state.ship = reset_ship();
            remaining_bullets = Vec::new(&env);
            if state.lives == 0 {
                state.game_over = true;
            }
        }

        if remaining_asteroids.len() == 0 {
            state.game_over = true;
        }

        state.asteroids = remaining_asteroids;
        state.bullets = remaining_bullets;
        save_state(&env, &state);
    }

    pub fn get_score(env: Env) -> u32 {
        let state = load_state(&env);
        state.score
    }

    pub fn check_game_over(env: Env) -> bool {
        let state = load_state(&env);
        state.game_over
    }
}

mod test;

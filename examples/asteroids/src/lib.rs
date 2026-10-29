#![no_std]

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

// ECS Components following Cougr-Core patterns

#[contracttype]
#[derive(Clone, Debug)]
pub struct ShipComponent {
    pub x: i128,
    pub y: i128,
    pub vx: i128,
    pub vy: i128,
    pub angle: u32,
}

cougr_core::impl_component!(ShipComponent, "ship", Table, { x: i128, y: i128, vx: i128, vy: i128, angle: u32 });

#[contracttype]
#[derive(Clone, Debug)]
pub struct AsteroidComponent {
    pub x: i128,
    pub y: i128,
    pub vx: i128,
    pub vy: i128,
    pub size: u32,
}

cougr_core::impl_component!(AsteroidComponent, "asteroid", Table, { x: i128, y: i128, vx: i128, vy: i128, size: u32 });

#[contracttype]
#[derive(Clone, Debug)]
pub struct BulletComponent {
    pub x: i128,
    pub y: i128,
    pub vx: i128,
    pub vy: i128,
    pub lifetime: u32,
}

cougr_core::impl_component!(BulletComponent, "bullet", Table, { x: i128, y: i128, vx: i128, vy: i128, lifetime: u32 });

#[contracttype]
#[derive(Clone, Debug)]
pub struct ScoreComponent {
    pub points: u32,
    pub lives: u32,
}

cougr_core::impl_component!(ScoreComponent, "score", Table, { points: u32, lives: u32 });

// ECS World State
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub ship: ShipComponent,
    pub asteroids: Vec<AsteroidComponent>,
    pub bullets: Vec<BulletComponent>,
    pub score: ScoreComponent,
    pub game_over: bool,
}

// External API state
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameState {
    pub ship_x: i128,
    pub ship_y: i128,
    pub ship_vx: i128,
    pub ship_vy: i128,
    pub ship_angle: u32,
    pub asteroid_count: u32,
    pub bullet_count: u32,
    pub score: u32,
    pub lives: u32,
    pub game_over: bool,
}

fn state_key() -> soroban_sdk::Symbol {
    symbol_short!("state")
}

fn load_world(env: &Env) -> ECSWorldState {
    env.storage()
        .instance()
        .get(&state_key())
        .unwrap_or_else(|| panic_with_error!(env, GameError::NotInitialized))
}

fn save_world(env: &Env, world: &ECSWorldState) {
    env.storage().instance().set(&state_key(), world);
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

fn dist2(x1: i128, y1: i128, x2: i128, y2: i128) -> i128 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    dx * dx + dy * dy
}

// ECS Systems

struct MovementSystem;
impl MovementSystem {
    fn update_ship(ship: &mut ShipComponent) {
        ship.x = wrap(ship.x + ship.vx, WORLD_W);
        ship.y = wrap(ship.y + ship.vy, WORLD_H);
    }

    fn update_bullet(bullet: &mut BulletComponent) {
        bullet.x = wrap(bullet.x + bullet.vx, WORLD_W);
        bullet.y = wrap(bullet.y + bullet.vy, WORLD_H);
    }

    fn update_asteroid(asteroid: &mut AsteroidComponent) {
        asteroid.x = wrap(asteroid.x + asteroid.vx, WORLD_W);
        asteroid.y = wrap(asteroid.y + asteroid.vy, WORLD_H);
    }
}

struct CollisionSystem;
impl CollisionSystem {
    fn check_bullet_asteroid(bullet: &BulletComponent, asteroid: &AsteroidComponent) -> bool {
        let radius = ASTEROID_BASE_RADIUS * asteroid.size as i128;
        dist2(bullet.x, bullet.y, asteroid.x, asteroid.y) <= radius * radius
    }

    fn check_ship_asteroid(ship: &ShipComponent, asteroid: &AsteroidComponent) -> bool {
        let radius = ASTEROID_BASE_RADIUS * asteroid.size as i128 + SHIP_RADIUS;
        dist2(ship.x, ship.y, asteroid.x, asteroid.y) <= radius * radius
    }
}

struct ShootingSystem;
impl ShootingSystem {
    fn spawn_bullet(ship: &ShipComponent) -> BulletComponent {
        let (dx, dy) = heading_vector(ship.angle);
        BulletComponent {
            x: ship.x,
            y: ship.y,
            vx: ship.vx + dx * BULLET_SPEED / SCALE,
            vy: ship.vy + dy * BULLET_SPEED / SCALE,
            lifetime: BULLET_TTL,
        }
    }
}

struct AsteroidSplitSystem;
impl AsteroidSplitSystem {
    fn split(asteroid: &AsteroidComponent, env: &Env) -> Vec<AsteroidComponent> {
        let mut result = Vec::new(env);
        if asteroid.size > 1 {
            let new_size = asteroid.size - 1;
            result.push_back(AsteroidComponent {
                x: asteroid.x,
                y: asteroid.y,
                vx: asteroid.vy,
                vy: -asteroid.vx,
                size: new_size,
            });
            result.push_back(AsteroidComponent {
                x: asteroid.x,
                y: asteroid.y,
                vx: -asteroid.vy,
                vy: asteroid.vx,
                size: new_size,
            });
        }
        result
    }
}

#[contractimpl]
impl Contract {
    pub fn init_game(env: Env) {
        if env.storage().instance().has(&state_key()) {
            panic_with_error!(&env, GameError::AlreadyInitialized);
        }

        let mut asteroids = Vec::new(&env);
        asteroids.push_back(AsteroidComponent {
            x: 200 * SCALE,
            y: 800 * SCALE,
            vx: 40,
            vy: -30,
            size: 3,
        });
        asteroids.push_back(AsteroidComponent {
            x: 800 * SCALE,
            y: 200 * SCALE,
            vx: -25,
            vy: 35,
            size: 2,
        });

        let world = ECSWorldState {
            ship: ShipComponent {
                x: WORLD_W / 2,
                y: WORLD_H / 2,
                vx: 0,
                vy: 0,
                angle: 0,
            },
            asteroids,
            bullets: Vec::new(&env),
            score: ScoreComponent {
                points: 0,
                lives: 3,
            },
            game_over: false,
        };
        save_world(&env, &world);
    }

    pub fn thrust_ship(env: Env) {
        let mut world = load_world(&env);
        if world.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }

        let (dx, dy) = heading_vector(world.ship.angle);
        world.ship.vx += dx * SHIP_THRUST / SCALE;
        world.ship.vy += dy * SHIP_THRUST / SCALE;
        save_world(&env, &world);
    }

    pub fn rotate_ship(env: Env, delta_steps: i32) {
        let mut world = load_world(&env);
        if world.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }

        let rot = world.ship.angle as i32;
        world.ship.angle = (rot + delta_steps).rem_euclid(DIRECTIONS as i32) as u32;
        save_world(&env, &world);
    }

    pub fn shoot(env: Env) {
        let mut world = load_world(&env);
        if world.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }
        if world.bullets.len() >= MAX_BULLETS {
            panic_with_error!(&env, GameError::InvalidAction);
        }

        let bullet = ShootingSystem::spawn_bullet(&world.ship);
        world.bullets.push_back(bullet);
        save_world(&env, &world);
    }

    pub fn update_tick(env: Env) {
        let mut world = load_world(&env);
        if world.game_over {
            panic_with_error!(&env, GameError::GameOver);
        }

        // MovementSystem
        MovementSystem::update_ship(&mut world.ship);

        let mut bullets = Vec::new(&env);
        let mut i = 0;
        while i < world.bullets.len() {
            let mut bullet = world.bullets.get(i).unwrap();
            bullet.lifetime = bullet.lifetime.saturating_sub(1);
            if bullet.lifetime > 0 {
                MovementSystem::update_bullet(&mut bullet);
                bullets.push_back(bullet);
            }
            i += 1;
        }

        let mut asteroids = Vec::new(&env);
        let mut j = 0;
        while j < world.asteroids.len() {
            let mut asteroid = world.asteroids.get(j).unwrap();
            MovementSystem::update_asteroid(&mut asteroid);
            asteroids.push_back(asteroid);
            j += 1;
        }

        // CollisionSystem - bullet-asteroid
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
                    if CollisionSystem::check_bullet_asteroid(&bullet, &asteroid) {
                        asteroid_hit.set(a, true);
                        hit = true;
                        world.score.points += 10;
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

        // AsteroidSplitSystem
        let mut remaining_asteroids = Vec::new(&env);
        let mut a = 0;
        while a < asteroids.len() {
            let asteroid = asteroids.get(a).unwrap();
            if asteroid_hit.get(a).unwrap() {
                let splits = AsteroidSplitSystem::split(&asteroid, &env);
                if remaining_asteroids.len() + splits.len() <= MAX_ASTEROIDS {
                    let mut s = 0;
                    while s < splits.len() {
                        remaining_asteroids.push_back(splits.get(s).unwrap());
                        s += 1;
                    }
                }
            } else {
                remaining_asteroids.push_back(asteroid);
            }
            a += 1;
        }

        // CollisionSystem - ship-asteroid
        let mut collided = false;
        let mut c = 0;
        while c < remaining_asteroids.len() {
            let asteroid = remaining_asteroids.get(c).unwrap();
            if CollisionSystem::check_ship_asteroid(&world.ship, &asteroid) {
                collided = true;
                break;
            }
            c += 1;
        }

        if collided {
            if world.score.lives > 0 {
                world.score.lives -= 1;
            }
            world.ship = ShipComponent {
                x: WORLD_W / 2,
                y: WORLD_H / 2,
                vx: 0,
                vy: 0,
                angle: 0,
            };
            remaining_bullets = Vec::new(&env);
            if world.score.lives == 0 {
                world.game_over = true;
            }
        }

        if remaining_asteroids.is_empty() {
            world.game_over = true;
        }

        world.asteroids = remaining_asteroids;
        world.bullets = remaining_bullets;
        save_world(&env, &world);
    }

    pub fn get_score(env: Env) -> u32 {
        let world = load_world(&env);
        world.score.points
    }

    pub fn check_game_over(env: Env) -> bool {
        let world = load_world(&env);
        world.game_over
    }

    pub fn get_game_state(env: Env) -> GameState {
        let world = load_world(&env);
        GameState {
            ship_x: world.ship.x,
            ship_y: world.ship.y,
            ship_vx: world.ship.vx,
            ship_vy: world.ship.vy,
            ship_angle: world.ship.angle,
            asteroid_count: world.asteroids.len(),
            bullet_count: world.bullets.len(),
            score: world.score.points,
            lives: world.score.lives,
            game_over: world.game_over,
        }
    }
}

mod test;

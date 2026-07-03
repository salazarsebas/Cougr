#![no_std]

use cougr_core::privacy::stable::{
    MerkleProofVerifier, OnChainMerkleProof, Sha256MerkleProofVerifier, SparseMerkleTree,
};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Bytes, BytesN,
    Env, Map,
};

#[cfg(test)]
mod test;

const INITIAL_X: u32 = 0;
const INITIAL_Y: u32 = 0;
const DEFAULT_MAX_HEALTH: u32 = 3;
const DEFAULT_TRAP_DAMAGE: u32 = 1;
const DEFAULT_TREASURE_VALUE: u32 = 100;

const CELL_TREASURE: u8 = 1;
const CELL_TRAP: u8 = 2;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TreasureHuntError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    OutOfBounds = 4,
    NonAdjacentMove = 5,
    InvalidCellValue = 6,
    InvalidProof = 7,
    AlreadyExplored = 8,
    GameFinished = 9,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameStatus {
    Active,
    Won,
    Lost,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MapRoot {
    pub root: BytesN<32>,
    pub width: u32,
    pub height: u32,
    pub total_treasures: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    pub x: u32,
    pub y: u32,
    pub health: u32,
    pub score: u32,
    pub treasures_found: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExploredMap {
    pub explored: Map<u32, bool>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameConfig {
    pub max_health: u32,
    pub trap_damage: u32,
    pub treasure_value: u32,
    pub status: GameStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub player: Address,
    pub map_root: MapRoot,
    pub player_state: PlayerState,
    pub explored_map: ExploredMap,
    pub game_config: GameConfig,
    pub fog_root: BytesN<32>,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Player,
    MapRoot,
    PlayerState,
    ExploredMap,
    GameConfig,
    FogRoot,
}

#[contract]
pub struct TreasureHuntContract;

#[contractimpl]
impl TreasureHuntContract {
    pub fn init_game(
        env: Env,
        player: Address,
        map_root: BytesN<32>,
        width: u32,
        height: u32,
        total_treasures: u32,
    ) {
        if env.storage().instance().has(&DataKey::MapRoot) {
            panic_with_error!(&env, TreasureHuntError::AlreadyInitialized);
        }
        if width == 0 || height == 0 {
            panic_with_error!(&env, TreasureHuntError::OutOfBounds);
        }

        player.require_auth();

        let map = MapRoot {
            root: map_root,
            width,
            height,
            total_treasures,
        };
        let player_state = PlayerState {
            x: INITIAL_X,
            y: INITIAL_Y,
            health: DEFAULT_MAX_HEALTH,
            score: 0,
            treasures_found: 0,
        };
        let explored_map = ExploredMap {
            explored: Map::new(&env),
        };
        let game_config = GameConfig {
            max_health: DEFAULT_MAX_HEALTH,
            trap_damage: DEFAULT_TRAP_DAMAGE,
            treasure_value: DEFAULT_TREASURE_VALUE,
            status: GameStatus::Active,
        };
        let fog_root = SparseMerkleTree::new(&env).root_bytes(&env);

        env.storage().instance().set(&DataKey::Player, &player);
        env.storage().instance().set(&DataKey::MapRoot, &map);
        env.storage()
            .instance()
            .set(&DataKey::PlayerState, &player_state);
        env.storage()
            .instance()
            .set(&DataKey::ExploredMap, &explored_map);
        env.storage()
            .instance()
            .set(&DataKey::GameConfig, &game_config);
        env.storage().instance().set(&DataKey::FogRoot, &fog_root);
    }

    pub fn explore(
        env: Env,
        player: Address,
        x: u32,
        y: u32,
        cell_value: u32,
        proof: OnChainMerkleProof,
    ) {
        ensure_initialized(&env);
        ensure_active(&env);
        ensure_player_auth(&env, &player);
        validate_cell_value(&env, cell_value);
        let cell_value_u8 = cell_value as u8;

        let map_root = read_map_root(&env);
        validate_bounds(&env, x, y, map_root.width, map_root.height);

        let mut player_state = read_player_state(&env);
        validate_adjacent_move(&env, player_state.x, player_state.y, x, y);

        let mut explored_map = read_explored_map(&env);
        let idx = cell_index(x, y, map_root.width);
        if explored_map.explored.get(idx).unwrap_or(false) {
            panic_with_error!(&env, TreasureHuntError::AlreadyExplored);
        }

        let expected_leaf = hash_leaf_for_proof(&env, x, y, cell_value_u8);
        if proof.leaf_index != idx || proof.leaf != expected_leaf {
            panic_with_error!(&env, TreasureHuntError::InvalidProof);
        }

        let verifier = Sha256MerkleProofVerifier;
        let is_valid = verifier
            .verify(&env, &proof, &map_root.root)
            .unwrap_or_else(|_| panic_with_error!(&env, TreasureHuntError::InvalidProof));
        if !is_valid {
            panic_with_error!(&env, TreasureHuntError::InvalidProof);
        }

        let mut game_config = read_game_config(&env);
        apply_discovery(
            &mut player_state,
            &mut game_config,
            &map_root,
            x,
            y,
            cell_value_u8,
        );

        explored_map.explored.set(idx, true);
        let fog_root = recompute_fog_root(
            &env,
            &explored_map.explored,
            map_root.width * map_root.height,
        );

        env.storage()
            .instance()
            .set(&DataKey::PlayerState, &player_state);
        env.storage()
            .instance()
            .set(&DataKey::ExploredMap, &explored_map);
        env.storage()
            .instance()
            .set(&DataKey::GameConfig, &game_config);
        env.storage().instance().set(&DataKey::FogRoot, &fog_root);
    }

    pub fn get_state(env: Env) -> GameState {
        ensure_initialized(&env);
        GameState {
            player: env
                .storage()
                .instance()
                .get(&DataKey::Player)
                .unwrap_or_else(|| panic_with_error!(&env, TreasureHuntError::NotInitialized)),
            map_root: read_map_root(&env),
            player_state: read_player_state(&env),
            explored_map: read_explored_map(&env),
            game_config: read_game_config(&env),
            fog_root: env
                .storage()
                .instance()
                .get(&DataKey::FogRoot)
                .unwrap_or_else(|| panic_with_error!(&env, TreasureHuntError::NotInitialized)),
        }
    }

    pub fn is_explored(env: Env, x: u32, y: u32) -> bool {
        ensure_initialized(&env);
        let map_root = read_map_root(&env);
        if x >= map_root.width || y >= map_root.height {
            return false;
        }
        let explored_map = read_explored_map(&env);
        explored_map
            .explored
            .get(cell_index(x, y, map_root.width))
            .unwrap_or(false)
    }
}

fn ensure_initialized(env: &Env) {
    if !env.storage().instance().has(&DataKey::MapRoot) {
        panic_with_error!(env, TreasureHuntError::NotInitialized);
    }
}

fn ensure_active(env: &Env) {
    let game_config = read_game_config(env);
    if game_config.status != GameStatus::Active {
        panic_with_error!(env, TreasureHuntError::GameFinished);
    }
}

fn ensure_player_auth(env: &Env, player: &Address) {
    player.require_auth();
    let stored: Address = env
        .storage()
        .instance()
        .get(&DataKey::Player)
        .unwrap_or_else(|| panic_with_error!(env, TreasureHuntError::NotInitialized));
    if stored != *player {
        panic_with_error!(env, TreasureHuntError::Unauthorized);
    }
}

fn validate_cell_value(env: &Env, cell_value: u32) {
    if cell_value > CELL_TRAP as u32 {
        panic_with_error!(env, TreasureHuntError::InvalidCellValue);
    }
}

fn validate_bounds(env: &Env, x: u32, y: u32, width: u32, height: u32) {
    if x >= width || y >= height {
        panic_with_error!(env, TreasureHuntError::OutOfBounds);
    }
}

fn validate_adjacent_move(env: &Env, from_x: u32, from_y: u32, to_x: u32, to_y: u32) {
    let dx = from_x.abs_diff(to_x);
    let dy = from_y.abs_diff(to_y);
    if dx + dy != 1 {
        panic_with_error!(env, TreasureHuntError::NonAdjacentMove);
    }
}

fn read_map_root(env: &Env) -> MapRoot {
    env.storage()
        .instance()
        .get(&DataKey::MapRoot)
        .unwrap_or_else(|| panic_with_error!(env, TreasureHuntError::NotInitialized))
}

fn read_player_state(env: &Env) -> PlayerState {
    env.storage()
        .instance()
        .get(&DataKey::PlayerState)
        .unwrap_or_else(|| panic_with_error!(env, TreasureHuntError::NotInitialized))
}

fn read_explored_map(env: &Env) -> ExploredMap {
    env.storage()
        .instance()
        .get(&DataKey::ExploredMap)
        .unwrap_or_else(|| panic_with_error!(env, TreasureHuntError::NotInitialized))
}

fn read_game_config(env: &Env) -> GameConfig {
    env.storage()
        .instance()
        .get(&DataKey::GameConfig)
        .unwrap_or_else(|| panic_with_error!(env, TreasureHuntError::NotInitialized))
}

fn cell_index(x: u32, y: u32, width: u32) -> u32 {
    y * width + x
}

fn encode_cell_leaf(x: u32, y: u32, cell_value: u8) -> [u8; 32] {
    let mut data = [0u8; 32];
    data[0..4].copy_from_slice(&x.to_be_bytes());
    data[4..8].copy_from_slice(&y.to_be_bytes());
    data[8] = cell_value;
    data
}

fn hash_leaf_for_proof(env: &Env, x: u32, y: u32, cell_value: u8) -> BytesN<32> {
    let raw = encode_cell_leaf(x, y, cell_value);
    let mut inbuf = [0u8; 33];
    inbuf[0] = 0x00;
    inbuf[1..].copy_from_slice(&raw);
    let hash = env
        .crypto()
        .sha256(&Bytes::from_slice(env, &inbuf))
        .to_array();
    BytesN::from_array(env, &hash)
}

fn apply_discovery(
    player_state: &mut PlayerState,
    game_config: &mut GameConfig,
    map_root: &MapRoot,
    x: u32,
    y: u32,
    cell_value: u8,
) {
    player_state.x = x;
    player_state.y = y;

    if cell_value == CELL_TREASURE {
        player_state.score += game_config.treasure_value;
        player_state.treasures_found += 1;
    } else if cell_value == CELL_TRAP {
        player_state.health = player_state.health.saturating_sub(game_config.trap_damage);
    }

    if player_state.health == 0 {
        game_config.status = GameStatus::Lost;
    } else if player_state.treasures_found >= map_root.total_treasures {
        game_config.status = GameStatus::Won;
    }
}

fn recompute_fog_root(env: &Env, explored: &Map<u32, bool>, max_cells: u32) -> BytesN<32> {
    let mut smt = SparseMerkleTree::new(env);
    for cell_idx in 0..max_cells {
        let is_seen = explored.get(cell_idx).unwrap_or(false);
        if is_seen {
            let mut key = [0u8; 32];
            key[0..4].copy_from_slice(&cell_idx.to_be_bytes());
            let value = [1u8; 32];
            smt.insert(env, &key, &value).unwrap_or_else(|_| {
                panic_with_error!(env, TreasureHuntError::InvalidProof);
            });
        }
    }
    smt.root_bytes(env)
}

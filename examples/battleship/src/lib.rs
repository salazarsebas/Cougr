#![no_std]

use cougr_core::component::ComponentTrait;
use cougr_core::privacy::stable::{
    MerkleProofVerifier, OnChainMerkleProof, Sha256MerkleProofVerifier,
};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, BytesN, Env, Map, Symbol,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellResult {
    Unknown,
    Miss,
    Hit,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    Setup,
    Attack,
    Finished,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardCommitment {
    pub commitment: BytesN<32>,
    pub merkle_root: BytesN<32>,
}

impl ComponentTrait for BoardCommitment {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        for i in 0..32 {
            bytes.push_back(self.commitment.get(i).unwrap());
        }
        for i in 0..32 {
            bytes.push_back(self.merkle_root.get(i).unwrap());
        }
        bytes
    }

    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AttackGrid {
    pub cells: Map<u32, CellResult>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ShipStatus {
    pub remaining_a: u32,
    pub remaining_b: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnState {
    pub current_player: Address,
    pub phase: Phase,
    pub pending_reveal_x: u32,
    pub pending_reveal_y: u32,
    pub has_pending: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub player_a: Address,
    pub player_b: Address,
    pub commitment_a: BytesN<32>,
    pub merkle_root_a: BytesN<32>,
    pub commitment_b: BytesN<32>,
    pub merkle_root_b: BytesN<32>,
    pub has_commitment_a: bool,
    pub has_commitment_b: bool,
    pub attack_grid_a: AttackGrid,
    pub attack_grid_b: AttackGrid,
    pub ship_status: ShipStatus,
    pub turn_state: TurnState,
    pub winner: Option<Address>,
}

const GAME_KEY: Symbol = symbol_short!("GAME");
const GRID_SIZE: u32 = 10;
const TOTAL_SHIP_CELLS: u32 = 17; // 5+4+3+3+2

#[contract]
#[derive(Clone)]
pub struct BattleshipContract;

#[contractimpl]
impl BattleshipContract {
    pub fn new_game(env: Env, player_a: Address, player_b: Address) {
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        let game = GameState {
            player_a: player_a.clone(),
            player_b,
            commitment_a: zero_hash.clone(),
            merkle_root_a: zero_hash.clone(),
            commitment_b: zero_hash.clone(),
            merkle_root_b: zero_hash.clone(),
            has_commitment_a: false,
            has_commitment_b: false,
            attack_grid_a: AttackGrid {
                cells: Map::new(&env),
            },
            attack_grid_b: AttackGrid {
                cells: Map::new(&env),
            },
            ship_status: ShipStatus {
                remaining_a: TOTAL_SHIP_CELLS,
                remaining_b: TOTAL_SHIP_CELLS,
            },
            turn_state: TurnState {
                current_player: player_a,
                phase: Phase::Setup,
                pending_reveal_x: 0,
                pending_reveal_y: 0,
                has_pending: false,
            },
            winner: None,
        };
        env.storage().instance().set(&GAME_KEY, &game);
    }

    pub fn commit_board(
        env: Env,
        player: Address,
        commitment: BytesN<32>,
        merkle_root: BytesN<32>,
    ) {
        player.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if game.turn_state.phase != Phase::Setup {
            panic!("Not in setup phase");
        }

        if player == game.player_a {
            if game.has_commitment_a {
                panic!("Already committed");
            }
            game.commitment_a = commitment;
            game.merkle_root_a = merkle_root;
            game.has_commitment_a = true;
        } else if player == game.player_b {
            if game.has_commitment_b {
                panic!("Already committed");
            }
            game.commitment_b = commitment;
            game.merkle_root_b = merkle_root;
            game.has_commitment_b = true;
        } else {
            panic!("Not a player");
        }

        // Transition to attack phase when both committed
        if game.has_commitment_a && game.has_commitment_b {
            game.turn_state.phase = Phase::Attack;
        }

        env.storage().instance().set(&GAME_KEY, &game);
    }

    pub fn attack(env: Env, attacker: Address, x: u32, y: u32) {
        attacker.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if game.turn_state.phase != Phase::Attack {
            panic!("Not in attack phase");
        }

        if game.turn_state.current_player != attacker {
            panic!("Not your turn");
        }

        if game.turn_state.has_pending {
            panic!("Pending reveal");
        }

        if x >= GRID_SIZE || y >= GRID_SIZE {
            panic!("Invalid coordinates");
        }

        let coord = Self::coord_to_index(x, y);

        // Check if already attacked
        let grid = if attacker == game.player_a {
            &game.attack_grid_b
        } else {
            &game.attack_grid_a
        };

        if grid.cells.get(coord).is_some() {
            panic!("Already attacked");
        }

        // Set pending reveal
        game.turn_state.pending_reveal_x = x;
        game.turn_state.pending_reveal_y = y;
        game.turn_state.has_pending = true;

        env.storage().instance().set(&GAME_KEY, &game);
    }

    pub fn reveal_cell(
        env: Env,
        defender: Address,
        x: u32,
        y: u32,
        value: u32,
        proof: OnChainMerkleProof,
    ) {
        defender.require_auth();

        let mut game: GameState = env
            .storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"));

        if !game.turn_state.has_pending {
            panic!("No pending attack");
        }

        if x != game.turn_state.pending_reveal_x || y != game.turn_state.pending_reveal_y {
            panic!("Wrong coordinates");
        }

        // Verify defender is correct
        let (is_defender_a, merkle_root) = if defender == game.player_a {
            (true, &game.merkle_root_a)
        } else if defender == game.player_b {
            (false, &game.merkle_root_b)
        } else {
            panic!("Not a player");
        };

        // Verify Merkle proof
        let coord = Self::coord_to_index(x, y);
        if proof.leaf_index != coord {
            panic!("Invalid proof");
        }

        let expected_leaf = Self::leaf_hash(&env, coord, value);
        if proof.leaf != expected_leaf {
            panic!("Invalid proof");
        }

        let verifier = Sha256MerkleProofVerifier;
        if !verifier.verify(&env, &proof, merkle_root).unwrap_or(false) {
            panic!("Invalid proof");
        }

        // Record result
        let result = if value == 1 {
            CellResult::Hit
        } else {
            CellResult::Miss
        };

        if is_defender_a {
            game.attack_grid_a.cells.set(coord, result.clone());
            if result == CellResult::Hit {
                game.ship_status.remaining_a -= 1;
            }
        } else {
            game.attack_grid_b.cells.set(coord, result.clone());
            if result == CellResult::Hit {
                game.ship_status.remaining_b -= 1;
            }
        }

        // Clear pending
        game.turn_state.has_pending = false;

        // Check win condition
        if game.ship_status.remaining_a == 0 {
            game.turn_state.phase = Phase::Finished;
            game.winner = Some(game.player_b.clone());
        } else if game.ship_status.remaining_b == 0 {
            game.turn_state.phase = Phase::Finished;
            game.winner = Some(game.player_a.clone());
        } else {
            // Switch turn
            game.turn_state.current_player = if game.turn_state.current_player == game.player_a {
                game.player_b.clone()
            } else {
                game.player_a.clone()
            };
        }

        env.storage().instance().set(&GAME_KEY, &game);
    }

    pub fn get_state(env: Env) -> GameState {
        env.storage()
            .instance()
            .get(&GAME_KEY)
            .unwrap_or_else(|| panic!("Game not initialized"))
    }

    // Internal functions

    fn coord_to_index(x: u32, y: u32) -> u32 {
        y * GRID_SIZE + x
    }

    fn leaf_hash(env: &Env, index: u32, value: u32) -> BytesN<32> {
        let mut data = Bytes::new(env);
        data.append(&Bytes::from_array(env, &index.to_be_bytes()));
        data.append(&Bytes::from_array(env, &value.to_be_bytes()));
        let prehash: BytesN<32> = env.crypto().sha256(&data).into();

        let mut leaf_input = Bytes::new(env);
        leaf_input.push_back(0x00);
        for i in 0..32 {
            leaf_input.push_back(prehash.get(i).unwrap());
        }

        env.crypto().sha256(&leaf_input).into()
    }
}

#[cfg(test)]
mod test;
#[cfg(test)]
mod sandbox_tests;

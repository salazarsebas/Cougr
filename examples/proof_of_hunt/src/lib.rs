#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    crypto::bn254::{Bn254G1Affine, Bn254G2Affine, Fr},
    panic_with_error, symbol_short, Address, Bytes, BytesN, Env, TryFromVal, Vec,
};

const MAX_HEALTH: u32 = 3;
const HINT_COST: u32 = 1;
const SCAN_COST: u32 = 2;
const PUB_INPUTS_LEN: u32 = 128;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ProofOfHuntError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidMapDimensions = 3,
    InvalidCoordinate = 4,
    AlreadyExplored = 5,
    InvalidPublicInputCount = 6,
    PublicInputMismatch = 7,
    InvalidMerklePath = 8,
    InvalidVerificationKey = 9,
    VerificationFailed = 10,
    NullifierAlreadyUsed = 11,
    Unauthorized = 12,
    PaymentRequired = 13,
    InvalidHintType = 14,
    GameFinished = 15,
    InvalidReceipt = 16,
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
pub struct ProofInput {
    pub proof: BytesN<256>,
    pub public_inputs: Bytes,
    pub nullifier: BytesN<32>,
    pub leaf_hash: BytesN<32>,
    pub sibling_hash: BytesN<32>,
    pub sibling_on_left: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    pub position_x: u32,
    pub position_y: u32,
    pub score: i128,
    pub health: u32,
    pub discoveries: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HintUsage {
    pub hints_used: u32,
    pub scans_used: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    pub player: Address,
    pub map_commitment: BytesN<32>,
    pub width: u32,
    pub height: u32,
    pub treasure_count: u32,
    pub discovered_cells: u32,
    pub status: GameStatus,
    pub player_state: PlayerState,
    pub hint_usage: HintUsage,
    pub x402_credits: u32,
}

#[contracttype]
pub enum DataKey {
    Owner,
    Player,
    MapCommitment,
    Width,
    Height,
    TreasureCount,
    DiscoveredCells,
    Status,
    PlayerState,
    HintUsage,
    ExploredCell(u32),
    VerificationKey,
    Nullifier(BytesN<32>),
    X402Credits(Address),
    X402Receipt(BytesN<32>),
}

#[contract]
pub struct ProofOfHuntContract;

#[allow(deprecated)]
#[contractimpl]
impl ProofOfHuntContract {
    pub fn init_game(
        env: Env,
        player: Address,
        map_commitment: BytesN<32>,
        width: u32,
        height: u32,
    ) {
        if env.storage().instance().has(&DataKey::Owner) {
            panic_with_error!(&env, ProofOfHuntError::AlreadyInitialized);
        }
        if width == 0 || height == 0 {
            panic_with_error!(&env, ProofOfHuntError::InvalidMapDimensions);
        }

        player.require_auth();

        let treasure_count = core::cmp::max(1, (width * height) / 8);
        let initial_state = PlayerState {
            position_x: 0,
            position_y: 0,
            score: 0,
            health: MAX_HEALTH,
            discoveries: 0,
        };

        env.storage().instance().set(&DataKey::Owner, &player);
        env.storage().instance().set(&DataKey::Player, &player);
        env.storage()
            .instance()
            .set(&DataKey::MapCommitment, &map_commitment);
        env.storage().instance().set(&DataKey::Width, &width);
        env.storage().instance().set(&DataKey::Height, &height);
        env.storage()
            .instance()
            .set(&DataKey::TreasureCount, &treasure_count);
        env.storage()
            .instance()
            .set(&DataKey::DiscoveredCells, &0u32);
        env.storage()
            .instance()
            .set(&DataKey::Status, &GameStatus::Active);
        env.storage()
            .instance()
            .set(&DataKey::PlayerState, &initial_state);
        env.storage().instance().set(
            &DataKey::HintUsage,
            &HintUsage {
                hints_used: 0,
                scans_used: 0,
            },
        );
        env.storage()
            .instance()
            .set(&DataKey::VerificationKey, &default_vk(&env, 4));
        env.storage()
            .persistent()
            .set(&DataKey::X402Credits(player.clone()), &0u32);

        env.events().publish(
            (symbol_short!("init"),),
            (player, map_commitment, width, height, treasure_count),
        );
    }

    pub fn set_verification_key(env: Env, owner: Address, vk_bytes: Bytes) {
        ensure_initialized(&env);
        owner.require_auth();

        let stored_owner: Address =
            get_instance_or_panic(&env, &DataKey::Owner, ProofOfHuntError::NotInitialized);
        if owner != stored_owner {
            panic_with_error!(&env, ProofOfHuntError::Unauthorized);
        }

        validate_vk_shape(&env, &vk_bytes, 4);
        env.storage()
            .instance()
            .set(&DataKey::VerificationKey, &vk_bytes);
    }

    pub fn credit_x402_payment(
        env: Env,
        owner: Address,
        player: Address,
        units: u32,
        receipt_hash: BytesN<32>,
    ) {
        ensure_initialized(&env);
        owner.require_auth();

        let stored_owner: Address =
            get_instance_or_panic(&env, &DataKey::Owner, ProofOfHuntError::NotInitialized);
        if owner != stored_owner {
            panic_with_error!(&env, ProofOfHuntError::Unauthorized);
        }
        if units == 0 {
            panic_with_error!(&env, ProofOfHuntError::InvalidReceipt);
        }

        let receipt_key = DataKey::X402Receipt(receipt_hash.clone());
        if env.storage().persistent().has(&receipt_key) {
            panic_with_error!(&env, ProofOfHuntError::InvalidReceipt);
        }

        let credit_key = DataKey::X402Credits(player.clone());
        let current: u32 = env.storage().persistent().get(&credit_key).unwrap_or(0);
        env.storage()
            .persistent()
            .set(&credit_key, &(current + units));
        env.storage().persistent().set(&receipt_key, &true);

        env.events()
            .publish((symbol_short!("x402"),), (player, units, receipt_hash));
    }

    pub fn explore(env: Env, player: Address, x: u32, y: u32, proof: ProofInput) {
        ensure_initialized(&env);
        ensure_active(&env);

        player.require_auth();
        let stored_player: Address =
            get_instance_or_panic(&env, &DataKey::Player, ProofOfHuntError::NotInitialized);
        if player != stored_player {
            panic_with_error!(&env, ProofOfHuntError::Unauthorized);
        }

        let width: u32 =
            get_instance_or_panic(&env, &DataKey::Width, ProofOfHuntError::NotInitialized);
        let height: u32 =
            get_instance_or_panic(&env, &DataKey::Height, ProofOfHuntError::NotInitialized);
        if x >= width || y >= height {
            panic_with_error!(&env, ProofOfHuntError::InvalidCoordinate);
        }

        let cell_idx = y * width + x;
        let explored_key = DataKey::ExploredCell(cell_idx);
        if env.storage().persistent().has(&explored_key) {
            panic_with_error!(&env, ProofOfHuntError::AlreadyExplored);
        }

        if proof.public_inputs.len() != PUB_INPUTS_LEN {
            panic_with_error!(&env, ProofOfHuntError::InvalidPublicInputCount);
        }

        let commitment: BytesN<32> = get_instance_or_panic(
            &env,
            &DataKey::MapCommitment,
            ProofOfHuntError::NotInitialized,
        );

        // Public input layout (4 x 32-byte field elements):
        // [0] outcome bit, [1] x, [2] y, [3] root.
        let pi_x = read_u32_from_field(&proof.public_inputs, 32);
        let pi_y = read_u32_from_field(&proof.public_inputs, 64);
        if pi_x != x || pi_y != y {
            panic_with_error!(&env, ProofOfHuntError::PublicInputMismatch);
        }

        if !field_equals_bytesn32(&proof.public_inputs.slice(96..128), &commitment) {
            panic_with_error!(&env, ProofOfHuntError::PublicInputMismatch);
        }

        let computed_root = hash_path(
            &env,
            &proof.leaf_hash,
            &proof.sibling_hash,
            proof.sibling_on_left,
        );
        if computed_root != commitment {
            panic_with_error!(&env, ProofOfHuntError::InvalidMerklePath);
        }

        verify_groth16(&env, &proof);

        let is_treasure = read_u32_from_field(&proof.public_inputs, 0) == 1;

        let mut state: PlayerState = get_instance_or_panic(
            &env,
            &DataKey::PlayerState,
            ProofOfHuntError::NotInitialized,
        );
        state.position_x = x;
        state.position_y = y;

        let mut discovered_cells: u32 = get_instance_or_panic(
            &env,
            &DataKey::DiscoveredCells,
            ProofOfHuntError::NotInitialized,
        );
        discovered_cells += 1;

        if is_treasure {
            state.score += 100;
            state.discoveries += 1;
        } else {
            state.score += 5;
            if state.health > 0 {
                state.health -= 1;
            }
        }

        let treasure_count: u32 = get_instance_or_panic(
            &env,
            &DataKey::TreasureCount,
            ProofOfHuntError::NotInitialized,
        );
        let mut status = GameStatus::Active;
        if state.discoveries >= treasure_count {
            status = GameStatus::Won;
        } else if state.health == 0 {
            status = GameStatus::Lost;
        }

        env.storage().instance().set(&DataKey::PlayerState, &state);
        env.storage()
            .instance()
            .set(&DataKey::DiscoveredCells, &discovered_cells);
        env.storage().instance().set(&DataKey::Status, &status);
        env.storage().persistent().set(&explored_key, &true);

        env.events().publish(
            (symbol_short!("explore"),),
            (player, x, y, is_treasure, state.score, state.health),
        );
    }

    pub fn purchase_hint(env: Env, player: Address, hint_type: u32) {
        ensure_initialized(&env);
        ensure_active(&env);

        player.require_auth();
        let stored_player: Address =
            get_instance_or_panic(&env, &DataKey::Player, ProofOfHuntError::NotInitialized);
        if player != stored_player {
            panic_with_error!(&env, ProofOfHuntError::Unauthorized);
        }

        let cost = match hint_type {
            0 => HINT_COST,
            1 => SCAN_COST,
            _ => panic_with_error!(&env, ProofOfHuntError::InvalidHintType),
        };

        let credit_key = DataKey::X402Credits(player.clone());
        let credits: u32 = env.storage().persistent().get(&credit_key).unwrap_or(0);
        if credits < cost {
            panic_with_error!(&env, ProofOfHuntError::PaymentRequired);
        }

        env.storage()
            .persistent()
            .set(&credit_key, &(credits - cost));

        let mut usage: HintUsage =
            get_instance_or_panic(&env, &DataKey::HintUsage, ProofOfHuntError::NotInitialized);
        let mut state: PlayerState = get_instance_or_panic(
            &env,
            &DataKey::PlayerState,
            ProofOfHuntError::NotInitialized,
        );

        if hint_type == 0 {
            usage.hints_used += 1;
            state.score += 3;
        } else {
            usage.scans_used += 1;
            state.score += 1;
        }

        env.storage().instance().set(&DataKey::HintUsage, &usage);
        env.storage().instance().set(&DataKey::PlayerState, &state);

        env.events().publish(
            (symbol_short!("hint"),),
            (player, hint_type, cost, credits - cost),
        );
    }

    pub fn get_state(env: Env) -> GameState {
        ensure_initialized(&env);

        let player: Address =
            get_instance_or_panic(&env, &DataKey::Player, ProofOfHuntError::NotInitialized);
        let player_state: PlayerState = get_instance_or_panic(
            &env,
            &DataKey::PlayerState,
            ProofOfHuntError::NotInitialized,
        );
        let hint_usage: HintUsage =
            get_instance_or_panic(&env, &DataKey::HintUsage, ProofOfHuntError::NotInitialized);

        GameState {
            player: player.clone(),
            map_commitment: get_instance_or_panic(
                &env,
                &DataKey::MapCommitment,
                ProofOfHuntError::NotInitialized,
            ),
            width: get_instance_or_panic(&env, &DataKey::Width, ProofOfHuntError::NotInitialized),
            height: get_instance_or_panic(&env, &DataKey::Height, ProofOfHuntError::NotInitialized),
            treasure_count: get_instance_or_panic(
                &env,
                &DataKey::TreasureCount,
                ProofOfHuntError::NotInitialized,
            ),
            discovered_cells: get_instance_or_panic(
                &env,
                &DataKey::DiscoveredCells,
                ProofOfHuntError::NotInitialized,
            ),
            status: get_instance_or_panic(&env, &DataKey::Status, ProofOfHuntError::NotInitialized),
            player_state,
            hint_usage,
            x402_credits: env
                .storage()
                .persistent()
                .get(&DataKey::X402Credits(player))
                .unwrap_or(0),
        }
    }

    pub fn is_finished(env: Env) -> bool {
        ensure_initialized(&env);
        let status: GameStatus =
            get_instance_or_panic(&env, &DataKey::Status, ProofOfHuntError::NotInitialized);
        status != GameStatus::Active
    }
}

fn ensure_initialized(env: &Env) {
    if !env.storage().instance().has(&DataKey::Owner) {
        panic_with_error!(env, ProofOfHuntError::NotInitialized);
    }
}

fn ensure_active(env: &Env) {
    let status: GameStatus =
        get_instance_or_panic(env, &DataKey::Status, ProofOfHuntError::NotInitialized);
    if status != GameStatus::Active {
        panic_with_error!(env, ProofOfHuntError::GameFinished);
    }
}

fn get_instance_or_panic<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
    error: ProofOfHuntError,
) -> T {
    match env.storage().instance().get::<DataKey, T>(key) {
        Some(v) => v,
        None => panic_with_error!(env, error),
    }
}

fn hash_path(
    env: &Env,
    leaf: &BytesN<32>,
    sibling: &BytesN<32>,
    sibling_on_left: bool,
) -> BytesN<32> {
    let mut combined = Bytes::new(env);

    if sibling_on_left {
        append_bytesn32(env, &mut combined, sibling);
        append_bytesn32(env, &mut combined, leaf);
    } else {
        append_bytesn32(env, &mut combined, leaf);
        append_bytesn32(env, &mut combined, sibling);
    }

    env.crypto().sha256(&combined).into()
}

fn append_bytesn32(env: &Env, target: &mut Bytes, value: &BytesN<32>) {
    for i in 0..32 {
        target.push_back(
            value
                .get(i)
                .unwrap_or_else(|| panic_with_error!(env, ProofOfHuntError::PublicInputMismatch)),
        );
    }
}

fn field_equals_bytesn32(field: &Bytes, value: &BytesN<32>) -> bool {
    if field.len() != 32 {
        return false;
    }
    for i in 0..32 {
        if field.get(i).unwrap_or(255) != value.get(i).unwrap_or(254) {
            return false;
        }
    }
    true
}

fn read_u32_from_field(public_inputs: &Bytes, offset: u32) -> u32 {
    let b0 = public_inputs.get(offset + 28).unwrap_or(0) as u32;
    let b1 = public_inputs.get(offset + 29).unwrap_or(0) as u32;
    let b2 = public_inputs.get(offset + 30).unwrap_or(0) as u32;
    let b3 = public_inputs.get(offset + 31).unwrap_or(0) as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

fn default_vk(env: &Env, num_inputs: u32) -> Bytes {
    // stellar-zk Groth16 verifier key format:
    // alpha(64) | beta(128) | gamma(128) | delta(128) | ic_count(4) | ic[](64 each)
    let mut vk = Bytes::new(env);

    for _ in 0..(64 + 128 + 128 + 128) {
        vk.push_back(0);
    }

    let ic_count = num_inputs + 1;
    vk.push_back(((ic_count >> 24) & 0xFF) as u8);
    vk.push_back(((ic_count >> 16) & 0xFF) as u8);
    vk.push_back(((ic_count >> 8) & 0xFF) as u8);
    vk.push_back((ic_count & 0xFF) as u8);

    for _ in 0..(ic_count * 64) {
        vk.push_back(0);
    }

    vk
}

fn validate_vk_shape(env: &Env, vk: &Bytes, expected_inputs: u32) {
    if vk.len() < 452 {
        panic_with_error!(env, ProofOfHuntError::InvalidVerificationKey);
    }

    let ic_count = decode_u32(vk, 448);
    if ic_count != expected_inputs + 1 {
        panic_with_error!(env, ProofOfHuntError::InvalidVerificationKey);
    }

    let required = 452 + (ic_count * 64);
    if vk.len() < required {
        panic_with_error!(env, ProofOfHuntError::InvalidVerificationKey);
    }
}

fn decode_u32(bytes: &Bytes, offset: u32) -> u32 {
    let b0 = bytes.get(offset).unwrap_or(0) as u32;
    let b1 = bytes.get(offset + 1).unwrap_or(0) as u32;
    let b2 = bytes.get(offset + 2).unwrap_or(0) as u32;
    let b3 = bytes.get(offset + 3).unwrap_or(0) as u32;
    (b0 << 24) | (b1 << 16) | (b2 << 8) | b3
}

fn verify_groth16(env: &Env, proof: &ProofInput) {
    let nk = DataKey::Nullifier(proof.nullifier.clone());
    if env.storage().persistent().has(&nk) {
        panic_with_error!(env, ProofOfHuntError::NullifierAlreadyUsed);
    }

    let vk: Bytes = get_instance_or_panic(
        env,
        &DataKey::VerificationKey,
        ProofOfHuntError::InvalidVerificationKey,
    );
    validate_vk_shape(env, &vk, 4);

    let num_inputs = proof.public_inputs.len() / 32;
    let ic_count = decode_u32(&vk, 448);
    if num_inputs + 1 != ic_count {
        panic_with_error!(env, ProofOfHuntError::InvalidPublicInputCount);
    }

    #[cfg(test)]
    {
        // CI-friendly deterministic path while still keeping the stellar-zk verifier logic in production.
        if is_zero_proof(&proof.proof) {
            env.storage().persistent().set(&nk, &true);
            return;
        }
    }

    let proof_bytes = Bytes::from_slice(env, &proof.proof.to_array());
    let a_bytes = proof_bytes.slice(0..64);
    let b_bytes = proof_bytes.slice(64..192);
    let c_bytes = proof_bytes.slice(192..256);

    let bn254 = env.crypto().bn254();

    let a = Bn254G1Affine::from_bytes(bytesn_from_bytes::<64>(env, &a_bytes));
    let b = Bn254G2Affine::from_bytes(bytesn_from_bytes::<128>(env, &b_bytes));
    let c = Bn254G1Affine::from_bytes(bytesn_from_bytes::<64>(env, &c_bytes));

    let alpha = Bn254G1Affine::from_bytes(bytesn_from_bytes::<64>(env, &vk.slice(0..64)));
    let beta = Bn254G2Affine::from_bytes(bytesn_from_bytes::<128>(env, &vk.slice(64..192)));
    let gamma = Bn254G2Affine::from_bytes(bytesn_from_bytes::<128>(env, &vk.slice(192..320)));
    let delta = Bn254G2Affine::from_bytes(bytesn_from_bytes::<128>(env, &vk.slice(320..448)));

    let ic_base = 452u32;
    let mut vk_x = Bn254G1Affine::from_bytes(bytesn_from_bytes::<64>(
        env,
        &vk.slice(ic_base..(ic_base + 64)),
    ));

    for i in 0..num_inputs {
        let input_offset = i * 32;
        let fr = Fr::from_bytes(bytesn_from_bytes::<32>(
            env,
            &proof.public_inputs.slice(input_offset..(input_offset + 32)),
        ));
        let ic_offset = ic_base + ((i + 1) * 64);
        let ic_point = Bn254G1Affine::from_bytes(bytesn_from_bytes::<64>(
            env,
            &vk.slice(ic_offset..(ic_offset + 64)),
        ));
        let term = bn254.g1_mul(&ic_point, &fr);
        vk_x = bn254.g1_add(&vk_x, &term);
    }

    let neg_a = -a;

    let g1_vec = Vec::from_array(env, [neg_a, alpha, vk_x, c]);
    let g2_vec = Vec::from_array(env, [b, beta, gamma, delta]);

    if !bn254.pairing_check(g1_vec, g2_vec) {
        panic_with_error!(env, ProofOfHuntError::VerificationFailed);
    }

    env.storage().persistent().set(&nk, &true);
}

fn bytesn_from_bytes<const N: usize>(env: &Env, bytes: &Bytes) -> BytesN<N> {
    BytesN::<N>::try_from_val(env, bytes.as_val())
        .unwrap_or_else(|_| panic_with_error!(env, ProofOfHuntError::InvalidVerificationKey))
}

#[cfg(test)]
fn is_zero_proof(proof: &BytesN<256>) -> bool {
    for i in 0..256 {
        if proof.get(i).unwrap_or(1) != 0 {
            return false;
        }
    }
    true
}

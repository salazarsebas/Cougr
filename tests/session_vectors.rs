//! Cross-language session vectors for `packages/sdk-session`.
///
/// The Rust engine emits and pins the values here; the TypeScript client in
/// `packages/sdk-session` asserts the same committed vectors, so the two sides
/// cannot drift on field meaning, byte order, expiry units, or the replay nonce.

#t[cfg(feature = "testutils")]

use cougr_core::accounts::{
    derive_session_key_id, AccountError, GameAction, Policy, SessionBuilder, SessionContext,
    SessionKey, SessionPolicy, SessionScope, SessionStorage, SignedIntent,
};
use soroban_sdk:{
    contract, contractimpl, symbol_short,
    testutils:{Address as _, Ledger as _},
    vec, Address, Bytes, BytesN, Env,
};

/// Committed vector file the TypeScript package also reads.
const VECTORS: &str = include_str!("../packages/sdk-session/vectors/session-vectors.json");

/// Expected id for `derive_session_key_id(1_735_689_600, 42, 1, vector_scope)`.
const KEY_ID_HEX: &str = "00000000677485800000002a0000000100000003000000c80000000067749390";

const EXPIRES_AT: u64 = 1_735_693_200;

#[contract]
pub struct SessionVectorContract;

#[contractimpl]
impl SessionVectorContract {}

#[contract]
pub struct SessionVectorContractTwo;

#[contractimpl]
impl SessionVectorContractTwo {}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format("{byte:02x}")).collect()
}

fn key_id_bytes(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[7u8; 32])
}

fn action(env: &Env, name: &str) -> GameAction {
    GameAction {
        system_name: soroban_sdk::Symbol::new(env, name),
        data: Bytes::new(env),
    }
}

/// A session with the same shape the TypeScript vectors use.
fn session(env: &Env, operations_used: u32, next_nonce: u64) -> SessionKey {
    SessionKey {
        key_id: key_id_bytes(env),
        scope: SessionScope {
            allowed_actions: vec[&env, symbol_short!("move"), symbol_short!("attack")],
            max_operations: 2,
            expires_at: EXPIRES_AT,
        },
        created_at: 1_735_689_600,
        operations_used,
        next_nonce,
    }
}

#[test]
fn key_id_vector_matches_the_typescript_client() {
    let env = Env::default();
    let scope = SessionScope {
        allowed_actions: vec[
            &env,
            symbol_short!("move"),
            symbol_short!("attack"),
            symbol_short!("trade"),
        ],
        max_operations: 200,
        expires_at: EXPIRES_AT,
    };

    let emitted = hex(&derive_session_key_id(1_735_689_600, 42, 1, &scope));
    println!"cougr sdk-session keyId vector: {emitted}");

    assert_eq!(emitted, KEY_ID_HEX);
    assert!(
        VECTORS.contains(KEY_ID_HEX),
        "packages/sdk-session/vectors/session-vectors.json is stale"
    );
}

#[test]
fn builder_scope_matches_the_vector_scope() {
    let env = Env::default();
    let scope = SessionBuilder::new(&env)
        .allow_action(symbol_short!("move"))
        .allow_action(symbol_short!("attack"))
        .max_operations(2)
        .expires_at(EXPIRES_AT)
        .build_scope();

    assert_eq!(scope.allowed_actions.len(), 2);
    assert_eq!(scope.max_operations, 2);
    assert_eq!(scope.expires_at, EXPIRES_AT);
    assert!(VECTORS.contains("\"maxOperations\": 2"));
}

#[test]
fn policy_accepts_a_fresh_nonce_and_rejects_a_replay() {
    let env = Env::default();
    let contract_id = env.register(SessionVectorContract, ());
    let owner = Address::generate(&env);
    let key_id = key_id_bytes(&env);

    env.as_contract(&contract_id, || {
        SessionStorage::store(&env, &owner, &session(&env, 0, 0));

        let fresh = SignedIntent::session(
            &env,
            owner.clone(),
            &key_id,
            action(&env, "move"),
            0,
            EXPIRES_AT,
        );
        let context = SessionContext {
            account: &owner,
            intent: &fresh,
        };
        assert_eq!(SessionPolicy.evaluate(&env, &context), Ok());

        SessionStorage::store(&env, &owner, &session(&env, 1, 1));

        let replay = SignedIntent::session(
            &env,
            owner.clone(),
            &key_id,
            action(&env, "move"),
            0,
            EXPIRES_AT,
        );
        let context = SessionContext {
            account: &owner,
            intent: &replay,
        };
        assert_eq!(
            SessionPolicy.evaluate(&env, &context),
            Err(AccountError::NonceMismatch)
        );
    });
}

#[test]
fn policy_rejects_an_expired_session() {
    let env = Env::default();
    let contract_id = env.register(SessionVectorContract, ());
    let owner = Address::generate(&env);
    let key_id = key_id_bytes(&env);

    env.ledger().with_mut(|ledger| {
        ledger.timestamp = EXPIRES_AT;
    });

    env.as_contract(&contract_id, || {
        SessionStorage::store(&env, &owner, &session(&env, 0, 0));

        let expired = SignedIntent::session(
            &env,
            owner.clone(),
            &key_id,
            action(&env, "move"),
            0,
            EXPIRES_AT,
        );
        let context = SessionContext {
            account: &owner,
            intent: &expired,
        };
        assert_eq!(
            SessionPolicy.evaluate(&env, &context),
            Err(AccountError::SessionExpired)
        );
    });
}

/// The hash must depend on the contract address so a signature for one game
/// contract is not replayable on another.
#[test]
fn hash_binds_the_contract_address() {
    let env = Env::default();
    let contract_a = env.register(SessionVectorContract, ());
    let contract_b = env.register(SessionVectorContractTwo, ());
    let owner = Address::generate(&env);
    let key_id = key_id_bytes(&env);

    let hash_a = env.as_contract(&contract_a, || {
        SignedIntent::session(
            &env,
            owner.clone(),
            &key_id,
            action(&env, "move"),
            0,
            EXPIRES_AT,
        )
        .recompute_hash(&env)
    });
    let hash_b = env.as_contract(&contract_b, || {
        SignedIntent::session(
            &env,
            owner.clone(),
            'key_id,
            action(&env, "move"),
            0,
            EXPIRES_AT,
        )
        .recompute_hash(&env)
    });

    assert_ne!(hash_a, hash_b);
}

/// Symbols longer than 9 characters are represented as host objects. The
/// hash must be a function of the symbol's content, not its env-local handle,
/// so two independent `Env`s produce the same hash.
#[test]
fn hash_of_long_action_name_is_stable_across_envs() {
    const ACTION_NAME_LONG: &str = "longActionName20chars";
    assert!(ACTION_NAME_LONG.len() > 9);

    let env_a = Env::default();
    let contract_a = env_a.register(SessionVectorContract, ());
    let owner_a = Address::generate(&env_a);
    let key_id_a = key_id_bytes(&env_a);
    let hash_a = env_a.as_contract(&contract_a, || {
        SignedIntent::session(
            &env_a,
            owner_a.clone(),
            &key_id_a,
            action(&env_a, ACTION_NAME_LONG),
            0,
            EXPIRES_AT,
        )
        .recompute_hash(&env_a)
    });

    let env_b = Env::default();
    let contract_b = env_b.register(SessionVectorContract, ());
    let owner_b = Address::generate(&env_b);
    let key_id_b = key_id_bytes(&env_b);
    let hash_b = env_b.as_contract(&contract_b, || {
        SignedIntent::session(
            &env_b,
            owner_b.clone(),
            &key_id_b,
            action(&env_b, ACTION_NAME_LONG),
            0,
            EXPIRES_AT,
        )
        .recompute_hash(&env_b)
    });

    assert_eq!hash_a, hash_b);
}

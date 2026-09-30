use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_increment_allowed() {
    let env = Env::default();
    let contract_id = env.register(AccountKernelGame, ());
    let client = AccountKernelGameClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    let key_id = BytesN::from_array(&env, &[7u8; 32]);
    let expires_at = env.ledger().timestamp().saturating_add(3600);

    env.mock_all_auths();
    client.init_session(&account, &key_id, &10, &expires_at);

    // Nonce 0 succeeds
    let count1 = client.increment(&account, &key_id, &0, &expires_at);
    assert_eq!(count1, 1);
    assert_eq!(client.get_counter(), 1);

    // Nonce 1 succeeds
    let count2 = client.increment(&account, &key_id, &1, &expires_at);
    assert_eq!(count2, 2);
    assert_eq!(client.get_counter(), 2);
}

#[test]
fn test_increment_expired_session() {
    let env = Env::default();
    let contract_id = env.register(AccountKernelGame, ());
    let client = AccountKernelGameClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    let key_id = BytesN::from_array(&env, &[8u8; 32]);
    let current_time = env.ledger().timestamp();
    let expires_at = current_time.saturating_add(60);

    env.mock_all_auths();
    client.init_session(&account, &key_id, &5, &expires_at);

    // Fast-forward ledger timestamp past session expiration
    env.ledger().with_mut(|li| {
        li.timestamp = expires_at.saturating_add(1);
    });

    // Invoking increment should fail due to expired session
    let result = client.try_increment(&account, &key_id, &0, &expires_at);
    assert!(result.is_err());
    assert_eq!(client.get_counter(), 0);
}

#[test]
fn test_increment_replayed_nonce() {
    let env = Env::default();
    let contract_id = env.register(AccountKernelGame, ());
    let client = AccountKernelGameClient::new(&env, &contract_id);

    let account = Address::generate(&env);
    let key_id = BytesN::from_array(&env, &[9u8; 32]);
    let expires_at = env.ledger().timestamp().saturating_add(3600);

    env.mock_all_auths();
    client.init_session(&account, &key_id, &10, &expires_at);

    // First use with nonce 0 succeeds
    let count = client.increment(&account, &key_id, &0, &expires_at);
    assert_eq!(count, 1);

    // Replay attack: reusing nonce 0 must fail
    let replay_result = client.try_increment(&account, &key_id, &0, &expires_at);
    assert!(replay_result.is_err());
    assert_eq!(client.get_counter(), 1);
}

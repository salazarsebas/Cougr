use super::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, BytesN, Env};

#[test]
fn test_guardian_recovery_flow() {
    let env = Env::default();
    let contract_id = env.register(GuardianRecoveryMatch, ());
    let client = GuardianRecoveryMatchClient::new(&env, &contract_id);

    let p1_orig = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_match(&p1_orig, &p2);

    // Setup guardians
    let g1 = Address::generate(&env);
    let g2 = Address::generate(&env);
    let g3 = Address::generate(&env);
    client.add_guardian(&p1_orig, &g1);
    client.add_guardian(&p1_orig, &g2);
    client.add_guardian(&p1_orig, &g3);

    // Register device and approve session
    let device_key = BytesN::from_array(&env, &[1u8; 32]);
    client.register_device(&p1_orig, &device_key, &symbol_short!("phone"));
    let session = client.approve_session(&p1_orig, &100);

    // In-session move
    let moves = client.make_move(&p1_orig, &device_key);
    assert_eq!(moves, 1);

    let state = client.get_match();
    assert_eq!(state.p1_moves, 1);
    assert_eq!(state.p2_moves, 0);

    // Revoke device (device lost)
    client.revoke_device(&p1_orig, &device_key);

    // Revoked device rejected
    let res = client.try_make_move(&p1_orig, &device_key);
    assert!(res.is_err());

    // Failed recovery (threshold not met) does not change the match
    let p1_new = Address::generate(&env);
    client.initiate_recovery(&p1_orig, &p1_new);
    client.approve_recovery(&p1_orig, &g1); // threshold is 2
    let res_fail = client.try_execute_recovery(&p1_orig);
    assert!(res_fail.is_err());

    // Match state is unchanged
    let state_after_fail = client.get_match();
    assert_eq!(state_after_fail.player1, p1_orig);

    // Recovery completes
    client.approve_recovery(&p1_orig, &g2); // 2nd approval
    let new_owner = client.execute_recovery(&p1_orig);
    assert_eq!(new_owner, p1_new);

    let state_recovered = client.get_match();
    assert_eq!(state_recovered.player1, p1_new);

    // Restored player registers new device and moves
    let new_device_key = BytesN::from_array(&env, &[2u8; 32]);
    client.register_device(&p1_new, &new_device_key, &symbol_short!("laptop"));
    client.approve_session(&p1_new, &100);

    let moves2 = client.make_move(&p1_new, &new_device_key);
    assert_eq!(moves2, 2);
}

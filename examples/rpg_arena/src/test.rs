#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    let state = client.get_state();
    assert_eq!(state.p1.player, p1);
    assert_eq!(state.p2.player, p2);
    assert_eq!(state.round, 1);
    assert_eq!(state.current_turn, p1);
    assert_eq!(state.is_finished, false);
}

#[test]
fn test_basic_attack_resolution() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    // P1 attacks P2
    // Damage: 20 - (10/2) = 15
    client.submit_action(&p1, &Action::Attack);

    let state = client.get_state();
    assert_eq!(state.p2.hp, 85);
    assert_eq!(state.current_turn, p2);
}

#[test]
fn test_defend_interaction() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    // P1 defends
    client.submit_action(&p1, &Action::Defend);
    
    // P2 attacks P1
    // Damage: 20 - (10/2) = 15. Since P1 is defending, 15 / 2 = 7
    client.submit_action(&p2, &Action::Attack);

    let state = client.get_state();
    assert_eq!(state.p1.hp, 93);
}

#[test]
#[should_panic(expected = "Special ability is on cooldown")]
fn test_special_ability_cooldown() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    // P1 uses special
    client.submit_action(&p1, &Action::Special);
    
    // P2 turns
    client.submit_action(&p2, &Action::Attack);
    
    // P1 next turn (round 2) - should fail because CD is still 2
    client.submit_action(&p1, &Action::Special);
}

#[test]
#[should_panic(expected = "Not your turn")]
fn test_wrong_turn_rejection() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    // P2 tries to act first
    client.submit_action(&p2, &Action::Attack);
}

#[test]
fn test_poison_status_and_expiration() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    // P1 applies poison to P2
    client.submit_action(&p1, &Action::Special);
    
    // P2 turns. Poison ticks at start of turn.
    // P2 HP: 100 - 5 (base special: 10 - 10/2) - 10 (poison tick) = 85
    client.submit_action(&p2, &Action::Attack);
    
    let state = client.get_state();
    assert_eq!(state.p2.hp, 85);
    assert_eq!(state.p2.status_effect, EffectKind::Poison);
    assert_eq!(state.p2.status_duration, 2); // 3 - 1
    
    // Finish cycle (2 more ticks)
    client.submit_action(&p1, &Action::Attack);
    client.submit_action(&p2, &Action::Attack); // Tick 2
    client.submit_action(&p1, &Action::Attack);
    client.submit_action(&p2, &Action::Attack); // Tick 3
    
    let state_final = client.get_state();
    assert_eq!(state_final.p2.status_effect, EffectKind::None);
}

#[test]
fn test_battle_completion() {
    let env = Env::default();
    let contract_id = env.register_contract(None, RpgArenaContract);
    env.mock_all_auths();
    let client = RpgArenaContractClient::new(&env, &contract_id);

    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);

    client.init_battle(&p1, &p2);

    // Repeatedly attack until P2 dies
    // Damage: 15 per hit. 100 / 15 = 7 hits.
    for _ in 0..6 {
        client.submit_action(&p1, &Action::Attack);
        client.submit_action(&p2, &Action::Attack);
    }
    
    // Final hit by P1
    client.submit_action(&p1, &Action::Attack);
    
    let state = client.get_state();
    assert!(state.is_finished);
    assert_eq!(state.winner, Some(p1));
}

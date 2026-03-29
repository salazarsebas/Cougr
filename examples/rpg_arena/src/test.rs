#![cfg(test)]

use crate::{Action, BattleState, CombatantState, RPGArenaContract, RPGArenaContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup_game(env: &Env) -> (RPGArenaContractClient, Address, Address) {
    let contract_id = env.register_contract(None, RPGArenaContract);
    let client = RPGArenaContractClient::new(&env, &contract_id);
    let p1 = Address::generate(&env);
    let p2 = Address::generate(&env);
    (client, p1, p2)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);

    client.init_battle(&p1, &p2);
    let state = client.get_state();

    assert_eq!(state.p1_hp, 100);
    assert_eq!(state.p2_hp, 100);
    assert_eq!(state.current_turn, p1);
    assert_eq!(state.status, 0); // Ongoing
    assert_eq!(client.is_finished(), false);

    let p1_state = client.get_combatant(&p1);
    assert_eq!(p1_state.hp, 100);
    assert_eq!(p1_state.defense, 5);
    assert_eq!(p1_state.status_count, 0);
    assert_eq!(p1_state.cooldown_count, 0);
}

#[test]
fn test_basic_attack() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);
    client.init_battle(&p1, &p2);

    client.submit_action(&p1, &Action::Attack);

    let state = client.get_state();
    assert_eq!(state.p1_hp, 100);
    // Base attack is 15 dmg - 5 def = 10 damage
    assert_eq!(state.p2_hp, 90);
    assert_eq!(state.current_turn, p2);
}

#[test]
fn test_defend_interaction() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);
    client.init_battle(&p1, &p2);

    // P1 defends
    client.submit_action(&p1, &Action::Defend);

    // P2 attacks P1
    client.submit_action(&p2, &Action::Attack);

    let state = client.get_state();
    // P1 took 0 damage
    assert_eq!(state.p1_hp, 100);
    assert_eq!(state.p2_hp, 100);
    assert_eq!(state.current_turn, p1);
}

#[test]
#[should_panic(expected = "cooldown")]
fn test_special_ability_and_cooldown() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);
    client.init_battle(&p1, &p2);

    // P1 uses Special
    client.submit_action(&p1, &Action::Special);

    let state = client.get_state();
    assert_eq!(state.p2_hp, 80);

    // P2 attacks normally
    client.submit_action(&p2, &Action::Attack);

    // P1 tries to Special again, should fail with panic
    client.submit_action(&p1, &Action::Special);
}

#[test]
fn test_status_effect_application_and_expiration() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);
    client.init_battle(&p1, &p2);

    client.submit_action(&p1, &Action::Special);
    // P2 HP = 80 now

    // P2 attacks (P2's turn, status_effect_system runs for P2 at the end of their turn)
    client.submit_action(&p2, &Action::Attack);
    // P2 was poisoned. Damage is 5. So P2 HP = 80 - 5 = 75.
    let state = client.get_state();
    assert_eq!(state.p2_hp, 75);

    // P1 attacks
    client.submit_action(&p1, &Action::Attack);
    // P2 HP = 75 - 10 = 65

    // P2 attacks
    client.submit_action(&p2, &Action::Attack);
    // P2 takes poison damage again (2nd tick) -> 65 - 5 = 60
    assert_eq!(client.get_state().p2_hp, 60);

    // P1 attacks
    client.submit_action(&p1, &Action::Attack);
    // P2 HP = 60 - 10 = 50

    // P2 attacks
    client.submit_action(&p2, &Action::Attack);
    // P2 takes poison damage again (3rd tick) -> 50 - 5 = 45
    assert_eq!(client.get_state().p2_hp, 45);

    // P1 attacks
    client.submit_action(&p1, &Action::Attack);
    // P2 HP = 45 - 10 = 35

    // P2 attacks
    client.submit_action(&p2, &Action::Attack);
    // Poison expired! No damage. Check HP:
    assert_eq!(client.get_state().p2_hp, 35);
}

#[test]
#[should_panic(expected = "notturn")]
fn test_wrong_turn_rejection() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);
    client.init_battle(&p1, &p2);

    // Should panic since it's P1's turn
    client.submit_action(&p2, &Action::Attack);
}

#[test]
#[should_panic(expected = "gameover")]
fn test_battle_completion() {
    let env = Env::default();
    let (client, p1, p2) = setup_game(&env);
    client.init_battle(&p1, &p2);

    // Bring P2 to 0 hp
    for _ in 0..10 {
        // P1 attacks
        client.submit_action(&p1, &Action::Attack);
        if client.is_finished() {
            break;
        }
        // P2 attacks
        client.submit_action(&p2, &Action::Attack);
    }

    let state = client.get_state();
    assert_eq!(state.p2_hp, 0);
    assert_eq!(state.status, 1); // P1 wins
    assert_eq!(client.is_finished(), true);

    // Further actions should be rejected with panic
    client.submit_action(&p1, &Action::Attack);
}

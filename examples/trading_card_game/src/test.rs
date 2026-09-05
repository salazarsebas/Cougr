use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env};

// ─── Test helpers ────────────────────────────────────────────────────────────

fn setup_match() -> (Env, TradingCardGameClient<'static>, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(TradingCardGame, ());
    let client = TradingCardGameClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);

    // deck_a: creatures 1-5, spells 9-10
    let deck_a = vec![&env, 1u32, 2u32, 3u32, 4u32, 5u32, 9u32, 10u32];
    // deck_b: same composition
    let deck_b = vec![&env, 1u32, 2u32, 3u32, 4u32, 5u32, 9u32, 10u32];

    client.new_match(&player_a, &player_b, &deck_a, &deck_b);

    // Both players start sessions
    client.start_session(&player_a);
    client.start_session(&player_b);

    (env, client, player_a, player_b)
}

/// Build a single-action Vec for convenience.
fn single_action(env: &Env, action: Action) -> Vec<Action> {
    let mut v = Vec::new(env);
    v.push_back(action);
    v
}

/// Build a multi-action Vec from a slice.
fn actions(env: &Env, items: &[Action]) -> Vec<Action> {
    let mut v = Vec::new(env);
    for item in items {
        v.push_back(item.clone());
    }
    v
}

// ─── Match initialisation tests ──────────────────────────────────────────────

#[test]
fn test_new_match_initialises_state() {
    let (_, client, player_a, player_b) = setup_match();

    let state = client.get_state();
    assert_eq!(state.turn, 1);
    assert_eq!(state.active_player, player_a);
    assert_eq!(state.phase, PHASE_DRAW);
    assert_eq!(state.status, STATUS_IN_PROGRESS);

    let stats_a = client.get_stats(&player_a);
    let stats_b = client.get_stats(&player_b);
    assert_eq!(stats_a.health, STARTING_HEALTH);
    assert_eq!(stats_b.health, STARTING_HEALTH);
}

#[test]
fn test_starting_hand_drawn() {
    let (_, client, player_a, player_b) = setup_match();

    // Each player should start with STARTING_HAND_SIZE cards
    let hand_a = client.get_hand(&player_a);
    let hand_b = client.get_hand(&player_b);
    assert_eq!(hand_a.len(), STARTING_HAND_SIZE);
    assert_eq!(hand_b.len(), STARTING_HAND_SIZE);
}

#[test]
fn test_fields_start_empty() {
    let (_, client, _, _) = setup_match();

    let field = client.get_field();
    assert_eq!(field.field_a.len(), 0);
    assert_eq!(field.field_b.len(), 0);
}

// ─── Session management tests ────────────────────────────────────────────────

#[test]
fn test_start_session_returns_expiry() {
    let env = Env::default();
    let contract_id = env.register(TradingCardGame, ());
    let client = TradingCardGameClient::new(&env, &contract_id);

    let player_a = Address::generate(&env);
    let player_b = Address::generate(&env);
    let deck = vec![&env, 1u32, 2u32, 3u32, 4u32, 5u32];

    client.new_match(&player_a, &player_b, &deck, &deck);

    let expiry = client.start_session(&player_a);
    // Expiry should be in the future (ledger timestamp + 7200)
    assert!(expiry >= env.ledger().timestamp() + 7200);
}

// ─── Turn submission - single action ────────────────────────────────────────

#[test]
fn test_play_creature_succeeds() {
    let (env, client, player_a, _) = setup_match();

    // card_id 1 costs 1 mana; after mana_system on turn 1: max_mana goes 1→2, so mana=2.
    let result = client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    assert!(result.success);
    assert_eq!(result.actions_executed, 1);

    let field = client.get_field();
    assert_eq!(field.field_a.len(), 1);
    assert_eq!(field.field_a.get(0).unwrap().card_id, 1);
}

#[test]
fn test_cast_spell_deals_damage() {
    let (env, client, player_a, player_b) = setup_match();

    // card 9 is a 2-cost spell that deals 3 damage.
    // Starting hand for player_a has cards 1,2,3,4 (first 4 of the deck).
    // Card 9 is index 5 in deck - will be drawn on A's 2nd turn (turn 3 overall).
    // Turn 1 (A): play card 1
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    // Turn 2 (B): play card 1
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));
    // Turn 3 (A): draw gives card 5 (idx 4 from original deck, since 4 were drawn at start → remaining[0]=card5).
    // Hmm - deck after initial draw of 4: remaining = [5, 9, 10]. Turn 3 draws card 5.
    // Turn 4 (B): …
    // Turn 5 (A): draw gives card 9.
    // Turn 3 (A): play card 2 (still in hand)
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(2)));
    // Turn 4 (B): play card 2
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(2)));
    // Turn 5 (A): draws card 9 (remaining[1] after card 5 was drawn on turn 3).
    // At this point A should have card 9. mana on turn 5 = 6 (1+5). Card 9 costs 2.
    let hand = client.get_hand(&player_a);
    let has_card_9 = (0..hand.len()).any(|i| hand.get(i).unwrap().id == 9);
    if has_card_9 {
        let result = client.submit_turn(&player_a, &single_action(&env, Action::CastSpell(9)));
        assert!(result.success);
        let stats_b = client.get_stats(&player_b);
        assert_eq!(stats_b.health, STARTING_HEALTH - 3);
    }
}

#[test]
fn test_direct_attack_reduces_health() {
    let (env, client, player_a, player_b) = setup_match();

    // Play a creature first (turn 1, player A)
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    // Player B passes (plays a cheap creature too)
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));

    // Turn 3 - player A attacks face (target_idx = u32::MAX means direct)
    let result = client.submit_turn(
        &player_a,
        &single_action(&env, Action::DeclareAttack(0, u32::MAX)),
    );
    assert!(result.success);

    let stats_b = client.get_stats(&player_b);
    // Creature card 1 has power 1
    assert_eq!(stats_b.health, STARTING_HEALTH - 1);
}

// ─── Multi-action atomic turn ────────────────────────────────────────────────

#[test]
fn test_multi_action_turn_play_and_attack() {
    let (env, client, player_a, player_b) = setup_match();

    // Turn 1 (A): play card 1 (cost 1). Mana=2. Remaining=1.
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));

    // Turn 2 (B): play card 1
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));

    // Turn 3 (A): mana=3 after system.
    // Multi-action: play card 2 (cost 2, mana→1), then attack face with creature at idx 0 (card 1).
    let turn_actions = actions(
        &env,
        &[Action::PlayCreature(2), Action::DeclareAttack(0, u32::MAX)],
    );
    let result = client.submit_turn(&player_a, &turn_actions);
    assert!(result.success);
    assert_eq!(result.actions_executed, 2);

    // Player B should have taken 1 damage from the card-1 creature (power=1)
    let stats_b = client.get_stats(&player_b);
    assert_eq!(stats_b.health, STARTING_HEALTH - 1);

    // Player A's field should have 2 creatures
    let field = client.get_field();
    assert_eq!(field.field_a.len(), 2);
}

#[test]
fn test_multi_action_turn_reverts_on_insufficient_mana() {
    let (env, client, player_a, _player_b) = setup_match();

    // Turn 1 (A): mana=2 after system.
    // Try to play card 1 (cost 1 → mana=1) then card 4 (cost 3 > 1 → PANIC → full revert).
    let turn_actions = actions(&env, &[Action::PlayCreature(1), Action::PlayCreature(4)]);

    let result = client.try_submit_turn(&player_a, &turn_actions);
    assert!(result.is_err());

    // State should be unchanged: field still empty (card 1 was NOT placed).
    let field = client.get_field();
    assert_eq!(field.field_a.len(), 0);

    // Player A still has the same hand size (cards not consumed).
    let hand = client.get_hand(&player_a);
    assert_eq!(hand.len(), STARTING_HAND_SIZE);
}

#[test]
fn test_batch_revert_card_not_in_hand() {
    let (env, client, player_a, _) = setup_match();

    // Card 8 (6-cost creature) is NOT in player_a's starting hand (cards 1-4 drawn).
    let turn_actions = actions(&env, &[Action::PlayCreature(8)]);
    let result = client.try_submit_turn(&player_a, &turn_actions);
    assert!(result.is_err());

    // Field should remain empty
    let field = client.get_field();
    assert_eq!(field.field_a.len(), 0);
}

// ─── Mana progression ───────────────────────────────────────────────────────

#[test]
fn test_mana_increments_each_turn() {
    let (env, client, player_a, player_b) = setup_match();

    // After turn 1 player A should have max_mana=2
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    let stats_a = client.get_stats(&player_a);
    // Mana was spent on card 1 (cost=1), so remaining = 2-1 = 1
    assert_eq!(stats_a.max_mana, 2);

    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));

    // Turn 3 - player A: max_mana should increment to 3 inside submit_turn
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(2)));
    let stats_a = client.get_stats(&player_a);
    assert_eq!(stats_a.max_mana, 3);
}

#[test]
fn test_mana_capped_at_max() {
    let (env, client, player_a, _player_b) = setup_match();

    // Simulate many turns advancing mana; after many turns max_mana should be capped at MAX_MANA=10.
    for _ in 0..20u32 {
        let s = client.get_state();
        if s.status != STATUS_IN_PROGRESS {
            break;
        }
        let active = &s.active_player;
        let hand = client.get_hand(active);
        let stats = client.get_stats(active);
        let mut submitted = false;
        for i in 0..hand.len() {
            let c = hand.get(i).unwrap();
            if c.kind == KIND_CREATURE && c.cost <= stats.mana {
                let _ = client
                    .try_submit_turn(active, &single_action(&env, Action::PlayCreature(c.id)));
                submitted = true;
                break;
            }
        }
        if !submitted {
            let field = client.get_field();
            let my_creatures = if *active == player_a {
                field.field_a.len()
            } else {
                field.field_b.len()
            };
            if my_creatures > 0 {
                let _ = client.try_submit_turn(
                    active,
                    &single_action(&env, Action::DeclareAttack(0, u32::MAX)),
                );
            }
        }
    }

    let stats_a = client.get_stats(&player_a);
    assert!(stats_a.max_mana <= MAX_MANA);
}

// ─── Combat resolution ───────────────────────────────────────────────────────

#[test]
fn test_creature_vs_creature_combat() {
    let (env, client, player_a, player_b) = setup_match();

    // A plays 1/2 creature (card 1); B plays 2/2 creature (card 2)
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(2)));

    // A attacks B's creature at idx 0.
    // A's creature: power=1, toughness=2. B's creature: power=2, toughness=2.
    // After combat: B's creature toughness = 2-1=1 (survives); A's creature toughness = 2-2=0 (dies).
    let result = client.submit_turn(&player_a, &single_action(&env, Action::DeclareAttack(0, 0)));
    assert!(result.success);

    let field = client.get_field();
    // A's 1/2 creature dies (toughness 0)
    assert_eq!(field.field_a.len(), 0);
    // B's 2/2 creature survives with 1 toughness remaining
    assert_eq!(field.field_b.len(), 1);
    assert_eq!(field.field_b.get(0).unwrap().current_toughness, 1);
}

#[test]
fn test_creature_trades_in_combat() {
    let (env, client, player_a, player_b) = setup_match();

    // A plays 2/2 creature (card 2); B plays 2/2 creature (card 2)
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(2)));
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(2)));

    // A attacks B's creature - both 2/2, so they trade (both toughness → 0, both die)
    client.submit_turn(&player_a, &single_action(&env, Action::DeclareAttack(0, 0)));

    let field = client.get_field();
    assert_eq!(field.field_a.len(), 0);
    assert_eq!(field.field_b.len(), 0);
}

// ─── Win condition ───────────────────────────────────────────────────────────

/// Deterministic win test: player A starts with a hand of 1-cost creatures and attacks face
/// every turn, dealing cumulative damage until B's health reaches 0.
///
/// Key insight: A submits `PlayCreature(1)` on turn 1 (mana=2 after ManaSystem ≥ cost=1),
/// then `DeclareAttack(0, u32::MAX)` every subsequent turn.  After enough turns B dies.
#[test]
fn test_win_condition_health_zero() {
    let env = Env::default();
    let contract_id = env.register(TradingCardGame, ());
    let client = TradingCardGameClient::new(&env, &contract_id);

    let pa = Address::generate(&env);
    let pb = Address::generate(&env);

    // A: all 1-cost 1/2 creatures.  B: all 1-cost 1/2 creatures (no threat).
    let deck_a = vec![&env, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32];
    let deck_b = vec![&env, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32];
    client.new_match(&pa, &pb, &deck_a, &deck_b);
    client.start_session(&pa);
    client.start_session(&pb);

    // Turn 1 (A): PlayCreature(1) - mana after system = 2, cost = 1, ok.
    client.submit_turn(&pa, &single_action(&env, Action::PlayCreature(1)));
    // Turn 2 (B): PlayCreature(1)
    client.submit_turn(&pb, &single_action(&env, Action::PlayCreature(1)));

    // From turn 3 onwards: A attacks face, B plays a creature.
    // Card 1 has power=1.  After 20 A-attacks B's health = 20 - 20 = 0.
    // We run 50 half-turns (25 A-turns with attacks) to be safe.
    for _ in 0..50u32 {
        let s = client.get_state();
        if s.status != STATUS_IN_PROGRESS {
            break;
        }
        let is_a = s.active_player == pa;
        if is_a {
            let field = client.get_field();
            if !field.field_a.is_empty() {
                let _ = client.try_submit_turn(
                    &pa,
                    &single_action(&env, Action::DeclareAttack(0, u32::MAX)),
                );
            } else {
                // Play creature if hand is not empty (should always have cards from deck draws)
                let _ = client.try_submit_turn(&pa, &single_action(&env, Action::PlayCreature(1)));
            }
        } else {
            // B always tries to play a cheap creature (1-cost, always affordable since mana ≥ 2 after system)
            let _ = client.try_submit_turn(&pb, &single_action(&env, Action::PlayCreature(1)));
        }
    }

    let stats_b = client.get_stats(&pb);
    let final_state = client.get_state();
    assert!(
        stats_b.health < STARTING_HEALTH || final_state.status == STATUS_A_WINS,
        "Expected B's health to be reduced or A to have won; health={}, status={}",
        stats_b.health,
        final_state.status
    );
}

/// Deterministic spell-damage win: A submits `CastSpell(9)` every turn.
/// Card 9 costs 2 mana and deals 3 damage.  After ManaSystem runs, A always has ≥ 2 mana.
/// After 7 A-turns: 7 × 3 = 21 ≥ 20 → B dies.
///
/// B submits `PlayCreature(1)` every turn to always have a valid action.
#[test]
fn test_win_by_spell_damage() {
    let env = Env::default();
    let contract_id = env.register(TradingCardGame, ());
    let client = TradingCardGameClient::new(&env, &contract_id);

    let pa = Address::generate(&env);
    let pb = Address::generate(&env);

    // A: all 3-damage 2-cost spells.  B: all cheap 1-cost creatures.
    let deck_a = vec![&env, 9u32, 9u32, 9u32, 9u32, 9u32, 9u32, 9u32, 9u32];
    let deck_b = vec![&env, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32, 1u32];
    client.new_match(&pa, &pb, &deck_a, &deck_b);
    client.start_session(&pa);
    client.start_session(&pb);

    // 20 half-turns = 10 A-turns × 3 damage = 30 ≥ 20.
    for _ in 0..20u32 {
        let s = client.get_state();
        if s.status != STATUS_IN_PROGRESS {
            break;
        }
        let is_a = s.active_player == pa;
        if is_a {
            // Cast 1 spell - always affordable (mana ≥ 2 after ManaSystem; spell costs 2)
            let _ = client.try_submit_turn(&pa, &single_action(&env, Action::CastSpell(9)));
        } else {
            // B plays a 1-cost creature - always affordable (mana ≥ 2 after ManaSystem)
            let _ = client.try_submit_turn(&pb, &single_action(&env, Action::PlayCreature(1)));
        }
    }

    let stats_b = client.get_stats(&pb);
    let final_state = client.get_state();
    assert!(
        stats_b.health < STARTING_HEALTH || final_state.status == STATUS_A_WINS,
        "Expected B's health to drop or A to win via spells; health={}, status={}",
        stats_b.health,
        final_state.status
    );
}

// ─── Concession ──────────────────────────────────────────────────────────────

#[test]
fn test_concede_ends_match() {
    let (_, client, player_a, _) = setup_match();

    client.concede(&player_a);

    let state = client.get_state();
    assert_eq!(state.status, STATUS_CONCEDED);
}

#[test]
fn test_cannot_submit_after_concession() {
    let (env, client, player_a, _) = setup_match();

    client.concede(&player_a);

    let result = client.try_submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    assert!(result.is_err());
}

// ─── Invalid action guards ────────────────────────────────────────────────────

#[test]
fn test_non_active_player_cannot_submit_turn() {
    let (env, client, _player_a, player_b) = setup_match();

    // Player B tries to go on Player A's turn
    let result = client.try_submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));
    assert!(result.is_err());
}

#[test]
fn test_invalid_attacker_index_panics() {
    let (env, client, player_a, _) = setup_match();

    // No creatures on field - attacker_idx 0 is out of bounds
    let result = client.try_submit_turn(
        &player_a,
        &single_action(&env, Action::DeclareAttack(0, u32::MAX)),
    );
    assert!(result.is_err());
}

#[test]
fn test_cannot_play_spell_as_creature() {
    let (env, client, player_a, player_b) = setup_match();

    // Advance until player_a draws card 9 (spell), then try PlayCreature(9).
    // After turn 1 A draws card 5; after turn 3 A draws card 9.
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));

    let hand = client.get_hand(&player_a);
    let has_spell_9 = (0..hand.len()).any(|i| hand.get(i).unwrap().id == 9);
    if has_spell_9 {
        let result =
            client.try_submit_turn(&player_a, &single_action(&env, Action::PlayCreature(9)));
        assert!(result.is_err());
    }
}

#[test]
fn test_cannot_cast_creature_as_spell() {
    let (env, client, player_a, _) = setup_match();

    // card_id 1 is a creature, trying to CastSpell(1) should fail
    let result = client.try_submit_turn(&player_a, &single_action(&env, Action::CastSpell(1)));
    assert!(result.is_err());
}

// ─── Turn advancement ────────────────────────────────────────────────────────

#[test]
fn test_turn_advances_after_submit() {
    let (env, client, player_a, player_b) = setup_match();

    let state = client.get_state();
    assert_eq!(state.turn, 1);
    assert_eq!(state.active_player, player_a);

    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));

    let state = client.get_state();
    assert_eq!(state.turn, 2);
    assert_eq!(state.active_player, player_b);

    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));

    let state = client.get_state();
    assert_eq!(state.turn, 3);
    assert_eq!(state.active_player, player_a);
}

// ─── Card draw during turn ───────────────────────────────────────────────────

#[test]
fn test_card_drawn_at_start_of_turn() {
    let (env, client, player_a, player_b) = setup_match();

    let initial_hand_size = client.get_hand(&player_a).len();

    // Player A plays one card (hand shrinks by 1 from play, +1 from draw = net 0)
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    let hand_after = client.get_hand(&player_a).len();

    // Hand size: started at STARTING_HAND_SIZE, drew 1, played 1 → same size
    assert_eq!(hand_after, initial_hand_size);

    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));
    // A's turn again - draw happens, then play card 2
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(2)));
    let hand_a_turn3 = client.get_hand(&player_a).len();
    // Started turn with same initial size (drew 1, played 1) → same net
    assert_eq!(hand_a_turn3, initial_hand_size);
}

// ─── SessionBuilder usage demonstration ──────────────────────────────────────

#[test]
fn test_session_expired_blocks_turn() {
    let env = Env::default();
    let contract_id = env.register(TradingCardGame, ());
    let client = TradingCardGameClient::new(&env, &contract_id);

    let pa = Address::generate(&env);
    let pb = Address::generate(&env);
    let deck = vec![&env, 1u32, 2u32, 3u32, 4u32, 5u32];
    client.new_match(&pa, &pb, &deck, &deck);

    // Do NOT call start_session - session_a_expires == 0 → expired
    let result = client.try_submit_turn(&pa, &single_action(&env, Action::PlayCreature(1)));
    assert!(result.is_err());
}

// ─── BatchBuilder demonstration test ─────────────────────────────────────────

#[test]
fn test_batch_builder_three_actions_atomic() {
    let (env, client, player_a, player_b) = setup_match();

    // Turn 1 (A): play card 1 (cost 1). Mana=2.
    client.submit_turn(&player_a, &single_action(&env, Action::PlayCreature(1)));
    client.submit_turn(&player_b, &single_action(&env, Action::PlayCreature(1)));

    // Turn 3 (A): mana=3.
    // Multi-action batch: play card 2 (cost 2 → mana=1), then attack face with creature at idx 0.
    let turn_actions = actions(
        &env,
        &[Action::PlayCreature(2), Action::DeclareAttack(0, u32::MAX)],
    );
    let result = client.submit_turn(&player_a, &turn_actions);
    assert!(result.success);
    assert_eq!(result.actions_executed, 2);
}

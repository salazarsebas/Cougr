use soroban_sdk::{Env, Vec};

use crate::components::{
    Card, CreatureState, ECSWorldState, GameError, PlayerField, PlayerHand, KIND_CREATURE,
    KIND_SPELL, MAX_FIELD, MAX_HAND, MAX_MANA,
};

// ── Card library ──────────────────────────────────────────────────────────────

/// Returns the canonical card definition for a given card id.
/// Cards 1-5: cheap creatures; 6-8: powerful creatures; 9-10: spells.
pub(crate) fn card_definition(card_id: u32) -> Option<Card> {
    match card_id {
        1 => Some(Card::new(1, KIND_CREATURE, 1, 1, 2)),
        2 => Some(Card::new(2, KIND_CREATURE, 2, 2, 2)),
        3 => Some(Card::new(3, KIND_CREATURE, 2, 1, 3)),
        4 => Some(Card::new(4, KIND_CREATURE, 3, 3, 2)),
        5 => Some(Card::new(5, KIND_CREATURE, 3, 2, 4)),
        6 => Some(Card::new(6, KIND_CREATURE, 4, 4, 4)),
        7 => Some(Card::new(7, KIND_CREATURE, 5, 5, 5)),
        8 => Some(Card::new(8, KIND_CREATURE, 6, 6, 6)),
        9 => Some(Card::new(9, KIND_SPELL, 2, 3, 0)),
        10 => Some(Card::new(10, KIND_SPELL, 4, 5, 0)),
        _ => None,
    }
}

// ── DrawSystem ────────────────────────────────────────────────────────────────

/// Draws one card from the player's deck into their hand.
pub(crate) fn draw_system(env: &Env, world: &mut ECSWorldState, is_a: bool) {
    let (hand, deck) = if is_a {
        (&mut world.hand_a, &mut world.deck_a)
    } else {
        (&mut world.hand_b, &mut world.deck_b)
    };

    if deck.is_empty() || hand.cards.len() >= MAX_HAND {
        return;
    }

    let card_id = deck.get(0).unwrap();
    let mut new_deck = Vec::new(env);
    for i in 1..deck.len() {
        new_deck.push_back(deck.get(i).unwrap());
    }
    *deck = new_deck;
    hand.cards.push_back(card_id);
}

// ── ManaSystem ────────────────────────────────────────────────────────────────

/// Increments max mana (capped at MAX_MANA) and refills current mana.
pub(crate) fn mana_system(world: &mut ECSWorldState, is_a: bool) {
    let stats = if is_a {
        &mut world.stats_a
    } else {
        &mut world.stats_b
    };
    if stats.max_mana < MAX_MANA {
        stats.max_mana += 1;
    }
    stats.mana = stats.max_mana;
}

// ── PlayCardSystem ────────────────────────────────────────────────────────────

/// Validates and moves a creature card from hand to field.
pub(crate) fn play_card_system(env: &Env, world: &mut ECSWorldState, is_a: bool, card_id: u32) {
    let card = match card_definition(card_id) {
        Some(c) => c,
        None => soroban_sdk::panic_with_error!(env, GameError::InvalidCard),
    };

    if card.kind != KIND_CREATURE {
        soroban_sdk::panic_with_error!(env, GameError::InvalidCard);
    }

    let (hand, field, stats) = if is_a {
        (&mut world.hand_a, &mut world.field_a, &mut world.stats_a)
    } else {
        (&mut world.hand_b, &mut world.field_b, &mut world.stats_b)
    };

    if stats.mana < card.cost {
        soroban_sdk::panic_with_error!(env, GameError::InsufficientMana);
    }

    if field.creatures.len() >= MAX_FIELD {
        soroban_sdk::panic_with_error!(env, GameError::FieldFull);
    }

    let pos = find_card_in_hand(hand, card_id);
    if pos >= hand.cards.len() {
        soroban_sdk::panic_with_error!(env, GameError::CardNotInHand);
    }
    let mut new_hand = Vec::new(env);
    for i in 0..hand.cards.len() {
        if i != pos {
            new_hand.push_back(hand.cards.get(i).unwrap());
        }
    }
    hand.cards = new_hand;

    stats.mana -= card.cost;
    field
        .creatures
        .push_back(CreatureState::new(card_id, card.power, card.toughness));
}

// ── CastSpellSystem ───────────────────────────────────────────────────────────

/// Validates and casts a spell, dealing direct damage to the opponent.
pub(crate) fn cast_spell_system(env: &Env, world: &mut ECSWorldState, is_a: bool, card_id: u32) {
    let card = match card_definition(card_id) {
        Some(c) => c,
        None => soroban_sdk::panic_with_error!(env, GameError::InvalidCard),
    };

    if card.kind != KIND_SPELL {
        soroban_sdk::panic_with_error!(env, GameError::InvalidCard);
    }

    let (hand, stats, opp_stats) = if is_a {
        (&mut world.hand_a, &mut world.stats_a, &mut world.stats_b)
    } else {
        (&mut world.hand_b, &mut world.stats_b, &mut world.stats_a)
    };

    if stats.mana < card.cost {
        soroban_sdk::panic_with_error!(env, GameError::InsufficientMana);
    }

    let pos = find_card_in_hand(hand, card_id);
    if pos >= hand.cards.len() {
        soroban_sdk::panic_with_error!(env, GameError::CardNotInHand);
    }
    let mut new_hand = Vec::new(env);
    for i in 0..hand.cards.len() {
        if i != pos {
            new_hand.push_back(hand.cards.get(i).unwrap());
        }
    }
    hand.cards = new_hand;

    stats.mana -= card.cost;
    opp_stats.health = opp_stats.health.saturating_sub(card.power);
}

// ── CombatSystem ──────────────────────────────────────────────────────────────

/// Resolves one attack declaration.
/// `target_idx == u32::MAX` means a direct face attack.
pub(crate) fn combat_system(
    env: &Env,
    world: &mut ECSWorldState,
    is_a: bool,
    attacker_idx: u32,
    target_idx: u32,
) {
    let (my_field, opp_field, opp_stats) = if is_a {
        (&mut world.field_a, &mut world.field_b, &mut world.stats_b)
    } else {
        (&mut world.field_b, &mut world.field_a, &mut world.stats_a)
    };

    if attacker_idx >= my_field.creatures.len() {
        soroban_sdk::panic_with_error!(env, GameError::InvalidTarget);
    }

    let attacker_power = my_field.creatures.get(attacker_idx).unwrap().power;

    if target_idx == u32::MAX {
        opp_stats.health = opp_stats.health.saturating_sub(attacker_power);
    } else {
        if target_idx >= opp_field.creatures.len() {
            soroban_sdk::panic_with_error!(env, GameError::InvalidTarget);
        }

        let blocker_power = opp_field.creatures.get(target_idx).unwrap().power;
        let blocker_toughness = opp_field
            .creatures
            .get(target_idx)
            .unwrap()
            .current_toughness;
        let attacker_toughness = my_field
            .creatures
            .get(attacker_idx)
            .unwrap()
            .current_toughness;

        let new_blocker_toughness = blocker_toughness.saturating_sub(attacker_power);
        let new_attacker_toughness = attacker_toughness.saturating_sub(blocker_power);

        let mut attacker = my_field.creatures.get(attacker_idx).unwrap();
        attacker.current_toughness = new_attacker_toughness;
        my_field.creatures.set(attacker_idx, attacker);

        let mut blocker = opp_field.creatures.get(target_idx).unwrap();
        blocker.current_toughness = new_blocker_toughness;
        opp_field.creatures.set(target_idx, blocker);

        remove_dead(env, my_field);
        remove_dead(env, opp_field);
    }
}

// ── WinConditionSystem ────────────────────────────────────────────────────────

/// Marks the match over if any player's health reaches 0.
pub(crate) fn win_condition_system(world: &mut ECSWorldState) {
    use crate::components::{STATUS_A_WINS, STATUS_B_WINS};
    if world.stats_a.health == 0 {
        world.match_state.status = STATUS_B_WINS;
    } else if world.stats_b.health == 0 {
        world.match_state.status = STATUS_A_WINS;
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn find_card_in_hand(hand: &PlayerHand, card_id: u32) -> u32 {
    for i in 0..hand.cards.len() {
        if hand.cards.get(i).unwrap() == card_id {
            return i;
        }
    }
    u32::MAX
}

pub(crate) fn remove_dead(env: &Env, field: &mut PlayerField) {
    let mut survivors = Vec::new(env);
    for i in 0..field.creatures.len() {
        let c = field.creatures.get(i).unwrap();
        if c.current_toughness > 0 {
            survivors.push_back(c);
        }
    }
    field.creatures = survivors;
}

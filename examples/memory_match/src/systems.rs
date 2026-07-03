use soroban_sdk::{Address, Env, Vec};

use crate::components::{CardState, ECSWorldState, GameState, RevealInfo, RevealResult};

// ── ValidationSystem ─────────────────────────────────────────────────────────

/// Validates that `player` is authorized to act on `world` and that the game
/// is in a state where a reveal can be attempted.
pub(crate) fn validate_reveal(world: &ECSWorldState, player: &Address, position: u32) {
    if world.game_state.player != *player {
        panic!("Not authorized player");
    }

    if world.game_state.game_over {
        panic!("Game is over");
    }

    if !world.game_state.can_reveal {
        panic!("Cannot reveal more than 2 cards");
    }

    if position >= world.board.cards.len() {
        panic!("Invalid position");
    }
}

/// Validates that `player` is authorized to reset `world`.
pub(crate) fn validate_player(world: &ECSWorldState, player: &Address) {
    if world.game_state.player != *player {
        panic!("Not authorized player");
    }
}

// ── RevealSystem ─────────────────────────────────────────────────────────────

/// Reveals the card at `position`, resolves a pair if two cards are now
/// revealed (match, no-match, or game-over), and returns the outcome. Mutates
/// `world` in place; the caller is responsible for persisting it.
pub(crate) fn reveal_card_system(
    env: &Env,
    world: &mut ECSWorldState,
    position: u32,
) -> RevealInfo {
    // Get card and check state
    let card_entity_id = world.board.cards.get(position).unwrap();
    let card_value = {
        let card = world.get_card(card_entity_id).unwrap();

        if matches!(card.state, CardState::Revealed | CardState::Matched) {
            panic!("Card already revealed or matched");
        }

        card.value
    };

    // Reveal the card
    world.update_card(card_entity_id, CardState::Revealed);

    // Add to revealed cards
    world.board.revealed_cards.push_back(position);

    // Update game state
    world.game_state.moves_count += 1;

    // If 2 cards are revealed, disable further reveals
    if world.board.revealed_cards.len() == 2 {
        world.game_state.can_reveal = false;
    }

    // Check if we have 2 cards revealed
    if world.board.revealed_cards.len() == 2 {
        resolve_pair_system(env, world, position, card_value)
    } else {
        RevealInfo {
            result: RevealResult::CardRevealed,
            position,
            value: card_value,
            positions: Vec::new(env),
        }
    }
}

/// Resolves the two currently-revealed cards: marks them matched (and checks
/// for game completion) or hides them again on a mismatch.
fn resolve_pair_system(
    env: &Env,
    world: &mut ECSWorldState,
    position: u32,
    card_value: u32,
) -> RevealInfo {
    let pos1 = world.board.revealed_cards.get(0).unwrap();
    let pos2 = world.board.revealed_cards.get(1).unwrap();

    let card1_entity_id = world.board.cards.get(pos1).unwrap();
    let card2_entity_id = world.board.cards.get(pos2).unwrap();

    let card1_value = world.get_card(card1_entity_id).unwrap().value;
    let card2_value = world.get_card(card2_entity_id).unwrap().value;

    let is_match = card1_value == card2_value;
    let mut positions = Vec::new(env);
    positions.push_back(pos1);
    positions.push_back(pos2);

    if is_match {
        // Mark cards as matched
        world.update_card(card1_entity_id, CardState::Matched);
        world.update_card(card2_entity_id, CardState::Matched);

        world.board.matched_pairs += 1;

        // Clear revealed cards and re-enable reveals
        world.board.revealed_cards = Vec::new(env);
        world.game_state.can_reveal = true;

        // Check for game over
        if world.board.matched_pairs == world.board.total_pairs {
            world.game_state.game_over = true;

            RevealInfo {
                result: RevealResult::GameOver,
                position,
                value: card_value,
                positions,
            }
        } else {
            RevealInfo {
                result: RevealResult::MatchFound,
                position,
                value: card_value,
                positions,
            }
        }
    } else {
        // Hide cards again
        world.update_card(card1_entity_id, CardState::Hidden);
        world.update_card(card2_entity_id, CardState::Hidden);

        // Clear revealed cards and re-enable reveals
        world.board.revealed_cards = Vec::new(env);
        world.game_state.can_reveal = true;

        RevealInfo {
            result: RevealResult::NoMatch,
            position,
            value: card_value,
            positions,
        }
    }
}

// ── ResetSystem ───────────────────────────────────────────────────────────────

/// Resets every card to hidden and clears board/game-state progress.
pub(crate) fn reset_system(env: &Env, world: &mut ECSWorldState) {
    // Reset all cards to hidden
    // This is complex with Soroban Vec - we'll need to recreate the cards
    let mut new_cards = Vec::new(env);
    for i in 0..world.cards.len() {
        let mut card = world.cards.get(i).unwrap().clone();
        card.state = CardState::Hidden;
        new_cards.push_back(card);
    }
    world.cards = new_cards;

    // Reset board
    world.board.revealed_cards = Vec::new(env);
    world.board.matched_pairs = 0;

    // Reset game state
    world.game_state.moves_count = 0;
    world.game_state.game_over = false;
    world.game_state.can_reveal = true;
}

// ── ProjectionSystem ─────────────────────────────────────────────────────────

/// Projects the internal `ECSWorldState` into the public `GameState` view.
pub(crate) fn to_game_state(env: &Env, world: &ECSWorldState) -> GameState {
    let mut board_state = Vec::new(env);
    for i in 0..16 {
        let card_entity_id = world.board.cards.get(i).unwrap();
        let card = world.get_card(card_entity_id).unwrap();

        let state_value = match card.state {
            CardState::Hidden => 0u32,
            CardState::Revealed => card.value + 1, // 1-8 for revealed values
            CardState::Matched => 9u32,
        };
        board_state.push_back(state_value);
    }

    GameState {
        board_state,
        revealed_count: world.board.revealed_cards.len(),
        matched_pairs: world.board.matched_pairs,
        total_pairs: world.board.total_pairs,
        moves_count: world.game_state.moves_count,
        game_over: world.game_state.game_over,
    }
}

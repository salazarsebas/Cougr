use super::*;
use soroban_sdk::{testutils::Address as _, Env};

fn setup_game() -> (Env, MemoryMatchContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(MemoryMatchContract, ());
    let client = MemoryMatchContractClient::new(&env, &contract_id);

    let player = Address::generate(&env);
    client.init_game(&player);

    (env, client, player)
}

#[test]
fn test_init_game() {
    let env = Env::default();
    let contract_id = env.register(MemoryMatchContract, ());
    let client = MemoryMatchContractClient::new(&env, &contract_id);
    
    let player = Address::generate(&env);
    
    // Initialize game
    let game_state = client.init_game(&player);
    
    // Verify initial state
    assert_eq!(game_state.board_state.len(), 16);
    assert_eq!(game_state.revealed_count, 0);
    assert_eq!(game_state.matched_pairs, 0);
    assert_eq!(game_state.total_pairs, 8);
    assert_eq!(game_state.moves_count, 0);
    assert!(!game_state.game_over);
    
    // All cards should be hidden initially (value 0)
    for i in 0..16 {
        assert_eq!(game_state.board_state.get(i).unwrap(), 0);
    }
}

#[test]
fn test_reveal_first_card() {
    let (_, client, player) = setup_game();
    
    // Reveal first card at position 0
    let reveal_info = client.reveal_card(&player, &0);
    
    // Should be CardRevealed result
    assert!(matches!(reveal_info.result, RevealResult::CardRevealed));
    assert_eq!(reveal_info.position, 0);
    assert_eq!(reveal_info.value, 0); // Card 0 has value 0
    
    // Check game state
    let game_state = client.get_game_state();
    assert_eq!(game_state.revealed_count, 1);
    assert_eq!(game_state.moves_count, 1);
    assert!(!game_state.game_over);
    
    // Card at position 0 should be revealed with value 1 (value + 1)
    assert_eq!(game_state.board_state.get(0).unwrap(), 1);
}

#[test]
fn test_reveal_matching_pair() {
    let (_, client, player) = setup_game();
    
    // Reveal card at position 0 (value 0)
    let reveal1 = client.reveal_card(&player, &0);
    assert!(matches!(reveal1.result, RevealResult::CardRevealed));
    
    // Reveal matching card at position 8 (also value 0)
    let reveal2 = client.reveal_card(&player, &8);
    assert!(matches!(reveal2.result, RevealResult::MatchFound));
    assert_eq!(reveal2.position, 8);
    assert_eq!(reveal2.value, 0);
    assert_eq!(reveal2.positions.len(), 2);
    
    // Check game state
    let game_state = client.get_game_state();
    assert_eq!(game_state.revealed_count, 0); // Should be reset after match
    assert_eq!(game_state.matched_pairs, 1);
    assert_eq!(game_state.moves_count, 2);
    
    // Both cards should be marked as matched (value 9)
    assert_eq!(game_state.board_state.get(0).unwrap(), 9);
    assert_eq!(game_state.board_state.get(8).unwrap(), 9);
}

#[test]
fn test_reveal_non_matching_pair() {
    let (_, client, player) = setup_game();
    
    // Reveal card at position 0 (value 0)
    let reveal1 = client.reveal_card(&player, &0);
    assert!(matches!(reveal1.result, RevealResult::CardRevealed));
    
    // Reveal non-matching card at position 1 (value 1)
    let reveal2 = client.reveal_card(&player, &1);
    assert!(matches!(reveal2.result, RevealResult::NoMatch));
    assert_eq!(reveal2.position, 1);
    assert_eq!(reveal2.value, 1);
    assert_eq!(reveal2.positions.len(), 2);
    
    // Check game state
    let game_state = client.get_game_state();
    assert_eq!(game_state.revealed_count, 0); // Should be reset after no match
    assert_eq!(game_state.matched_pairs, 0);
    assert_eq!(game_state.moves_count, 2);
    
    // Both cards should be hidden again (value 0)
    assert_eq!(game_state.board_state.get(0).unwrap(), 0);
    assert_eq!(game_state.board_state.get(1).unwrap(), 0);
}

#[test]
fn test_reset_game() {
    let (_, client, player) = setup_game();
    
    // Play a few moves
    client.reveal_card(&player, &0);
    client.reveal_card(&player, &8); // This should match
    
    // Verify some progress was made
    let game_state = client.get_game_state();
    assert_eq!(game_state.matched_pairs, 1);
    assert_eq!(game_state.moves_count, 2);
    
    // Reset the game
    let reset_state = client.reset_game(&player);
    
    // Verify reset state
    assert_eq!(reset_state.revealed_count, 0);
    assert_eq!(reset_state.matched_pairs, 0);
    assert_eq!(reset_state.moves_count, 0);
    assert!(!reset_state.game_over);
    
    // All cards should be hidden again
    for i in 0..16 {
        assert_eq!(reset_state.board_state.get(i).unwrap(), 0);
    }
}

#[test]
#[should_panic(expected = "Game not initialized")]
fn test_reveal_without_init() {
    let env = Env::default();
    let contract_id = env.register(MemoryMatchContract, ());
    let client = MemoryMatchContractClient::new(&env, &contract_id);
    
    let player = Address::generate(&env);
    
    // Try to reveal without initializing
    client.reveal_card(&player, &0);
}

#[test]
#[should_panic(expected = "Not authorized player")]
fn test_unauthorized_player() {
    let env = Env::default();
    let contract_id = env.register(MemoryMatchContract, ());
    let client = MemoryMatchContractClient::new(&env, &contract_id);
    
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    
    client.init_game(&player1);
    
    // Try to reveal with different player
    client.reveal_card(&player2, &0);
}

#[test]
#[should_panic(expected = "Invalid position")]
fn test_invalid_position() {
    let (_, client, player) = setup_game();
    
    // Try to reveal invalid position
    client.reveal_card(&player, &16);
}

#[test]
#[should_panic(expected = "Card already revealed or matched")]
fn test_reveal_same_card_twice() {
    let (_, client, player) = setup_game();
    
    // Reveal a card
    client.reveal_card(&player, &0);
    
    // Try to reveal the same card again
    client.reveal_card(&player, &0);
}

#[test]
fn test_reveal_three_cards_sequence() {
    let (_, client, player) = setup_game();
    
    // Reveal two cards (0 and 1 - these don't match)
    let reveal1 = client.reveal_card(&player, &0);
    assert!(matches!(reveal1.result, RevealResult::CardRevealed));
    
    let reveal2 = client.reveal_card(&player, &1);
    assert!(matches!(reveal2.result, RevealResult::NoMatch));
    
    // After no match, can_reveal should be true again
    // So we can reveal a third card
    let reveal3 = client.reveal_card(&player, &2);
    assert!(matches!(reveal3.result, RevealResult::CardRevealed));
}

#[test]
fn test_card_values() {
    let env = Env::default();
    let contract_id = env.register(MemoryMatchContract, ());
    let client = MemoryMatchContractClient::new(&env, &contract_id);
    
    let player = Address::generate(&env);
    client.init_game(&player);
    
    // Test that card values are correct according to our deterministic layout
    // Positions 0-7 should have values 0-7
    // Positions 8-15 should have values 0-7 (matching pairs)
    
    let expected_values = [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7];
    
    for i in 0..16 {
        // Reveal card at position i
        let reveal = client.reveal_card(&player, &i);
        assert!(matches!(reveal.result, RevealResult::CardRevealed));
        assert_eq!(reveal.value, expected_values[i as usize]);
        
        // Reset after each reveal to avoid the 2-card limit
        client.reset_game(&player);
        client.init_game(&player);
    }
}

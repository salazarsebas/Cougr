#![allow(dead_code)]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec};

// ── Constants ────────────────────────────────────────────────────────────────

pub(crate) const WORLD_KEY: Symbol = symbol_short!("WORLD");

// ── Card states ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[contracttype]
pub enum CardState {
    Hidden = 0,
    Revealed = 1,
    Matched = 2,
}

// ── Components ───────────────────────────────────────────────────────────────

// Card component representing a single card
#[contracttype]
#[derive(Clone, Debug)]
pub struct CardComponent {
    pub card_id: u32,
    pub value: u32, // 0-7 for 8 pairs (16 cards total)
    pub state: CardState,
    pub position: u32, // 0-15 board position
    pub entity_id: u32,
}

impl CardComponent {
    pub fn new(card_id: u32, value: u32, position: u32, entity_id: u32) -> Self {
        Self {
            card_id,
            value,
            state: CardState::Hidden,
            position,
            entity_id,
        }
    }
}

impl ComponentTrait for CardComponent {
    fn component_type() -> Symbol {
        symbol_short!("card")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.card_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.value.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &(self.state as u32).to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.position.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 20 {
            return None;
        }

        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);

        let card_id = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);

        let value = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);

        let state = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);

        let position = u32::from_be_bytes([
            data.get(16).unwrap(),
            data.get(17).unwrap(),
            data.get(18).unwrap(),
            data.get(19).unwrap(),
        ]);

        Some(CardComponent {
            card_id,
            value,
            state: match state {
                0 => CardState::Hidden,
                1 => CardState::Revealed,
                2 => CardState::Matched,
                _ => return None,
            },
            position,
            entity_id,
        })
    }
}

// Board component managing the game board
#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardComponent {
    pub cards: Vec<u32>,          // Card entity IDs
    pub revealed_cards: Vec<u32>, // Currently revealed card positions
    pub matched_pairs: u32,
    pub total_pairs: u32,
    pub entity_id: u32,
}

impl BoardComponent {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        let mut cards = Vec::new(env);
        for i in 0..16 {
            cards.push_back(i);
        }

        Self {
            cards,
            revealed_cards: Vec::new(env),
            matched_pairs: 0,
            total_pairs: 8,
            entity_id,
        }
    }
}

impl ComponentTrait for BoardComponent {
    fn component_type() -> Symbol {
        symbol_short!("board")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.matched_pairs.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.total_pairs.to_be_bytes()));

        // Serialize cards vector
        bytes.append(&Bytes::from_array(env, &self.cards.len().to_be_bytes()));
        for i in 0..self.cards.len() {
            let card_id = self.cards.get(i).unwrap();
            bytes.append(&Bytes::from_array(env, &card_id.to_be_bytes()));
        }

        // Serialize revealed_cards vector
        bytes.append(&Bytes::from_array(
            env,
            &self.revealed_cards.len().to_be_bytes(),
        ));
        for i in 0..self.revealed_cards.len() {
            let pos = self.revealed_cards.get(i).unwrap();
            bytes.append(&Bytes::from_array(env, &pos.to_be_bytes()));
        }

        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }

        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);

        let matched_pairs = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);

        let total_pairs = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);

        let mut offset = 12;

        // Deserialize cards
        let cards_len = u32::from_be_bytes([
            data.get(offset).unwrap(),
            data.get(offset + 1).unwrap(),
            data.get(offset + 2).unwrap(),
            data.get(offset + 3).unwrap(),
        ]);
        offset += 4;

        let mut cards = Vec::new(env);
        for _ in 0..cards_len {
            let card_id = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            cards.push_back(card_id);
            offset += 4;
        }

        // Deserialize revealed_cards
        let revealed_len = u32::from_be_bytes([
            data.get(offset).unwrap(),
            data.get(offset + 1).unwrap(),
            data.get(offset + 2).unwrap(),
            data.get(offset + 3).unwrap(),
        ]);
        offset += 4;

        let mut revealed_cards = Vec::new(env);
        for _ in 0..revealed_len {
            let pos = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            revealed_cards.push_back(pos);
            offset += 4;
        }

        Some(BoardComponent {
            cards,
            revealed_cards,
            matched_pairs,
            total_pairs,
            entity_id,
        })
    }
}

// Game state component
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStateComponent {
    pub player: Address,
    pub moves_count: u32,
    pub game_over: bool,
    pub can_reveal: bool, // Can reveal a card (true if 0 or 1 cards revealed)
    pub entity_id: u32,
}

impl GameStateComponent {
    pub fn new(player: Address, entity_id: u32) -> Self {
        Self {
            player,
            moves_count: 0,
            game_over: false,
            can_reveal: true,
            entity_id,
        }
    }
}

impl ComponentTrait for GameStateComponent {
    fn component_type() -> Symbol {
        symbol_short!("gamestate")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&self.player.to_string().to_bytes());
        bytes.append(&Bytes::from_array(env, &self.moves_count.to_be_bytes()));
        bytes.append(&Bytes::from_array(
            env,
            &(if self.game_over { 1u32 } else { 0u32 }).to_be_bytes(),
        ));
        bytes.append(&Bytes::from_array(
            env,
            &(if self.can_reveal { 1u32 } else { 0u32 }).to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 48 {
            return None;
        }

        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);

        let player_bytes = data.slice(4..36);
        let player = Address::from_string_bytes(&player_bytes);

        let moves_count = u32::from_be_bytes([
            data.get(36).unwrap(),
            data.get(37).unwrap(),
            data.get(38).unwrap(),
            data.get(39).unwrap(),
        ]);

        let game_over = u32::from_be_bytes([
            data.get(40).unwrap(),
            data.get(41).unwrap(),
            data.get(42).unwrap(),
            data.get(43).unwrap(),
        ]) == 1;

        let can_reveal = u32::from_be_bytes([
            data.get(44).unwrap(),
            data.get(45).unwrap(),
            data.get(46).unwrap(),
            data.get(47).unwrap(),
        ]) == 1;

        Some(GameStateComponent {
            player,
            moves_count,
            game_over,
            can_reveal,
            entity_id,
        })
    }
}

// ── ECS World ────────────────────────────────────────────────────────────────

/// ECS World State - stores all game entities and components
#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub cards: Vec<CardComponent>,
    pub board: BoardComponent,
    pub game_state: GameStateComponent,
}

impl ECSWorldState {
    pub fn new(env: &Env, player: Address) -> Self {
        // Create card components
        let mut cards = Vec::new(env);
        for i in 0..16 {
            let value = match i {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                5 => 5,
                6 => 6,
                7 => 7,
                8 => 0,
                9 => 1,
                10 => 2,
                11 => 3,
                12 => 4,
                13 => 5,
                14 => 6,
                15 => 7,
                _ => 0,
            };
            cards.push_back(CardComponent::new(i, value, i, i));
        }

        // Create board
        let board = BoardComponent::new(env, 16);

        // Create game state
        let game_state = GameStateComponent::new(player, 17);

        Self {
            cards,
            board,
            game_state,
        }
    }

    pub fn get_card(&self, entity_id: u32) -> Option<CardComponent> {
        for i in 0..self.cards.len() {
            let card = self.cards.get(i).unwrap();
            if card.entity_id == entity_id {
                return Some(card.clone());
            }
        }
        None
    }

    pub fn update_card(&mut self, entity_id: u32, new_state: CardState) {
        let mut new_cards = Vec::new(self.cards.env());
        for i in 0..self.cards.len() {
            let mut card = self.cards.get(i).unwrap().clone();
            if card.entity_id == entity_id {
                card.state = new_state;
            }
            new_cards.push_back(card);
        }
        self.cards = new_cards;
    }
}

// ── Public API types ─────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub board_state: Vec<u32>, // 0=Hidden, 1-8=Revealed values, 9=Matched
    pub revealed_count: u32,
    pub matched_pairs: u32,
    pub total_pairs: u32,
    pub moves_count: u32,
    pub game_over: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum RevealResult {
    Success = 0,
    CardRevealed = 1,
    MatchFound = 2,
    NoMatch = 3,
    GameOver = 4,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RevealInfo {
    pub result: RevealResult,
    pub position: u32,
    pub value: u32,
    pub positions: Vec<u32>,
}

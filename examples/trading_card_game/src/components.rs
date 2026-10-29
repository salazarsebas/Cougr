use cougr_core::component::ComponentTrait;
use soroban_sdk::{contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec};

// ── Error codes ──────────────────────────────────────────────────────────────

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GameError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotYourTurn = 3,
    NotAPlayer = 4,
    InvalidCard = 5,
    InsufficientMana = 6,
    CardNotInHand = 7,
    FieldFull = 8,
    InvalidTarget = 9,
    WrongPhase = 10,
    GameOver = 11,
    BatchEmpty = 12,
    InvalidAction = 13,
    InvalidPosition = 14,
    SessionExpired = 15,
}

// ── Card kinds ───────────────────────────────────────────────────────────────

pub const KIND_CREATURE: u32 = 0;
pub const KIND_SPELL: u32 = 1;

// ── Match phases ─────────────────────────────────────────────────────────────

pub const PHASE_DRAW: u32 = 0;
pub const PHASE_MAIN: u32 = 1;
pub const PHASE_COMBAT: u32 = 2;
pub const PHASE_END: u32 = 3;

// ── Match status ─────────────────────────────────────────────────────────────

pub const STATUS_IN_PROGRESS: u32 = 0;
pub const STATUS_A_WINS: u32 = 1;
pub const STATUS_B_WINS: u32 = 2;
pub const STATUS_CONCEDED: u32 = 3;

// ── Field / hand limits ──────────────────────────────────────────────────────

pub const MAX_HAND: u32 = 7;
pub const MAX_FIELD: u32 = 5;
pub const MAX_MANA: u32 = 10;
pub const STARTING_HEALTH: u32 = 20;
pub const STARTING_HAND_SIZE: u32 = 4;

// ── Action symbols ───────────────────────────────────────────────────────────

pub const SYM_PLAY: Symbol = symbol_short!("play");
pub const SYM_SPELL: Symbol = symbol_short!("spell");
pub const SYM_ATTACK: Symbol = symbol_short!("attack");
pub const SYM_CONCEDE: Symbol = symbol_short!("concede");

// ── Card definition ──────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct Card {
    pub id: u32,
    pub kind: u32,
    pub cost: u32,
    pub power: u32,
    pub toughness: u32,
}

impl Card {
    pub fn new(id: u32, kind: u32, cost: u32, power: u32, toughness: u32) -> Self {
        Self {
            id,
            kind,
            cost,
            power,
            toughness,
        }
    }
}

// ── CreatureState ────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct CreatureState {
    pub card_id: u32,
    pub power: u32,
    pub toughness: u32,
    pub current_toughness: u32,
}

impl CreatureState {
    pub fn new(card_id: u32, power: u32, toughness: u32) -> Self {
        Self {
            card_id,
            power,
            toughness,
            current_toughness: toughness,
        }
    }
}

// ── PlayerHand ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerHand {
    pub cards: Vec<u32>,
    pub entity_id: u32,
}

impl PlayerHand {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        Self {
            cards: Vec::new(env),
            entity_id,
        }
    }
}

impl ComponentTrait for PlayerHand {
    fn component_type() -> Symbol {
        symbol_short!("hand")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.cards.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let card_id = self.cards.get(i).unwrap_or(0);
            bytes.append(&Bytes::from_array(env, &card_id.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let len = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let mut cards = Vec::new(env);
        for i in 0..len {
            let offset = 8 + (i * 4);
            let card_id = u32::from_be_bytes([
                data.get(offset).unwrap(),
                data.get(offset + 1).unwrap(),
                data.get(offset + 2).unwrap(),
                data.get(offset + 3).unwrap(),
            ]);
            cards.push_back(card_id);
        }
        Some(Self { cards, entity_id })
    }
}

// ── PlayerField ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerField {
    pub creatures: Vec<CreatureState>,
    pub entity_id: u32,
}

impl PlayerField {
    pub fn new(env: &Env, entity_id: u32) -> Self {
        Self {
            creatures: Vec::new(env),
            entity_id,
        }
    }
}

impl ComponentTrait for PlayerField {
    fn component_type() -> Symbol {
        symbol_short!("field")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        let len = self.creatures.len();
        bytes.append(&Bytes::from_array(env, &len.to_be_bytes()));
        for i in 0..len {
            let c = self.creatures.get(i).unwrap();
            bytes.append(&Bytes::from_array(env, &c.card_id.to_be_bytes()));
            bytes.append(&Bytes::from_array(env, &c.power.to_be_bytes()));
            bytes.append(&Bytes::from_array(env, &c.toughness.to_be_bytes()));
            bytes.append(&Bytes::from_array(env, &c.current_toughness.to_be_bytes()));
        }
        bytes
    }

    fn deserialize(env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() < 8 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let len = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let mut creatures = Vec::new(env);
        for i in 0..len {
            let base = 8 + (i * 16);
            let card_id = u32::from_be_bytes([
                data.get(base).unwrap(),
                data.get(base + 1).unwrap(),
                data.get(base + 2).unwrap(),
                data.get(base + 3).unwrap(),
            ]);
            let power = u32::from_be_bytes([
                data.get(base + 4).unwrap(),
                data.get(base + 5).unwrap(),
                data.get(base + 6).unwrap(),
                data.get(base + 7).unwrap(),
            ]);
            let toughness = u32::from_be_bytes([
                data.get(base + 8).unwrap(),
                data.get(base + 9).unwrap(),
                data.get(base + 10).unwrap(),
                data.get(base + 11).unwrap(),
            ]);
            let current_toughness = u32::from_be_bytes([
                data.get(base + 12).unwrap(),
                data.get(base + 13).unwrap(),
                data.get(base + 14).unwrap(),
                data.get(base + 15).unwrap(),
            ]);
            creatures.push_back(CreatureState {
                card_id,
                power,
                toughness,
                current_toughness,
            });
        }
        Some(Self {
            creatures,
            entity_id,
        })
    }
}

// ── PlayerStats ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerStats {
    pub health: u32,
    pub mana: u32,
    pub max_mana: u32,
    pub entity_id: u32,
}

impl PlayerStats {
    pub fn new(entity_id: u32) -> Self {
        Self {
            health: STARTING_HEALTH,
            mana: 1,
            max_mana: 1,
            entity_id,
        }
    }
}

impl ComponentTrait for PlayerStats {
    fn component_type() -> Symbol {
        symbol_short!("pstats")
    }

    fn serialize(&self, env: &Env) -> Bytes {
        let mut bytes = Bytes::new(env);
        bytes.append(&Bytes::from_array(env, &self.entity_id.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.health.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.mana.to_be_bytes()));
        bytes.append(&Bytes::from_array(env, &self.max_mana.to_be_bytes()));
        bytes
    }

    fn deserialize(_env: &Env, data: &Bytes) -> Option<Self> {
        if data.len() != 16 {
            return None;
        }
        let entity_id = u32::from_be_bytes([
            data.get(0).unwrap(),
            data.get(1).unwrap(),
            data.get(2).unwrap(),
            data.get(3).unwrap(),
        ]);
        let health = u32::from_be_bytes([
            data.get(4).unwrap(),
            data.get(5).unwrap(),
            data.get(6).unwrap(),
            data.get(7).unwrap(),
        ]);
        let mana = u32::from_be_bytes([
            data.get(8).unwrap(),
            data.get(9).unwrap(),
            data.get(10).unwrap(),
            data.get(11).unwrap(),
        ]);
        let max_mana = u32::from_be_bytes([
            data.get(12).unwrap(),
            data.get(13).unwrap(),
            data.get(14).unwrap(),
            data.get(15).unwrap(),
        ]);
        Some(Self {
            health,
            mana,
            max_mana,
            entity_id,
        })
    }
}

// ── MatchState ───────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchState {
    pub turn: u32,
    pub active_player: Address,
    pub phase: u32,
    pub status: u32,
    pub entity_id: u32,
}

impl MatchState {
    pub fn new(active_player: Address, entity_id: u32) -> Self {
        Self {
            turn: 1,
            active_player,
            phase: PHASE_DRAW,
            status: STATUS_IN_PROGRESS,
            entity_id,
        }
    }
}

// ── ECSWorldState ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub player_a: Address,
    pub player_b: Address,
    pub deck_a: Vec<u32>,
    pub deck_b: Vec<u32>,
    pub hand_a: PlayerHand,
    pub hand_b: PlayerHand,
    pub field_a: PlayerField,
    pub field_b: PlayerField,
    pub stats_a: PlayerStats,
    pub stats_b: PlayerStats,
    pub match_state: MatchState,
    pub session_a_expires: u64,
    pub session_b_expires: u64,
    pub next_entity_id: u32,
}

// ── Action enum ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub enum Action {
    PlayCreature(u32),
    CastSpell(u32),
    DeclareAttack(u32, u32),
}

// ── TurnResult ────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnResult {
    pub success: bool,
    pub actions_executed: u32,
    pub match_status: u32,
    pub message: Symbol,
}

// ── FieldState (external view) ────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct FieldState {
    pub field_a: Vec<CreatureState>,
    pub field_b: Vec<CreatureState>,
}

// ── Storage key ──────────────────────────────────────────────────────────────

pub const WORLD_KEY: Symbol = symbol_short!("WORLD");

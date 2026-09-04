#![no_std]

mod components;
mod systems;
#[cfg(test)]
mod test;

pub use components::{
    Action, Card, CreatureState, ECSWorldState, FieldState, GameError, MatchState, PlayerField,
    PlayerHand, PlayerStats, TurnResult, KIND_CREATURE, KIND_SPELL, MAX_FIELD, MAX_HAND, MAX_MANA,
    PHASE_COMBAT, PHASE_DRAW, PHASE_END, PHASE_MAIN, STARTING_HAND_SIZE, STARTING_HEALTH,
    STATUS_A_WINS, STATUS_B_WINS, STATUS_CONCEDED, STATUS_IN_PROGRESS, SYM_ATTACK, SYM_CONCEDE,
    SYM_PLAY, SYM_SPELL, WORLD_KEY,
};
use systems::{
    card_definition, cast_spell_system, combat_system, draw_system, mana_system, play_card_system,
    win_condition_system,
};

use cougr_core::auth::{BatchBuilder, GameAction, SessionBuilder};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, Bytes, Env, Vec,
};

#[contract]
pub struct TradingCardGame;

#[contractimpl]
impl TradingCardGame {
    // ── Match Setup ──────────────────────────────────────────────────────────

    /// Initialize a new match.  `deck_a` / `deck_b` are ordered lists of card IDs.
    pub fn new_match(
        env: Env,
        player_a: Address,
        player_b: Address,
        deck_a: Vec<u32>,
        deck_b: Vec<u32>,
    ) {
        if env.storage().instance().has(&WORLD_KEY) {
            panic_with_error!(&env, GameError::AlreadyInitialized);
        }

        let mut eid = 0u32;

        let mut hand_a = PlayerHand::new(&env, eid);
        eid += 1;
        let mut hand_b = PlayerHand::new(&env, eid);
        eid += 1;
        let field_a = PlayerField::new(&env, eid);
        eid += 1;
        let field_b = PlayerField::new(&env, eid);
        eid += 1;
        let stats_a = PlayerStats::new(eid);
        eid += 1;
        let stats_b = PlayerStats::new(eid);
        eid += 1;
        let match_state = MatchState::new(player_a.clone(), eid);
        eid += 1;

        // Draw starting hands from the front of each deck.
        let mut remaining_a = Vec::new(&env);
        let draw_a = STARTING_HAND_SIZE.min(deck_a.len());
        for i in 0..deck_a.len() {
            if i < draw_a {
                hand_a.cards.push_back(deck_a.get(i).unwrap());
            } else {
                remaining_a.push_back(deck_a.get(i).unwrap());
            }
        }

        let mut remaining_b = Vec::new(&env);
        let draw_b = STARTING_HAND_SIZE.min(deck_b.len());
        for i in 0..deck_b.len() {
            if i < draw_b {
                hand_b.cards.push_back(deck_b.get(i).unwrap());
            } else {
                remaining_b.push_back(deck_b.get(i).unwrap());
            }
        }

        let world = ECSWorldState {
            player_a,
            player_b,
            deck_a: remaining_a,
            deck_b: remaining_b,
            hand_a,
            hand_b,
            field_a,
            field_b,
            stats_a,
            stats_b,
            match_state,
            session_a_expires: 0,
            session_b_expires: 0,
            next_entity_id: eid,
        };

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── Session Management ───────────────────────────────────────────────────

    /// Create a match-scoped session for a player.  Returns the session expiry timestamp.
    pub fn start_session(env: Env, player: Address) -> u64 {
        let mut world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        let is_a = player == world.player_a;
        let is_b = player == world.player_b;
        if !is_a && !is_b {
            panic_with_error!(&env, GameError::NotAPlayer);
        }

        let expires_at = env.ledger().timestamp() + 7200;
        let _scope = SessionBuilder::new(&env)
            .allow_action(SYM_PLAY)
            .allow_action(SYM_SPELL)
            .allow_action(SYM_ATTACK)
            .max_operations(200)
            .expires_at(expires_at)
            .build_scope();

        if is_a {
            world.session_a_expires = expires_at;
        } else {
            world.session_b_expires = expires_at;
        }

        env.storage().instance().set(&WORLD_KEY, &world);
        expires_at
    }

    // ── Turn Submission ──────────────────────────────────────────────────────

    /// Submit a full turn as an atomic batch of actions.
    pub fn submit_turn(env: Env, player: Address, actions: Vec<Action>) -> TurnResult {
        let mut world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        if world.match_state.status != STATUS_IN_PROGRESS {
            panic_with_error!(&env, GameError::GameOver);
        }

        if player != world.match_state.active_player {
            panic_with_error!(&env, GameError::NotYourTurn);
        }

        let is_a = player == world.player_a;

        let session_expires = if is_a {
            world.session_a_expires
        } else {
            world.session_b_expires
        };
        if session_expires == 0 || env.ledger().timestamp() > session_expires {
            panic_with_error!(&env, GameError::SessionExpired);
        }

        if actions.is_empty() {
            panic_with_error!(&env, GameError::BatchEmpty);
        }

        // DrawSystem and ManaSystem run at the start of each turn.
        draw_system(&env, &mut world, is_a);
        mana_system(&mut world, is_a);

        // Compose BatchBuilder manifest (proves atomicity intent).
        let mut batch = BatchBuilder::new();
        for i in 0..actions.len() {
            let action = actions.get(i).unwrap();
            let sym = match &action {
                Action::PlayCreature(_) => SYM_PLAY,
                Action::CastSpell(_) => SYM_SPELL,
                Action::DeclareAttack(_, _) => SYM_ATTACK,
            };
            batch.add(GameAction {
                system_name: sym,
                data: Bytes::new(&env),
            });
        }

        if batch.is_empty() {
            panic_with_error!(&env, GameError::BatchEmpty);
        }

        // Execute each action atomically - any panic reverts all storage writes.
        let mut executed = 0u32;
        for i in 0..actions.len() {
            let action = actions.get(i).unwrap();
            match action {
                Action::PlayCreature(card_id) => {
                    play_card_system(&env, &mut world, is_a, card_id);
                }
                Action::CastSpell(card_id) => {
                    cast_spell_system(&env, &mut world, is_a, card_id);
                }
                Action::DeclareAttack(attacker_idx, target_idx) => {
                    combat_system(&env, &mut world, is_a, attacker_idx, target_idx);
                }
            }
            executed += 1;
        }

        win_condition_system(&mut world);

        if world.match_state.status == STATUS_IN_PROGRESS {
            world.match_state.turn += 1;
            world.match_state.active_player = if is_a {
                world.player_b.clone()
            } else {
                world.player_a.clone()
            };
            world.match_state.phase = PHASE_DRAW;
        }

        let status = world.match_state.status;
        env.storage().instance().set(&WORLD_KEY, &world);

        TurnResult {
            success: true,
            actions_executed: executed,
            match_status: status,
            message: symbol_short!("ok"),
        }
    }

    // ── Query Methods ─────────────────────────────────────────────────────────

    pub fn get_state(env: Env) -> MatchState {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));
        world.match_state
    }

    pub fn get_hand(env: Env, player: Address) -> Vec<Card> {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        let hand = if player == world.player_a {
            &world.hand_a
        } else {
            &world.hand_b
        };
        let mut result = Vec::new(&env);
        for i in 0..hand.cards.len() {
            let id = hand.cards.get(i).unwrap();
            if let Some(card) = card_definition(id) {
                result.push_back(card);
            }
        }
        result
    }

    pub fn get_field(env: Env) -> FieldState {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));
        FieldState {
            field_a: world.field_a.creatures,
            field_b: world.field_b.creatures,
        }
    }

    pub fn get_stats(env: Env, player: Address) -> PlayerStats {
        let world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));
        if player == world.player_a {
            world.stats_a
        } else {
            world.stats_b
        }
    }

    /// Concede the match.
    pub fn concede(env: Env, player: Address) {
        let mut world: ECSWorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        if world.match_state.status != STATUS_IN_PROGRESS {
            panic_with_error!(&env, GameError::GameOver);
        }

        let is_a = player == world.player_a;
        let is_b = player == world.player_b;
        if !is_a && !is_b {
            panic_with_error!(&env, GameError::NotAPlayer);
        }

        world.match_state.status = STATUS_CONCEDED;
        env.storage().instance().set(&WORLD_KEY, &world);
    }
}

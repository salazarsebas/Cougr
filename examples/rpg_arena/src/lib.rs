#![no_std]

use cougr_core::component::ComponentTrait;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Bytes, Env, Symbol, Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Attack,
    Defend,
    Special,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CombatantComponent {
    pub hp: u32,
    pub defense: u32,
    pub speed: u32,
    pub entity_id: u32,
}

impl ComponentTrait for CombatantComponent {
    fn component_type() -> Symbol {
        symbol_short!("combnnt")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::new(env)
    }
    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct StatusEffectComponent {
    pub effect_kind: u32, // 1 = Poison, 2 = DefenseBuff
    pub duration: u32,
    pub magnitude: u32,
    pub entity_id: u32,
}

impl ComponentTrait for StatusEffectComponent {
    fn component_type() -> Symbol {
        symbol_short!("status")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::new(env)
    }
    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CooldownComponent {
    pub action_kind: u32, // 1 = Special
    pub remaining_turns: u32,
    pub entity_id: u32,
}

impl ComponentTrait for CooldownComponent {
    fn component_type() -> Symbol {
        symbol_short!("cooldown")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::new(env)
    }
    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnComponent {
    pub current_actor: Address,
    pub round: u32,
    pub entity_id: u32,
}

impl ComponentTrait for TurnComponent {
    fn component_type() -> Symbol {
        symbol_short!("turn")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::new(env)
    }
    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BattleStatusComponent {
    pub status: u32, // 0 = Ongoing, 1 = PlayerOneWins, 2 = PlayerTwoWins, 3 = Draw
    pub entity_id: u32,
}

impl ComponentTrait for BattleStatusComponent {
    fn component_type() -> Symbol {
        symbol_short!("bstatus")
    }
    fn serialize(&self, env: &Env) -> Bytes {
        Bytes::new(env)
    }
    fn deserialize(_env: &Env, _data: &Bytes) -> Option<Self> {
        None
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ECSWorldState {
    pub player_one: Address,
    pub player_two: Address,

    pub p1_combatant: CombatantComponent,
    pub p2_combatant: CombatantComponent,

    pub p1_statuses: Vec<StatusEffectComponent>,
    pub p2_statuses: Vec<StatusEffectComponent>,

    pub p1_cooldowns: Vec<CooldownComponent>,
    pub p2_cooldowns: Vec<CooldownComponent>,

    pub turn: TurnComponent,
    pub battle_status: BattleStatusComponent,

    pub next_entity_id: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BattleState {
    pub p1_hp: u32,
    pub p2_hp: u32,
    pub current_turn: Address,
    pub status: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CombatantState {
    pub hp: u32,
    pub defense: u32,
    pub status_count: u32,
    pub cooldown_count: u32,
}

const WORLD_KEY: Symbol = symbol_short!("WORLD");

#[contract]
pub struct RPGArenaContract;

#[contractimpl]
impl RPGArenaContract {
    pub fn init_battle(env: Env, player_one: Address, player_two: Address) {
        let mut next_entity_id = 0u32;

        let p1_combatant = CombatantComponent {
            hp: 100,
            defense: 5,
            speed: 10,
            entity_id: next_entity_id,
        };
        next_entity_id += 1;
        let p2_combatant = CombatantComponent {
            hp: 100,
            defense: 5,
            speed: 10,
            entity_id: next_entity_id,
        };
        next_entity_id += 1;

        let turn = TurnComponent {
            current_actor: player_one.clone(),
            round: 1,
            entity_id: next_entity_id,
        };
        next_entity_id += 1;

        let battle_status = BattleStatusComponent {
            status: 0,
            entity_id: next_entity_id,
        };
        next_entity_id += 1;

        let world_state = ECSWorldState {
            player_one: player_one.clone(),
            player_two: player_two.clone(),
            p1_combatant,
            p2_combatant,
            p1_statuses: Vec::new(&env),
            p2_statuses: Vec::new(&env),
            p1_cooldowns: Vec::new(&env),
            p2_cooldowns: Vec::new(&env),
            turn,
            battle_status,
            next_entity_id,
        };

        env.storage().instance().set(&WORLD_KEY, &world_state);
    }

    pub fn submit_action(env: Env, player: Address, action: Action) {
        let mut world: ECSWorldState = env.storage().instance().get(&WORLD_KEY).unwrap();

        // System 1: Action Validation System
        let val = Self::action_validation_system(&world, &player, &action);
        if !val.0 {
            panic!("Invalid action: {:?}", val.1);
        }

        // System 2 & 3: Damage Resolution & Status Application System
        Self::damage_resolution_system(&env, &mut world, &player, &action);

        // System 4: End Condition System (Check immediate impact)
        Self::end_condition_system(&mut world);

        if world.battle_status.status == 0 {
            // System 5: Status Effect System (Apply poison, decrement status turns)
            Self::status_effect_system(&env, &mut world);

            // System 6: Cooldown System (Decrement cooldowns)
            Self::cooldown_system(&env, &mut world);

            // Turn Advance
            Self::turn_advance_system(&mut world);

            // Recheck End Condition after status effect (like poison damage)
            Self::end_condition_system(&mut world);
        }

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn get_state(env: Env) -> BattleState {
        let world: ECSWorldState = env.storage().instance().get(&WORLD_KEY).unwrap();
        Self::to_battle_state(&world)
    }

    pub fn get_combatant(env: Env, player: Address) -> CombatantState {
        let world: ECSWorldState = env.storage().instance().get(&WORLD_KEY).unwrap();
        if player == world.player_one {
            CombatantState {
                hp: world.p1_combatant.hp,
                defense: world.p1_combatant.defense,
                status_count: world.p1_statuses.len(),
                cooldown_count: world.p1_cooldowns.len(),
            }
        } else if player == world.player_two {
            CombatantState {
                hp: world.p2_combatant.hp,
                defense: world.p2_combatant.defense,
                status_count: world.p2_statuses.len(),
                cooldown_count: world.p2_cooldowns.len(),
            }
        } else {
            panic!("Unknown player");
        }
    }

    pub fn is_finished(env: Env) -> bool {
        let world: ECSWorldState = env.storage().instance().get(&WORLD_KEY).unwrap();
        world.battle_status.status != 0
    }

    // --- Systems ---

    fn action_validation_system(
        world: &ECSWorldState,
        player: &Address,
        action: &Action,
    ) -> (bool, Symbol) {
        if world.battle_status.status != 0 {
            return (false, symbol_short!("gameover"));
        }
        if world.turn.current_actor != *player {
            return (false, symbol_short!("notturn"));
        }

        // Check cooldowns
        let is_p1 = *player == world.player_one;
        let cooldowns = if is_p1 {
            &world.p1_cooldowns
        } else {
            &world.p2_cooldowns
        };

        if *action == Action::Special {
            for i in 0..cooldowns.len() {
                let cd = cooldowns.get(i).unwrap();
                if cd.action_kind == 1 && cd.remaining_turns > 0 {
                    return (false, symbol_short!("cooldown"));
                }
            }
        }

        (true, symbol_short!("ok"))
    }

    fn damage_resolution_system(
        env: &Env,
        world: &mut ECSWorldState,
        player: &Address,
        action: &Action,
    ) {
        let is_p1 = *player == world.player_one;

        let (
            attacker_stats,
            defender_stats,
            defender_statuses,
            attacker_statuses,
            attacker_cooldowns,
        ) = if is_p1 {
            (
                &mut world.p1_combatant,
                &mut world.p2_combatant,
                &mut world.p2_statuses,
                &mut world.p1_statuses,
                &mut world.p1_cooldowns,
            )
        } else {
            (
                &mut world.p2_combatant,
                &mut world.p1_combatant,
                &mut world.p1_statuses,
                &mut world.p2_statuses,
                &mut world.p2_cooldowns,
            )
        };

        // Determine effective defense (base + DefBuff)
        let mut def = defender_stats.defense;
        for i in 0..defender_statuses.len() {
            let st = defender_statuses.get(i).unwrap();
            if st.effect_kind == 2 {
                // DefBuff
                def += st.magnitude;
            }
        }

        match action {
            Action::Attack => {
                let dmg = 15u32.saturating_sub(def);
                defender_stats.hp = defender_stats.hp.saturating_sub(dmg);
            }
            Action::Defend => {
                // Apply a DefBuff for 1 turn
                let mut buff_id = world.next_entity_id;
                world.next_entity_id += 1;
                attacker_statuses.push_back(StatusEffectComponent {
                    effect_kind: 2,
                    duration: 1, // Will apply to incoming damage this turn / next turn
                    magnitude: 10,
                    entity_id: buff_id,
                });
            }
            Action::Special => {
                let dmg = 25u32.saturating_sub(def);
                defender_stats.hp = defender_stats.hp.saturating_sub(dmg);

                // Add poison status
                let mut psn_id = world.next_entity_id;
                world.next_entity_id += 1;
                defender_statuses.push_back(StatusEffectComponent {
                    effect_kind: 1,
                    duration: 3,
                    magnitude: 5,
                    entity_id: psn_id,
                });

                // Add cooldown
                let mut cd_id = world.next_entity_id;
                world.next_entity_id += 1;
                attacker_cooldowns.push_back(CooldownComponent {
                    action_kind: 1,
                    remaining_turns: 3,
                    entity_id: cd_id,
                });
            }
        }
    }

    fn status_effect_system(env: &Env, world: &mut ECSWorldState) {
        // Apply end-of-turn status impacts (e.g. Poison) to both players
        // Actually, normally statuses are resolved exactly for the player whose turn it just ended,
        // to not double-tick. Let's do it for the current actor.
        let is_p1 = world.turn.current_actor == world.player_one;

        let mut updated_statuses = Vec::new(env);

        if is_p1 {
            for i in 0..world.p1_statuses.len() {
                let mut st = world.p1_statuses.get(i).unwrap();
                if st.effect_kind == 1 {
                    // Poison
                    world.p1_combatant.hp = world.p1_combatant.hp.saturating_sub(st.magnitude);
                }
                if st.duration > 1 {
                    st.duration -= 1;
                    updated_statuses.push_back(st);
                }
            }
            world.p1_statuses = updated_statuses;
        } else {
            for i in 0..world.p2_statuses.len() {
                let mut st = world.p2_statuses.get(i).unwrap();
                if st.effect_kind == 1 {
                    // Poison
                    world.p2_combatant.hp = world.p2_combatant.hp.saturating_sub(st.magnitude);
                }
                if st.duration > 1 {
                    st.duration -= 1;
                    updated_statuses.push_back(st);
                }
            }
            world.p2_statuses = updated_statuses;
        }
    }

    fn cooldown_system(env: &Env, world: &mut ECSWorldState) {
        // Decrement cooldowns for current actor
        let is_p1 = world.turn.current_actor == world.player_one;
        let mut updated_cds = Vec::new(env);

        if is_p1 {
            for i in 0..world.p1_cooldowns.len() {
                let mut cd = world.p1_cooldowns.get(i).unwrap();
                if cd.remaining_turns > 1 {
                    cd.remaining_turns -= 1;
                    updated_cds.push_back(cd);
                }
            }
            world.p1_cooldowns = updated_cds;
        } else {
            for i in 0..world.p2_cooldowns.len() {
                let mut cd = world.p2_cooldowns.get(i).unwrap();
                if cd.remaining_turns > 1 {
                    cd.remaining_turns -= 1;
                    updated_cds.push_back(cd);
                }
            }
            world.p2_cooldowns = updated_cds;
        }
    }

    fn end_condition_system(world: &mut ECSWorldState) {
        if world.p1_combatant.hp == 0 && world.p2_combatant.hp == 0 {
            world.battle_status.status = 3; // Draw
        } else if world.p2_combatant.hp == 0 {
            world.battle_status.status = 1; // P1 wins
        } else if world.p1_combatant.hp == 0 {
            world.battle_status.status = 2; // P2 wins
        }
    }

    fn turn_advance_system(world: &mut ECSWorldState) {
        if world.turn.current_actor == world.player_one {
            world.turn.current_actor = world.player_two.clone();
        } else {
            world.turn.current_actor = world.player_one.clone();
            world.turn.round += 1;
        }
    }

    // --- Helpers ---
    fn to_battle_state(world: &ECSWorldState) -> BattleState {
        BattleState {
            p1_hp: world.p1_combatant.hp,
            p2_hp: world.p2_combatant.hp,
            current_turn: world.turn.current_actor.clone(),
            status: world.battle_status.status,
        }
    }
}

#[cfg(test)]
mod test;

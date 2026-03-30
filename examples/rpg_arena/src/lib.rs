#![no_std]

use cougr_core::{impl_component, impl_marker_component};
use cougr_core::simple_world::{SimpleWorld, EntityId};
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, Symbol,
};

// --- Contract Types ---

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Attack,
    Defend,
    Special,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffectKind {
    None = 0,
    Poison = 1,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatantState {
    pub player: Address,
    pub hp: i32,
    pub max_hp: i32,
    pub defense: i32,
    pub speed: i32,
    pub status_effect: EffectKind,
    pub status_duration: u32,
    pub is_defending: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleState {
    pub p1: CombatantState,
    pub p2: CombatantState,
    pub round: u32,
    pub current_turn: Address,
    pub is_finished: bool,
    pub winner: Option<Address>,
}

// --- Components ---

#[contracttype]
#[derive(Clone, Debug)]
pub struct CombatantComponent {
    pub hp: i32,
    pub max_hp: i32,
    pub defense: i32,
    pub speed: i32,
}

impl_component!(CombatantComponent, "cbt", Table, {
    hp: i32,
    max_hp: i32,
    defense: i32,
    speed: i32
});

#[contracttype]
#[derive(Clone, Debug)]
pub struct StatusEffectComponent {
    pub effect_kind: u32, // 0: None, 1: Poison
    pub duration: u32,
    pub magnitude: i32,
}

impl_component!(StatusEffectComponent, "status", Table, {
    effect_kind: u32,
    duration: u32,
    magnitude: i32
});

#[contracttype]
#[derive(Clone, Debug)]
pub struct CooldownComponent {
    pub remaining_turns: u32,
}

impl_component!(CooldownComponent, "cooldown", Table, {
    remaining_turns: u32
});

#[contracttype]
#[derive(Clone, Debug)]
pub struct TurnComponent {
    pub current_actor: u32, // 1 for P1, 2 for P2
    pub round: u32,
}

impl_component!(TurnComponent, "turn", Table, {
    current_actor: u32,
    round: u32
});

#[contracttype]
#[derive(Clone, Debug)]
pub struct BattleStatusComponent {
    pub status: u32, // 0: Ongoing, 1: P1 Wins, 2: P2 Wins
}

impl_component!(BattleStatusComponent, "bst", Table, {
    status: u32
});

// Marker for "Defending" stance (Sparse storage)
pub struct DefendingMarker;
impl_marker_component!(DefendingMarker, "defend", Sparse);

// --- Storage Keys ---

const CONFIG_KEY: Symbol = symbol_short!("CONFIG");
const WORLD_KEY: Symbol = symbol_short!("WORLD");

#[contracttype]
#[derive(Clone, Debug)]
pub struct BattleConfig {
    pub p1: Address,
    pub p2: Address,
    pub p1_entity: EntityId,
    pub p2_entity: EntityId,
    pub global_entity: EntityId,
}

// --- Contract Implementation ---

#[contract]
pub struct RpgArenaContract;

#[contractimpl]
impl RpgArenaContract {
    pub fn init_battle(env: Env, player_one: Address, player_two: Address) {
        player_one.require_auth();
        
        let mut world = SimpleWorld::new(&env);
        
        // Spawn entities
        let p1_entity = world.spawn_entity();
        let p2_entity = world.spawn_entity();
        let global_entity = world.spawn_entity();

        let config = BattleConfig {
            p1: player_one.clone(),
            p2: player_two.clone(),
            p1_entity,
            p2_entity,
            global_entity,
        };
        env.storage().instance().set(&CONFIG_KEY, &config);

        // Initialize P1
        world.set_typed(&env, p1_entity, &CombatantComponent { hp: 100, max_hp: 100, defense: 10, speed: 10 });
        world.set_typed(&env, p1_entity, &StatusEffectComponent { effect_kind: 0, duration: 0, magnitude: 0 });
        world.set_typed(&env, p1_entity, &CooldownComponent { remaining_turns: 0 });

        // Initialize P2
        world.set_typed(&env, p2_entity, &CombatantComponent { hp: 100, max_hp: 100, defense: 10, speed: 10 });
        world.set_typed(&env, p2_entity, &StatusEffectComponent { effect_kind: 0, duration: 0, magnitude: 0 });
        world.set_typed(&env, p2_entity, &CooldownComponent { remaining_turns: 0 });

        // Initialize Global state
        world.set_typed(&env, global_entity, &TurnComponent { current_actor: 1, round: 1 });
        world.set_typed(&env, global_entity, &BattleStatusComponent { status: 0 });

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn submit_action(env: Env, player: Address, action: Action) {
        player.require_auth();
        
        let config: BattleConfig = env.storage().instance().get(&CONFIG_KEY).unwrap();
        let mut world: SimpleWorld = env.storage().instance().get(&WORLD_KEY).unwrap();

        // 1. Action Validation System
        Self::action_validation_system(&config, &world, &player, &action);

        // 2. Pre-action Processing (Status Effects, Cooldowns)
        Self::status_effect_system(&env, &config, &mut world);
        Self::cooldown_system(&env, &config, &mut world);

        // If status system finished the battle, skip execution
        let bst: BattleStatusComponent = world.get_typed(&env, config.global_entity).unwrap();
        if bst.status == 0 {
            // 3. Execution System
            Self::execution_system(&env, &config, &mut world, &action);
            
            // 4. End Condition System
            Self::end_condition_system(&env, &config, &mut world);
        }

        // 5. Turn Advance System
        let bst_check: BattleStatusComponent = world.get_typed(&env, config.global_entity).unwrap();
        if bst_check.status == 0 {
            Self::turn_advance_system(&env, &config, &mut world);
        }

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    pub fn get_state(env: Env) -> BattleState {
        let config: BattleConfig = env.storage().instance().get(&CONFIG_KEY).unwrap();
        let world: SimpleWorld = env.storage().instance().get(&WORLD_KEY).unwrap();
        let turn: TurnComponent = world.get_typed(&env, config.global_entity).unwrap();
        let bst: BattleStatusComponent = world.get_typed(&env, config.global_entity).unwrap();

        let current_turn = if turn.current_actor == 1 {
            config.p1.clone()
        } else {
            config.p2.clone()
        };

        let winner = match bst.status {
            1 => Some(config.p1.clone()),
            2 => Some(config.p2.clone()),
            _ => None,
        };

        BattleState {
            p1: Self::map_combatant(&env, &config.p1, config.p1_entity, &world),
            p2: Self::map_combatant(&env, &config.p2, config.p2_entity, &world),
            round: turn.round,
            current_turn,
            is_finished: bst.status != 0,
            winner,
        }
    }

    pub fn get_combatant(env: Env, player: Address) -> CombatantState {
        let config: BattleConfig = env.storage().instance().get(&CONFIG_KEY).unwrap();
        let world: SimpleWorld = env.storage().instance().get(&WORLD_KEY).unwrap();

        if player == config.p1 {
            Self::map_combatant(&env, &config.p1, config.p1_entity, &world)
        } else if player == config.p2 {
            Self::map_combatant(&env, &config.p2, config.p2_entity, &world)
        } else {
            panic!("Player not in battle");
        }
    }

    pub fn is_finished(env: Env) -> bool {
        let config: BattleConfig = env.storage().instance().get(&CONFIG_KEY).unwrap();
        let world: SimpleWorld = env.storage().instance().get(&WORLD_KEY).unwrap();
        let bst: BattleStatusComponent = world.get_typed(&env, config.global_entity).unwrap();
        bst.status != 0
    }

    // --- Systems ---

    fn action_validation_system(config: &BattleConfig, world: &SimpleWorld, player: &Address, action: &Action) {
        let env = world.components.env();
        let bst: BattleStatusComponent = world.get_typed(env, config.global_entity).expect("BST");
        if bst.status != 0 {
            panic!("Battle finished");
        }

        let is_p1 = *player == config.p1;
        let is_p2 = *player == config.p2;
        if !is_p1 && !is_p2 { panic!("Not in battle"); }

        let turn: TurnComponent = world.get_typed(env, config.global_entity).expect("Turn");
        if (is_p1 && turn.current_actor != 1) || (is_p2 && turn.current_actor != 2) {
            panic!("Not your turn");
        }

        if let Action::Special = action {
            let entity = if is_p1 { config.p1_entity } else { config.p2_entity };
            let cd: CooldownComponent = world.get_typed(env, entity).expect("CD");
            if cd.remaining_turns > 0 {
                panic!("Special ability is on cooldown");
            }
        }
    }

    fn execution_system(env: &Env, config: &BattleConfig, world: &mut SimpleWorld, action: &Action) {
        let turn: TurnComponent = world.get_typed(env, config.global_entity).unwrap();
        let is_p1_turn = turn.current_actor == 1;
        
        match action {
            Action::Attack => {
                Self::apply_damage(env, world, !is_p1_turn, 20, config);
                // Clear defense stance when attacking
                let actor_entity = if is_p1_turn { config.p1_entity } else { config.p2_entity };
                world.remove_typed::<DefendingMarker>(actor_entity);
            },
            Action::Defend => {
                let actor_entity = if is_p1_turn { config.p1_entity } else { config.p2_entity };
                world.set_typed(env, actor_entity, &DefendingMarker);
            },
            Action::Special => {
                Self::apply_damage(env, world, !is_p1_turn, 10, config);
                
                let (actor_entity, target_entity) = if is_p1_turn {
                    (config.p1_entity, config.p2_entity)
                } else {
                    (config.p2_entity, config.p1_entity)
                };

                world.set_typed(env, target_entity, &StatusEffectComponent {
                    effect_kind: 1, // Poison
                    duration: 3,
                    magnitude: 10,
                });
                world.set_typed(env, actor_entity, &CooldownComponent { remaining_turns: 3 });
                world.remove_typed::<DefendingMarker>(actor_entity);
            }
        }
    }

    fn apply_damage(env: &Env, world: &mut SimpleWorld, to_p1: bool, base_damage: i32, config: &BattleConfig) {
        let target_entity = if to_p1 { config.p1_entity } else { config.p2_entity };
        let mut cbt: CombatantComponent = world.get_typed(env, target_entity).unwrap();
        let is_defending = world.has_typed::<DefendingMarker>(target_entity);

        let mut actual_damage = base_damage - cbt.defense / 2;
        if is_defending { actual_damage /= 2; }
        if actual_damage < 0 { actual_damage = 0; }

        cbt.hp -= actual_damage;
        if cbt.hp < 0 { cbt.hp = 0; }
        world.set_typed(env, target_entity, &cbt);
    }

    fn status_effect_system(env: &Env, config: &BattleConfig, world: &mut SimpleWorld) {
        let turn: TurnComponent = world.get_typed(env, config.global_entity).unwrap();
        let actor_entity = if turn.current_actor == 1 { config.p1_entity } else { config.p2_entity };
        
        let mut cbt: CombatantComponent = world.get_typed(env, actor_entity).unwrap();
        let mut status: StatusEffectComponent = world.get_typed(env, actor_entity).unwrap();

        if status.effect_kind == 1 && status.duration > 0 {
            cbt.hp -= status.magnitude;
            if cbt.hp < 0 { cbt.hp = 0; }
            status.duration -= 1;
            if status.duration == 0 { status.effect_kind = 0; }
            
            world.set_typed(env, actor_entity, &cbt);
            world.set_typed(env, actor_entity, &status);
        }

        Self::end_condition_system(env, config, world);
    }

    fn cooldown_system(env: &Env, config: &BattleConfig, world: &mut SimpleWorld) {
        let turn: TurnComponent = world.get_typed(env, config.global_entity).unwrap();
        let actor_entity = if turn.current_actor == 1 { config.p1_entity } else { config.p2_entity };
        
        let mut cd: CooldownComponent = world.get_typed(env, actor_entity).unwrap();
        if cd.remaining_turns > 0 {
            cd.remaining_turns -= 1;
            world.set_typed(env, actor_entity, &cd);
        }
    }

    fn turn_advance_system(env: &Env, config: &BattleConfig, world: &mut SimpleWorld) {
        let mut turn: TurnComponent = world.get_typed(env, config.global_entity).unwrap();
        if turn.current_actor == 1 {
            turn.current_actor = 2;
        } else {
            turn.current_actor = 1;
            turn.round += 1;
        }
        world.set_typed(env, config.global_entity, &turn);
    }

    fn end_condition_system(env: &Env, config: &BattleConfig, world: &mut SimpleWorld) {
        let p1_cbt: CombatantComponent = world.get_typed(env, config.p1_entity).unwrap();
        let p2_cbt: CombatantComponent = world.get_typed(env, config.p2_entity).unwrap();
        
        if p1_cbt.hp == 0 || p2_cbt.hp == 0 {
            let status = if p1_cbt.hp == 0 { 2 } else { 1 };
            world.set_typed(env, config.global_entity, &BattleStatusComponent { status });
        }
    }

    fn map_combatant(env: &Env, addr: &Address, entity: EntityId, world: &SimpleWorld) -> CombatantState {
        let cbt: CombatantComponent = world.get_typed(env, entity).unwrap();
        let status: StatusEffectComponent = world.get_typed(env, entity).unwrap();
        let is_defending = world.has_typed::<DefendingMarker>(entity);

        CombatantState {
            player: addr.clone(),
            hp: cbt.hp,
            max_hp: cbt.max_hp,
            defense: cbt.defense,
            speed: cbt.speed,
            status_effect: if status.effect_kind == 1 { EffectKind::Poison } else { EffectKind::None },
            status_duration: status.duration,
            is_defending,
        }
    }
}

#[cfg(test)]
mod test;

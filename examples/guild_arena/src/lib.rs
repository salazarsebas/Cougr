#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec,
};

use cougr_core::auth::{
    DeviceManager, DevicePolicy, MultiDeviceProvider, RecoverableAccount, RecoveryConfig,
    RecoveryProvider,
};
use cougr_core::component::ComponentTrait;

// --- Components ---

#[contracttype]
#[derive(Clone, Debug)]
pub struct Fighter {
    pub health: u32,
    pub attack: u32,
    pub defense: u32,
    pub level: u32,
}

impl ComponentTrait for Fighter {
    fn component_type() -> Symbol {
        symbol_short!("fighter")
    }

    fn serialize(&self, env: &Env) -> soroban_sdk::Bytes {
        let mut bytes = soroban_sdk::Bytes::new(env);
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.health.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.attack.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.defense.to_be_bytes(),
        ));
        bytes.append(&soroban_sdk::Bytes::from_array(
            env,
            &self.level.to_be_bytes(),
        ));
        bytes
    }

    fn deserialize(_env: &Env, data: &soroban_sdk::Bytes) -> Option<Self> {
        if data.len() != 16 {
            return None;
        }
        let health = u32::from_be_bytes([data.get(0)?, data.get(1)?, data.get(2)?, data.get(3)?]);
        let attack = u32::from_be_bytes([data.get(4)?, data.get(5)?, data.get(6)?, data.get(7)?]);
        let defense =
            u32::from_be_bytes([data.get(8)?, data.get(9)?, data.get(10)?, data.get(11)?]);
        let level =
            u32::from_be_bytes([data.get(12)?, data.get(13)?, data.get(14)?, data.get(15)?]);
        Some(Self {
            health,
            attack,
            defense,
            level,
        })
    }
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MatchRecord {
    pub wins: u32,
    pub losses: u32,
    pub rating: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum GuildRole {
    Leader = 0,
    Officer = 1,
    Member = 2,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GuildMembership {
    pub guild_id: u32,
    pub role: GuildRole,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProfile {
    pub address: Address,
    pub fighter: Fighter,
    pub record: MatchRecord,
    pub guild: GuildMembership,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MatchStatus {
    WaitingForPlayers = 0,
    InProgress = 1,
    Finished = 2,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ArenaState {
    pub challenger: Address,
    pub defender: Address,
    pub turn: Address,
    pub challenger_hp: u32,
    pub defender_hp: u32,
    pub round: u32,
    pub status: MatchStatus,
    pub winner: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CombatAction {
    Attack = 0,
    Defend = 1,
    Special = 2,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RoundResult {
    pub round: u32,
    pub challenger_hp: u32,
    pub defender_hp: u32,
    pub finished: bool,
    pub winner: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DevicePolicyLevel {
    Full = 0,
    PlayOnly = 1,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct DevicePolicyEntry {
    pub level: DevicePolicyLevel,
}

// --- Storage Keys ---

const PLAYER_KEY: Symbol = symbol_short!("PLAYER");
const ARENA_KEY: Symbol = symbol_short!("ARENA");
const DEVICE_POL: Symbol = symbol_short!("DEV_POL");
const QUEUE_KEY: Symbol = symbol_short!("QUEUE");
const DEVOWNER: Symbol = symbol_short!("DEVOWNER");

fn player_key(_env: &Env, addr: &Address) -> (Symbol, Address) {
    (PLAYER_KEY, addr.clone())
}

fn device_policy_key(_env: &Env, device: &Address) -> (Symbol, Address) {
    (DEVICE_POL, device.clone())
}

fn device_owner_key(_env: &Env, device: &Address) -> (Symbol, Address) {
    (DEVOWNER, device.clone())
}

// --- Contract ---

#[contract]
pub struct GuildArenaContract;

#[contractimpl]
impl GuildArenaContract {
    pub fn register_player(
        env: Env,
        player: Address,
        guardians: Vec<Address>,
        threshold: u32,
        timelock: u64,
    ) {
        player.require_auth();
        if env.storage().persistent().has(&player_key(&env, &player)) {
            panic!("player already registered");
        }

        let config = RecoveryConfig {
            threshold,
            timelock_period: timelock,
            max_guardians: 5,
        };
        let mut account = RecoverableAccount::new(player.clone(), config, &env);
        for i in 0..guardians.len() {
            if let Some(g) = guardians.get(i) {
                account.add_guardian(&env, g).unwrap();
            }
        }

        let device_policy = DevicePolicy {
            max_devices: 5,
            auto_revoke_after: 0,
        };
        let _dm = DeviceManager::new(player.clone(), device_policy, &env);

        let profile = PlayerProfile {
            address: player.clone(),
            fighter: Fighter {
                health: 100,
                attack: 15,
                defense: 10,
                level: 1,
            },
            record: MatchRecord {
                wins: 0,
                losses: 0,
                rating: 1200,
            },
            guild: GuildMembership {
                guild_id: 1,
                role: GuildRole::Member,
            },
        };

        env.storage()
            .persistent()
            .set(&player_key(&env, &player), &profile);
    }

    pub fn add_device(env: Env, player: Address, device_key: Address, policy: DevicePolicyEntry) {
        player.require_auth();
        device_key.require_auth();

        let existing_owner: Option<Address> = env
            .storage()
            .persistent()
            .get(&device_owner_key(&env, &device_key));
        if existing_owner.is_some() {
            panic!("device already registered");
        }

        let mut dm = DeviceManager::load(player.clone());
        let key_bytes = Self::addr_to_bytes32(&env, &device_key);
        dm.register_device(&env, key_bytes, symbol_short!("device"))
            .unwrap();

        env.storage()
            .persistent()
            .set(&device_policy_key(&env, &device_key), &policy);
        env.storage()
            .persistent()
            .set(&device_owner_key(&env, &device_key), &player);
    }

    pub fn remove_device(env: Env, player: Address, device_key: Address) {
        player.require_auth();

        let mut dm = DeviceManager::load(player.clone());
        let key_bytes = Self::addr_to_bytes32(&env, &device_key);
        dm.revoke_device(&env, &key_bytes).unwrap();

        env.storage()
            .persistent()
            .remove(&device_policy_key(&env, &device_key));
        env.storage()
            .persistent()
            .remove(&device_owner_key(&env, &device_key));
    }

    pub fn start_match(env: Env, device_key: Address) {
        device_key.require_auth();

        let player: Address = env
            .storage()
            .persistent()
            .get(&device_owner_key(&env, &device_key))
            .unwrap_or_else(|| panic!("device not registered"));

        Self::require_play_permission(&env, &device_key);

        let profile: PlayerProfile = env
            .storage()
            .persistent()
            .get(&player_key(&env, &player))
            .unwrap_or_else(|| panic!("player not registered"));

        let active_arena: Option<ArenaState> = env.storage().persistent().get(&ARENA_KEY);
        if let Some(arena) = active_arena {
            if arena.status == MatchStatus::InProgress {
                panic!("match already in progress");
            }
        }

        let existing: Option<Address> = env.storage().persistent().get(&QUEUE_KEY);

        if let Some(challenger_addr) = existing {
            if challenger_addr == player {
                panic!("already in queue");
            }

            let challenger_profile: PlayerProfile = env
                .storage()
                .persistent()
                .get(&player_key(&env, &challenger_addr))
                .unwrap_or_else(|| panic!("challenger not found"));

            let arena = ArenaState {
                challenger: challenger_addr.clone(),
                defender: player.clone(),
                turn: challenger_addr.clone(),
                challenger_hp: challenger_profile.fighter.health,
                defender_hp: profile.fighter.health,
                round: 0,
                status: MatchStatus::InProgress,
                winner: challenger_addr.clone(), // placeholder
            };
            env.storage().persistent().set(&ARENA_KEY, &arena);
            env.storage().persistent().remove(&QUEUE_KEY);
        } else {
            env.storage().persistent().set(&QUEUE_KEY, &player);
        }
    }

    pub fn submit_action(env: Env, device_key: Address, action: CombatAction) -> RoundResult {
        device_key.require_auth();

        let player: Address = env
            .storage()
            .persistent()
            .get(&device_owner_key(&env, &device_key))
            .unwrap_or_else(|| panic!("device not registered"));

        Self::require_play_permission(&env, &device_key);

        let mut arena: ArenaState = env
            .storage()
            .persistent()
            .get(&ARENA_KEY)
            .unwrap_or_else(|| panic!("no active match"));

        if arena.status != MatchStatus::InProgress {
            panic!("match not in progress");
        }

        let is_challenger = arena.challenger == player;
        if !is_challenger && arena.defender != player {
            panic!("not a participant");
        }
        if arena.turn != player {
            panic!("not your turn");
        }

        let challenger_profile: PlayerProfile = env
            .storage()
            .persistent()
            .get(&player_key(&env, &arena.challenger))
            .unwrap_or_else(|| panic!("challenger not found"));

        let defender_profile: PlayerProfile = env
            .storage()
            .persistent()
            .get(&player_key(&env, &arena.defender))
            .unwrap_or_else(|| panic!("defender not found"));

        let (atk_stat, def_stat) = if is_challenger {
            (
                challenger_profile.fighter.attack,
                defender_profile.fighter.defense,
            )
        } else {
            (
                defender_profile.fighter.attack,
                challenger_profile.fighter.defense,
            )
        };

        let damage = Self::compute_damage(atk_stat, def_stat, &action);

        if is_challenger {
            arena.defender_hp = arena.defender_hp.saturating_sub(damage);
        } else {
            arena.challenger_hp = arena.challenger_hp.saturating_sub(damage);
        }

        arena.round += 1;

        let mut finished = false;
        if arena.challenger_hp == 0 {
            arena.status = MatchStatus::Finished;
            arena.winner = arena.defender.clone();
            finished = true;
        } else if arena.defender_hp == 0 {
            arena.status = MatchStatus::Finished;
            arena.winner = arena.challenger.clone();
            finished = true;
        }

        arena.turn = if is_challenger {
            arena.defender.clone()
        } else {
            arena.challenger.clone()
        };

        if finished {
            Self::update_ratings(&env, &arena);
        }

        let result = RoundResult {
            round: arena.round,
            challenger_hp: arena.challenger_hp,
            defender_hp: arena.defender_hp,
            finished,
            winner: arena.winner.clone(),
        };

        env.storage().persistent().set(&ARENA_KEY, &arena);
        result
    }

    pub fn initiate_recovery(env: Env, guardian: Address, player: Address, new_key: Address) {
        guardian.require_auth();

        let mut account = RecoverableAccount::load(player.clone());
        account.initiate_recovery(&env, new_key).unwrap();
        account.approve_recovery(&env, &guardian).unwrap();
    }

    pub fn approve_recovery(env: Env, guardian: Address, player: Address, new_key: Address) {
        guardian.require_auth();

        let mut account = RecoverableAccount::load(player.clone());

        if account.active_request(&env).is_none() {
            account.initiate_recovery(&env, new_key).unwrap();
            account.approve_recovery(&env, &guardian).unwrap();
            return;
        }

        account.approve_recovery(&env, &guardian).unwrap();
    }

    pub fn finalize_recovery(env: Env, player: Address) {
        let mut account = RecoverableAccount::load(player.clone());
        let new_owner = account.execute_recovery(&env).unwrap();

        let profile: Option<PlayerProfile> =
            env.storage().persistent().get(&player_key(&env, &player));

        if let Some(mut p) = profile {
            p.address = new_owner.clone();
            env.storage()
                .persistent()
                .set(&player_key(&env, &new_owner), &p);
        }
    }

    pub fn get_player(env: Env, player: Address) -> PlayerProfile {
        env.storage()
            .persistent()
            .get(&player_key(&env, &player))
            .unwrap_or_else(|| panic!("player not found"))
    }

    pub fn get_match(env: Env) -> ArenaState {
        env.storage()
            .persistent()
            .get(&ARENA_KEY)
            .unwrap_or_else(|| panic!("no active match"))
    }

    // --- Internal helpers ---

    fn compute_damage(atk: u32, def: u32, action: &CombatAction) -> u32 {
        let base = if atk > def { atk - def } else { 1 };
        match action {
            CombatAction::Attack => base + 5,
            CombatAction::Defend => base.saturating_sub(3).max(1),
            CombatAction::Special => base + 12,
        }
    }

    fn update_ratings(env: &Env, arena: &ArenaState) {
        let mut winner_profile: PlayerProfile = env
            .storage()
            .persistent()
            .get(&player_key(env, &arena.winner))
            .unwrap();

        let loser_addr = if arena.winner == arena.challenger {
            &arena.defender
        } else {
            &arena.challenger
        };

        let mut loser_profile: PlayerProfile = env
            .storage()
            .persistent()
            .get(&player_key(env, loser_addr))
            .unwrap();

        let k: u32 = 32;

        winner_profile.record.wins += 1;
        winner_profile.record.rating = winner_profile.record.rating.saturating_add(k);

        loser_profile.record.losses += 1;
        loser_profile.record.rating = loser_profile.record.rating.saturating_sub(k);

        if winner_profile.record.wins.is_multiple_of(3) {
            winner_profile.fighter.level += 1;
            winner_profile.fighter.attack += 2;
            winner_profile.fighter.defense += 1;
            winner_profile.fighter.health += 10;
        }

        env.storage()
            .persistent()
            .set(&player_key(env, &arena.winner), &winner_profile);
        env.storage()
            .persistent()
            .set(&player_key(env, loser_addr), &loser_profile);
    }

    fn require_play_permission(env: &Env, device_key: &Address) {
        let policy: Option<DevicePolicyEntry> = env
            .storage()
            .persistent()
            .get(&device_policy_key(env, device_key));

        if policy.is_none() {
            panic!("device has no policy");
        }
    }

    pub fn require_admin_permission(env: &Env, device_key: &Address) {
        let policy: DevicePolicyEntry = env
            .storage()
            .persistent()
            .get(&device_policy_key(env, device_key))
            .unwrap_or_else(|| panic!("device has no policy"));

        if policy.level != DevicePolicyLevel::Full {
            panic!("insufficient device permissions");
        }
    }

    fn addr_to_bytes32(env: &Env, addr: &Address) -> BytesN<32> {
        let raw = addr.to_string();
        let bytes = raw.to_bytes();
        let mut arr = [0u8; 32];
        let len = bytes.len();
        let start = len.saturating_sub(32);
        for i in 0..len.min(32) {
            if let Some(b) = bytes.get(start + i) {
                arr[i as usize] = b;
            }
        }
        BytesN::from_array(env, &arr)
    }
}

#[cfg(test)]
mod test;

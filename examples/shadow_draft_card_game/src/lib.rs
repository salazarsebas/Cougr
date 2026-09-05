#![no_std]

extern crate alloc;
use alloc::vec::Vec as RustVec;

use cougr_core::zk::{experimental, Groth16Proof, Scalar, VerificationKey};
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Bytes, BytesN, Env, Symbol, Vec,
};

// ─── Phases ───────────────────────────────────────────────────────────────────

/// Draft phase: both players submit sealed hand commitments.
pub const PHASE_DRAFT: u32 = 0;
/// Play phase: both players reveal their card and submit a legality proof.
pub const PHASE_PLAY: u32 = 1;
/// Resolution phase: the round is being resolved (transitional).
pub const PHASE_RESOLUTION: u32 = 2;
/// Finished: the match has ended.
pub const PHASE_FINISHED: u32 = 3;

// ─── Match status ─────────────────────────────────────────────────────────────

pub const STATUS_IN_PROGRESS: u32 = 0;
pub const STATUS_PLAYER_ONE_WINS: u32 = 1;
pub const STATUS_PLAYER_TWO_WINS: u32 = 2;
pub const STATUS_DRAW: u32 = 3;

// ─── Proposal status ─────────────────────────────────────────────────────────

pub const PROPOSAL_PENDING: u32 = 0;
pub const PROPOSAL_ACCEPTED: u32 = 1;

// ─── Game constants ───────────────────────────────────────────────────────────

/// Number of rounds a player must win to claim the match.
pub const ROUNDS_TO_WIN: u32 = 3;
/// Highest valid card ID in the default card set.
pub const MAX_CARD_ID: u32 = 10;

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GameError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotAPlayer = 3,
    WrongPhase = 4,
    AlreadyCommitted = 5,
    AlreadyRevealed = 6,
    NotCommitted = 7,
    InvalidProof = 8,
    CardBanned = 9,
    InvalidCard = 10,
    CommitmentMismatch = 11,
    GameOver = 12,
}

// ─── Components ───────────────────────────────────────────────────────────────

/// Tracks deck-selection context and the active season format for the match.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DeckComponent {
    pub player: Address,
    pub active_format: u32,
}

/// Hidden-hand state enforced via SHA-256 commitment-reveal scheme.
///
/// `commitment = sha256(card_id.to_be_bytes() || nonce_bytes)`
///
/// The commitment hides which card the player chose until the play phase.
/// On reveal the contract re-derives the hash and rejects any mismatch.
#[contracttype]
#[derive(Clone, Debug)]
pub struct HandCommitmentComponent {
    pub commitment: BytesN<32>,
    pub committed: bool,
    pub revealed: bool,
    pub revealed_card: u32,
}

/// Visible board state after each round resolves.
#[contracttype]
#[derive(Clone, Debug)]
pub struct BoardStateComponent {
    pub round: u32,
    pub score_one: u32,
    pub score_two: u32,
    /// Card played by player one in the most recently resolved round.
    pub last_played_one: u32,
    /// Card played by player two in the most recently resolved round.
    pub last_played_two: u32,
    /// Current phase (PHASE_* constants).
    pub round_state: u32,
}

/// DAO-style governance proposal for updating the active format.
///
/// In a full deployment this would integrate with governance.script3.io
/// for on-chain quorum voting. Here proposals auto-accept to demonstrate
/// the data flow and contract surface.
#[contracttype]
#[derive(Clone, Debug)]
pub struct FormatProposalComponent {
    pub proposal_id: u32,
    /// Card ID to add to the ban list (0 = no ban change).
    pub ban_card: u32,
    /// Card ID to remove from the ban list (0 = no unban change).
    pub unban_card: u32,
    pub proposer: Address,
    pub vote_count: u32,
    pub status: u32,
}

/// Top-level match status component.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GameStatusComponent {
    pub status: u32,
    pub phase: u32,
}

// ─── Input types ─────────────────────────────────────────────────────────────

/// Input for the draft phase: a sealed SHA-256 commitment to a chosen card.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ChoiceInput {
    /// `sha256(card_id.to_be_bytes() || nonce_bytes)`
    pub commitment: BytesN<32>,
}

/// Input for the play phase: card reveal + stellar-zk legality proof.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayInput {
    pub card_id: u32,
    pub nonce: BytesN<16>,
    /// Groth16 proof that `card_id` is a member of the active allowed-card set.
    /// Verified on-chain via `stellar-zk` when a verification key is registered.
    pub proof: Groth16Proof,
    pub public_inputs: Vec<Scalar>,
}

/// Input for a format-governance proposal.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalInput {
    /// Card ID to ban (0 = no ban action).
    pub ban_card: u32,
    /// Card ID to unban (0 = no unban action).
    pub unban_card: u32,
}

// ─── Query output ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct GameState {
    pub player_one: Address,
    pub player_two: Address,
    pub board: BoardStateComponent,
    pub status: GameStatusComponent,
    pub active_format: u32,
    pub banned_cards: Vec<u32>,
    /// Number of proposals still awaiting resolution.
    pub pending_proposals: u32,
}

// ─── World state (full ECS snapshot) ─────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug)]
pub struct WorldState {
    pub player_one: Address,
    pub player_two: Address,
    // Format governance
    pub active_format: u32,
    pub banned_cards: Vec<u32>,
    // Match progression
    pub phase: u32,
    pub round: u32,
    pub status: u32,
    // Hidden hand commitments (HandCommitmentComponent per player)
    pub hand_one: HandCommitmentComponent,
    pub hand_two: HandCommitmentComponent,
    // Scores
    pub score_one: u32,
    pub score_two: u32,
    // Last-resolved round cards (BoardStateComponent)
    pub last_played_one: u32,
    pub last_played_two: u32,
    // Governance proposals (FormatProposalComponent list)
    pub proposals: Vec<FormatProposalComponent>,
    pub next_proposal_id: u32,
    // stellar-zk VK registration flag
    pub vk_set: bool,
}

// ─── Storage keys ─────────────────────────────────────────────────────────────

const WORLD_KEY: Symbol = symbol_short!("WORLD");
const VK_KEY: Symbol = symbol_short!("VK");

// ─── Card power lookup ────────────────────────────────────────────────────────

/// Returns the power value for a card ID (1–10).
/// Higher power wins a round during resolution.
pub fn card_power(card_id: u32) -> u32 {
    match card_id {
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        6 => 6,
        7 => 7,
        8 => 8,
        9 => 3,  // Spell - 3-damage equivalent power
        10 => 5, // Spell - 5-damage equivalent power
        _ => 0,
    }
}

/// Compute the SHA-256 commitment for a card reveal.
///
/// `commitment = sha256(card_id.to_be_bytes() || nonce_bytes)`
///
/// Used both by the contract to verify reveals and by clients/tests to
/// construct commitments during the draft phase.
pub fn compute_commitment(env: &Env, card_id: u32, nonce: &BytesN<16>) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.append(&Bytes::from_array(env, &card_id.to_be_bytes()));
    data.append(&Bytes::from_array(env, &nonce.to_array()));
    env.crypto().sha256(&data).into()
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct ShadowDraftCardGame;

#[contractimpl]
impl ShadowDraftCardGame {
    // ── Match setup ───────────────────────────────────────────────────────────

    /// Initialize a new match between `player_one` and `player_two`.
    ///
    /// Both players start in `PHASE_DRAFT` with no commitments.
    pub fn init_match(env: Env, player_one: Address, player_two: Address) {
        if env.storage().instance().has(&WORLD_KEY) {
            panic_with_error!(&env, GameError::AlreadyInitialized);
        }

        let empty_commitment = BytesN::from_array(&env, &[0u8; 32]);
        let hand_one = HandCommitmentComponent {
            commitment: empty_commitment.clone(),
            committed: false,
            revealed: false,
            revealed_card: 0,
        };
        let hand_two = HandCommitmentComponent {
            commitment: empty_commitment,
            committed: false,
            revealed: false,
            revealed_card: 0,
        };

        let world = WorldState {
            player_one,
            player_two,
            active_format: 1,
            banned_cards: Vec::new(&env),
            phase: PHASE_DRAFT,
            round: 1,
            status: STATUS_IN_PROGRESS,
            hand_one,
            hand_two,
            score_one: 0,
            score_two: 0,
            last_played_one: 0,
            last_played_two: 0,
            proposals: Vec::new(&env),
            next_proposal_id: 1,
            vk_set: false,
        };

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── Draft phase ───────────────────────────────────────────────────────────

    /// Submit a sealed card choice during the draft phase.
    ///
    /// `choice.commitment` must equal `sha256(card_id.to_be_bytes() || nonce_bytes)`.
    /// This hides the chosen card until both players commit, after which the
    /// contract (DraftSystem) automatically advances to `PHASE_PLAY`.
    pub fn submit_choice(env: Env, player: Address, choice: ChoiceInput) {
        player.require_auth();

        let mut world: WorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        if world.status != STATUS_IN_PROGRESS {
            panic_with_error!(&env, GameError::GameOver);
        }
        if world.phase != PHASE_DRAFT {
            panic_with_error!(&env, GameError::WrongPhase);
        }

        let is_one = player == world.player_one;
        let is_two = player == world.player_two;
        if !is_one && !is_two {
            panic_with_error!(&env, GameError::NotAPlayer);
        }

        if is_one {
            if world.hand_one.committed {
                panic_with_error!(&env, GameError::AlreadyCommitted);
            }
            world.hand_one.commitment = choice.commitment;
            world.hand_one.committed = true;
        } else {
            if world.hand_two.committed {
                panic_with_error!(&env, GameError::AlreadyCommitted);
            }
            world.hand_two.commitment = choice.commitment;
            world.hand_two.committed = true;
        }

        // DraftSystem: advance to play phase once both players have committed.
        if world.hand_one.committed && world.hand_two.committed {
            world.phase = PHASE_PLAY;
        }

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── Play phase ────────────────────────────────────────────────────────────

    /// Reveal a card and submit its `stellar-zk` legality proof.
    ///
    /// Enforces three checks in sequence (ProofValidationSystem + CardPlaySystem):
    ///
    /// 1. **Commitment check**: `sha256(card_id || nonce) == stored_commitment`.
    ///    Ensures the player cannot switch cards after seeing the opponent commit.
    ///
    /// 2. **Format check**: `card_id` must not appear in the active banned-card list.
    ///    Enforces season rules governed by the DAO (FormatGovernanceSystem).
    ///
    /// 3. **stellar-zk Groth16 proof** (when a VK is registered): proves in
    ///    zero-knowledge that `card_id` is a member of the current allowed set,
    ///    without requiring the verifier to learn which specific card was played
    ///    before both reveals are complete.
    ///
    /// After both players reveal, RoundResolutionSystem runs automatically.
    pub fn play_card(env: Env, player: Address, play: PlayInput) {
        player.require_auth();

        let mut world: WorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        if world.status != STATUS_IN_PROGRESS {
            panic_with_error!(&env, GameError::GameOver);
        }
        if world.phase != PHASE_PLAY {
            panic_with_error!(&env, GameError::WrongPhase);
        }

        let is_one = player == world.player_one;
        let is_two = player == world.player_two;
        if !is_one && !is_two {
            panic_with_error!(&env, GameError::NotAPlayer);
        }

        if play.card_id == 0 || play.card_id > MAX_CARD_ID {
            panic_with_error!(&env, GameError::InvalidCard);
        }

        // ── ProofValidationSystem: commitment check ────────────────────────
        let stored_commitment = if is_one {
            if !world.hand_one.committed {
                panic_with_error!(&env, GameError::NotCommitted);
            }
            if world.hand_one.revealed {
                panic_with_error!(&env, GameError::AlreadyRevealed);
            }
            world.hand_one.commitment.clone()
        } else {
            if !world.hand_two.committed {
                panic_with_error!(&env, GameError::NotCommitted);
            }
            if world.hand_two.revealed {
                panic_with_error!(&env, GameError::AlreadyRevealed);
            }
            world.hand_two.commitment.clone()
        };

        let expected = compute_commitment(&env, play.card_id, &play.nonce);
        if expected != stored_commitment {
            panic_with_error!(&env, GameError::CommitmentMismatch);
        }

        // ── CardPlaySystem: format / ban check ─────────────────────────────
        for i in 0..world.banned_cards.len() {
            if world.banned_cards.get(i).unwrap() == play.card_id {
                panic_with_error!(&env, GameError::CardBanned);
            }
        }

        // ── ProofValidationSystem: stellar-zk Groth16 proof ───────────────
        // When a verification key is registered, validate the Groth16 proof
        // that proves card_id ∈ allowed_set in zero-knowledge.
        // References: https://crates.io/crates/stellar-zk
        //             https://github.com/salazarsebas/stellar-zk
        if world.vk_set {
            let vk: VerificationKey = env
                .storage()
                .instance()
                .get(&VK_KEY)
                .unwrap_or_else(|| panic_with_error!(&env, GameError::InvalidProof));

            let count = (play.public_inputs.len() as usize).min(4);
            let mut rust_inputs: RustVec<Scalar> = RustVec::with_capacity(count);
            for i in 0..count {
                rust_inputs.push(play.public_inputs.get_unchecked(i as u32));
            }

            let valid =
                experimental::verify_groth16(&env, &vk, &play.proof, &rust_inputs).unwrap_or(false);
            if !valid {
                panic_with_error!(&env, GameError::InvalidProof);
            }
        }

        // ── Store reveal ───────────────────────────────────────────────────
        if is_one {
            world.hand_one.revealed = true;
            world.hand_one.revealed_card = play.card_id;
        } else {
            world.hand_two.revealed = true;
            world.hand_two.revealed_card = play.card_id;
        }

        // ── RoundResolutionSystem: resolve when both have revealed ─────────
        if world.hand_one.revealed && world.hand_two.revealed {
            Self::round_resolution_system(&env, &mut world);
        }

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── Governance ────────────────────────────────────────────────────────────

    /// Submit a DAO-style format-governance proposal.
    ///
    /// Any match participant may propose banning or unbanning a card from the
    /// active season format. The FormatGovernanceSystem applies the proposal
    /// immediately (auto-accept), modelling the on-chain execution path that
    /// would follow a successful quorum vote via governance.script3.io.
    pub fn submit_format_proposal(env: Env, proposer: Address, proposal: ProposalInput) {
        proposer.require_auth();

        let mut world: WorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        let proposal_id = world.next_proposal_id;
        world.next_proposal_id += 1;

        let new_proposal = FormatProposalComponent {
            proposal_id,
            ban_card: proposal.ban_card,
            unban_card: proposal.unban_card,
            proposer,
            vote_count: 1,
            status: PROPOSAL_PENDING,
        };

        world.proposals.push_back(new_proposal);

        // FormatGovernanceSystem: apply the proposal immediately.
        Self::format_governance_system(&env, &mut world, proposal_id);

        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Return the current visible game state.
    pub fn get_state(env: Env) -> GameState {
        let world: WorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        let board = BoardStateComponent {
            round: world.round,
            score_one: world.score_one,
            score_two: world.score_two,
            last_played_one: world.last_played_one,
            last_played_two: world.last_played_two,
            round_state: world.phase,
        };

        let mut pending = 0u32;
        for i in 0..world.proposals.len() {
            if world.proposals.get(i).unwrap().status == PROPOSAL_PENDING {
                pending += 1;
            }
        }

        GameState {
            player_one: world.player_one,
            player_two: world.player_two,
            board,
            status: GameStatusComponent {
                status: world.status,
                phase: world.phase,
            },
            active_format: world.active_format,
            banned_cards: world.banned_cards,
            pending_proposals: pending,
        }
    }

    /// Register a `stellar-zk` Groth16 verification key for card-set proofs.
    ///
    /// Once set, every `play_card` call must supply a valid proof.
    pub fn set_vk(env: Env, vk: VerificationKey) {
        let mut world: WorldState = env
            .storage()
            .instance()
            .get(&WORLD_KEY)
            .unwrap_or_else(|| panic_with_error!(&env, GameError::NotInitialized));

        world.vk_set = true;
        env.storage().instance().set(&VK_KEY, &vk);
        env.storage().instance().set(&WORLD_KEY, &world);
    }

    // ── ECS Systems ───────────────────────────────────────────────────────────

    /// RoundResolutionSystem - resolves the current round by comparing card powers.
    ///
    /// Awards a round point to the player with the higher-power card; ties award
    /// nothing. Advances to the next draft phase or ends the match when a player
    /// reaches `ROUNDS_TO_WIN`.
    fn round_resolution_system(env: &Env, world: &mut WorldState) {
        world.phase = PHASE_RESOLUTION;

        let power_one = card_power(world.hand_one.revealed_card);
        let power_two = card_power(world.hand_two.revealed_card);

        if power_one > power_two {
            world.score_one += 1;
        } else if power_two > power_one {
            world.score_two += 1;
        }
        // Ties: no point awarded.

        // Persist last-played cards for board display before resetting hands.
        world.last_played_one = world.hand_one.revealed_card;
        world.last_played_two = world.hand_two.revealed_card;

        // Win-condition check.
        if world.score_one >= ROUNDS_TO_WIN {
            world.status = STATUS_PLAYER_ONE_WINS;
            world.phase = PHASE_FINISHED;
            return;
        }
        if world.score_two >= ROUNDS_TO_WIN {
            world.status = STATUS_PLAYER_TWO_WINS;
            world.phase = PHASE_FINISHED;
            return;
        }

        // Advance to next round - DraftSystem resets hand commitments.
        world.round += 1;
        let empty = BytesN::from_array(env, &[0u8; 32]);
        world.hand_one = HandCommitmentComponent {
            commitment: empty.clone(),
            committed: false,
            revealed: false,
            revealed_card: 0,
        };
        world.hand_two = HandCommitmentComponent {
            commitment: empty,
            committed: false,
            revealed: false,
            revealed_card: 0,
        };
        world.phase = PHASE_DRAFT;
    }

    /// FormatGovernanceSystem - applies an accepted format proposal.
    ///
    /// Marks the proposal `PROPOSAL_ACCEPTED` and updates the `banned_cards`
    /// list accordingly. In a real DAO integration this would only run after a
    /// quorum vote clears on-chain.
    fn format_governance_system(env: &Env, world: &mut WorldState, proposal_id: u32) {
        let mut found_idx = u32::MAX;
        for i in 0..world.proposals.len() {
            if world.proposals.get(i).unwrap().proposal_id == proposal_id {
                found_idx = i;
                break;
            }
        }
        if found_idx == u32::MAX {
            return;
        }

        let mut proposal = world.proposals.get(found_idx).unwrap();
        proposal.status = PROPOSAL_ACCEPTED;

        // Apply ban.
        if proposal.ban_card > 0 {
            let already_banned = (0..world.banned_cards.len())
                .any(|i| world.banned_cards.get(i).unwrap() == proposal.ban_card);
            if !already_banned {
                world.banned_cards.push_back(proposal.ban_card);
            }
        }

        // Apply unban.
        if proposal.unban_card > 0 {
            let mut new_banned = Vec::new(env);
            for i in 0..world.banned_cards.len() {
                let card = world.banned_cards.get(i).unwrap();
                if card != proposal.unban_card {
                    new_banned.push_back(card);
                }
            }
            world.banned_cards = new_banned;
        }

        world.proposals.set(found_idx, proposal);
    }
}

#[cfg(test)]
mod test;

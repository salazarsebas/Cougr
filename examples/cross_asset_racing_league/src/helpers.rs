use crate::types::{DataKey, ProofInput, RacingError};
use soroban_sdk::{panic_with_error, Address, Env};

pub fn assert_initialized(env: &Env) {
    if !env.storage().instance().has(&DataKey::Owner) {
        panic_with_error!(env, RacingError::NotInitialized);
    }
}

pub fn assert_owner(env: &Env, owner: &Address) {
    let stored_owner: Address = env
        .storage()
        .instance()
        .get(&DataKey::Owner)
        .expect("Not initialized");
    if owner != &stored_owner {
        panic_with_error!(env, RacingError::UnauthorizedOwner);
    }
}

pub fn verify_proof_stub(proof: &ProofInput) -> bool {
    !proof.proof.is_empty() && !proof.public_inputs.is_empty()
}

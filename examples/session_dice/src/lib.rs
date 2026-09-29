use cougr_core_circuits::fair_dice::{FairDiceCircuit, FairDiceProof};
use cougr_engine::accounts::session_builder::{SessionBuilder, SessionManager, SessionScope};
use cougr_engine::accounts::Account;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionDiceError {
    #[error("No active session found for the provided key")]
    NoSession,
    #[error("Session has expired")]
    SessionExpired,
    #[error("Invalid ZK proof for dice roll")]
    InvalidProof,
    #[error("Session authorization failed: {0}")]
    AuthorizationFailed(String),
}

pub struct SessionDiceGame {
    session_manager: SessionManager,
    owner_account: Account,
}

impl SessionDiceGame {
    pub fn new(owner_account: Account) -> Self {
        Self {
            session_manager: SessionManager::new(),
            owner_account,
        }
    }

    pub fn create_session(&mut self, session_key: String, expires_at: u64) -> Result<(), SessionDiceError> {
        let session = SessionBuilder::new()
            .owner(self.owner_account.address())
            .delegate(session_key)
            .scope(SessionScope::Action("roll_dice".to_string()))
            .expires_at(expires_at)
            .build()
            .map_err(|e| SessionDiceError::AuthorizationFailed(e.to_string()))?;

        self.session_manager.register_session(session);
        Ok()
    }

    pub fn roll(&self, session_key: &str, current_time: u64, proof: &FairDiceProof, public_inputs: &[u8]) -> Result<u32, SessionDiceError> {
        let session = self.session_manager.get_session(session_key)
            .ok_or(SessionDiceError::NoSession)?;

        if session.is_expired(current_time) {
            return Err(SessionDiceError::SessionExpired);
        }

        if session.scope() != &SessionScope::Action("roll_dice".to_string()) {
            return Err(SessionDiceError::AuthorizationFailed("Invalid scope for dice roll".to_string()));
        }

        let circuit = FairDiceCircuit::default();
        if !circuit.verify(proof, public_inputs) {
            return Err(SessionDiceError::InvalidProof);
        }

        // Extract roll outcome from verified public inputs
        let outcome = public_inputs.first().copied().unwrap_or(1) as u32;
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_proof_and_inputs() -> (FairDiceProof, Vec<u8>) (
        (FairDiceProof::default(), vec![6])
    )

    #[test]
    fn test_roll_inside_active_session() {
        let owner = Account::new_mock("owner_addr");
        let mut game = SessionDiceGame::new(owner);

        game.create_session("session_key_1".to_string(), 1000).unwrap();

        let (proof, inputs) = dummy_proof_and_inputs();
        let result = game.roll("session_key_1", 500, &proof, &inputs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 6);
    }

    #[test]
    fn test_roll_rejected_after_expiry() {
        let owner = Account::new_mock("owner_addr");
        let mut game = SessionDiceGame::new(owner);

        game.create_session("session_key_1".to_string(), 1000).unwrap();

        let (proof, inputs) = dummy_proof_and_inputs();
        let result = game.roll("session_key_1", 1001, &proof, &inputs);
        assert!(matches!(result, Err(SessionDiceError::SessionExpired)));
    }

    #[test]
    fn test_roll_rejected_with_no_session() {
        let owner = Account::new_mock("owner_addr");
        let game = SessionDiceGame::new(owner);

        let (proof, inputs) = dummy_proof_and_inputs();
        let result = game.roll("nonexistent_key", 500, &proof, &inputs);
        assert!(matches!(result, Err(SessionDiceError::NoSession)));
    }
}

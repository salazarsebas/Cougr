//! Contract test harness: env, registration, players, and `as_contract` scope.

use alloc::vec::Vec;
use soroban_sdk::testutils::{Address as _, ContractFunctionSet};
use soroban_sdk::{Address, Env};

/// Index into [`GameHarness::players`] for turn rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerSlot(pub u32);

/// Soroban contract test shell shared by game integration tests.
pub struct GameHarness {
    env: Env,
    contract_id: Address,
    players: Vec<Address>,
}

impl GameHarness {
    /// Register `contract` in a fresh [`Env`] and return a harness around it.
    pub fn new<C>(env: Env, contract: C) -> Self
    where
        C: ContractFunctionSet + Clone + 'static,
    {
        let contract_id = env.register(contract, ());
        Self {
            env,
            contract_id,
            players: Vec::new(),
        }
    }

    /// Wrap an already-registered contract address.
    pub fn from_registered(env: Env, contract_id: Address) -> Self {
        Self {
            env,
            contract_id,
            players: Vec::new(),
        }
    }

    pub fn env(&self) -> &Env {
        &self.env
    }

    pub fn contract_id(&self) -> &Address {
        &self.contract_id
    }

    pub fn players(&self) -> &[Address] {
        &self.players
    }

    /// Generate `count` mock player addresses and store them on the harness.
    pub fn mock_players(&mut self, count: u32) -> &[Address] {
        self.players.clear();
        for _ in 0..count {
            self.players.push(Address::generate(&self.env));
        }
        &self.players
    }

    /// Mock authorization for every address returned by [`mock_players`](Self::mock_players).
    pub fn mock_all_auths(&self) {
        self.env.mock_all_auths();
    }

    /// Resolve a [`PlayerSlot`] to the corresponding player address.
    ///
    /// # Panics
    ///
    /// Panics if no players were registered or `slot` is out of range.
    pub fn player(&self, slot: PlayerSlot) -> &Address {
        self.players
            .get(slot.0 as usize)
            .unwrap_or_else(|| panic!("player slot {} is not registered", slot.0))
    }

    /// `(env, contract_id)` tuple for constructing generated contract clients.
    ///
    /// ```ignore
    /// let client = MyContractClient::new(harness.env(), harness.contract_id());
    /// ```
    pub fn client_args(&self) -> (&Env, &Address) {
        (self.env(), self.contract_id())
    }

    /// Run `f` inside the registered contract's execution context.
    pub fn as_contract<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.env.as_contract(&self.contract_id, f)
    }
}

//! Contract test harness: env, registration, players, and `as_contract` scope.

use alloc::string::String;
use alloc::vec::Vec;
use soroban_sdk::testutils::{Address as _, ContractFunctionSet};
use soroban_sdk::{Address, Env};

/// Index into [`GameHarness::players`] for turn rotation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerSlot(pub u32);

/// The Soroban budget dimensions one top-level invocation consumed.
///
/// The values are the snapshot `env.cost_estimate().resources()` returns - the
/// `InvocationResources` the soroban-sdk testutils host already meters - not a
/// second estimator. Because the sandbox registers a Rust contract instead of a
/// Wasm one, they cover host-side work only: VM instantiation, Wasm reads, and
/// rent bumps on real entries are not part of them, which is why the rent
/// dimensions of `InvocationResources` are left out. Treat the report as a
/// regression signal for one scenario, not as a fee quote.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceReport {
    /// Modelled CPU instructions.
    pub instructions: i64,
    /// Modelled memory in bytes. An upper bound; it does not affect fees.
    pub mem_bytes: i64,
    /// Ledger entries read from disk.
    pub disk_read_entries: u32,
    /// Bytes read from disk.
    pub disk_read_bytes: u32,
    /// Ledger entries written.
    pub write_entries: u32,
    /// Bytes written to the ledger.
    pub write_bytes: u32,
    /// In-memory ledger entries touched. An upper bound; it does not affect fees.
    pub memory_read_entries: u32,
    /// Total size of the contract events the invocation emitted, in bytes.
    pub contract_events_size_bytes: u32,
}

impl ResourceReport {
    /// Column order of [`ResourceReport::to_csv`], and of the committed baseline.
    pub const CSV_HEADER: &'static str = "instructions,mem_bytes,disk_read_entries,disk_read_bytes,write_entries,write_bytes,memory_read_entries,contract_events_size_bytes";

    /// The resources metered for the last top-level contract invocation.
    ///
    /// Metering resets before every top-level invocation, so capture this
    /// immediately after the call being measured.
    ///
    /// # Panics
    ///
    /// Panics when there is no invocation to report, which is the soroban-sdk
    /// behaviour rather than a zeroed report that would compare as healthy.
    pub fn capture(env: &Env) -> Self {
        let resources = env.cost_estimate().resources();
        Self {
            instructions: resources.instructions,
            mem_bytes: resources.mem_bytes,
            disk_read_entries: resources.disk_read_entries,
            disk_read_bytes: resources.disk_read_bytes,
            write_entries: resources.write_entries,
            write_bytes: resources.write_bytes,
            memory_read_entries: resources.memory_read_entries,
            contract_events_size_bytes: resources.contract_events_size_bytes,
        }
    }

    /// One baseline row, without the scenario name, in [`Self::CSV_HEADER`] order.
    pub fn to_csv(&self) -> String {
        alloc::format!(
            "{},{},{},{},{},{},{},{}",
            self.instructions,
            self.mem_bytes,
            self.disk_read_entries,
            self.disk_read_bytes,
            self.write_entries,
            self.write_bytes,
            self.memory_read_entries,
            self.contract_events_size_bytes
        )
    }

    /// Parse one baseline row in [`Self::CSV_HEADER`] order.
    ///
    /// Returns `None` when a field is missing or is not a number, so a malformed
    /// baseline fails the scenario instead of comparing against zeroes.
    pub fn from_csv(row: &str) -> Option<Self> {
        let fields: Vec<&str> = row.split(',').map(|field| field.trim()).collect();
        if fields.len() != 8 || fields.iter().any(|field| field.is_empty()) {
            return None;
        }

        Some(Self {
            instructions: fields[0].parse().ok()?,
            mem_bytes: fields[1].parse().ok()?,
            disk_read_entries: fields[2].parse().ok()?,
            disk_read_bytes: fields[3].parse().ok()?,
            write_entries: fields[4].parse().ok()?,
            write_bytes: fields[5].parse().ok()?,
            memory_read_entries: fields[6].parse().ok()?,
            contract_events_size_bytes: fields[7].parse().ok()?,
        })
    }
}

/// A committed budget for one scenario: the recorded report plus its slack.
///
/// The rule is split by dimension. The metered CPU, memory, and byte numbers
/// move with the compiler and the host version, so they get a percentage. Ledger
/// entry counts are exact integers, so they are gated exactly and only change
/// when the baseline does - that is what catches a structural regression such as
/// an extra write or an O(n) loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceBudget {
    /// What the scenario measured when the baseline was recorded.
    pub baseline: ResourceReport,
    /// Allowed growth on instructions, memory, and byte dimensions, in percent.
    pub tolerance_percent: u32,
    /// Allowed extra ledger entries, on top of the baseline, for the entry counts.
    pub extra_entries: u32,
}

impl ResourceBudget {
    pub const fn new(baseline: ResourceReport, tolerance_percent: u32, extra_entries: u32) -> Self {
        Self {
            baseline,
            tolerance_percent,
            extra_entries,
        }
    }

    /// `Ok(())` when `report` fits the budget, otherwise the first overrun.
    pub fn check(&self, scenario: &str, report: &ResourceReport) -> Result<(), String> {
        let tolerance = self.tolerance_percent as i64;

        Self::check_units(
            scenario,
            "instructions",
            report.instructions,
            self.baseline.instructions,
            self.baseline.instructions.saturating_mul(tolerance) / 100,
        )?;
        Self::check_units(
            scenario,
            "mem_bytes",
            report.mem_bytes,
            self.baseline.mem_bytes,
            self.baseline.mem_bytes.saturating_mul(tolerance) / 100,
        )?;

        let entries = self.extra_entries as u64;
        for (label, actual, baseline, is_entry) in [
            (
                "disk_read_entries",
                report.disk_read_entries as u64,
                self.baseline.disk_read_entries as u64,
                true,
            ),
            (
                "disk_read_bytes",
                report.disk_read_bytes as u64,
                self.baseline.disk_read_bytes as u64,
                false,
            ),
            (
                "write_entries",
                report.write_entries as u64,
                self.baseline.write_entries as u64,
                true,
            ),
            (
                "write_bytes",
                report.write_bytes as u64,
                self.baseline.write_bytes as u64,
                false,
            ),
            (
                "memory_read_entries",
                report.memory_read_entries as u64,
                self.baseline.memory_read_entries as u64,
                true,
            ),
            (
                "contract_events_size_bytes",
                report.contract_events_size_bytes as u64,
                self.baseline.contract_events_size_bytes as u64,
                false,
            ),
        ] {
            let allowed = baseline
                + baseline * self.tolerance_percent as u64 / 100
                + if is_entry { entries } else { 0 };
            if actual > allowed {
                return Err(if is_entry {
                    alloc::format!(
                        "resource regression in {scenario}: {label} is {actual}, baseline {baseline} + {}% + {entries} entries is {allowed}",
                        self.tolerance_percent
                    )
                } else {
                    alloc::format!(
                        "resource regression in {scenario}: {label} is {actual}, baseline {baseline} + {}% is {allowed}",
                        self.tolerance_percent
                    )
                });
            }
        }

        Ok(())
    }

    fn check_units(
        scenario: &str,
        label: &str,
        actual: i64,
        baseline: i64,
        allowance: i64,
    ) -> Result<(), String> {
        let allowed = baseline.saturating_add(allowance);
        if actual > allowed {
            return Err(alloc::format!(
                "resource regression in {scenario}: {label} is {actual}, baseline {baseline} + {allowance} is {allowed}"
            ));
        }
        Ok(())
    }
}

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

    /// The resources the last top-level invocation through this harness consumed.
    ///
    /// Call this immediately after the measured call: metering resets before the
    /// next top-level invocation.
    pub fn resource_report(&self) -> ResourceReport {
        ResourceReport::capture(&self.env)
    }

    /// Capture the report for the last invocation and assert it is inside `budget`.
    ///
    /// # Panics
    ///
    /// Panics with the offending dimension when the scenario regressed, which is
    /// how the committed baseline gates CI.
    pub fn assert_resource_budget(
        &self,
        scenario: &str,
        budget: &ResourceBudget,
    ) -> ResourceReport {
        let report = self.resource_report();
        if let Err(message) = budget.check(scenario, &report) {
            panic!("{message}");
        }
        report
    }
}

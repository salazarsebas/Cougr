use alloc::string::String;
use alloc::vec::Vec;

/// Ordered execution phases for Soroban gameplay loops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleStage {
    Startup,
    PreUpdate,
    Update,
    PostUpdate,
    Cleanup,
}

impl ScheduleStage {
    pub(crate) fn ordered() -> [Self; 5] {
        [
            Self::Startup,
            Self::PreUpdate,
            Self::Update,
            Self::PostUpdate,
            Self::Cleanup,
        ]
    }
}

/// Declarative configuration for a scheduled system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemConfig {
    stage: ScheduleStage,
    set: Option<String>,
    after: Vec<String>,
    before: Vec<String>,
    after_sets: Vec<String>,
    before_sets: Vec<String>,
}

impl SystemConfig {
    pub fn new() -> Self {
        Self {
            stage: ScheduleStage::Update,
            set: None,
            after: Vec::new(),
            before: Vec::new(),
            after_sets: Vec::new(),
            before_sets: Vec::new(),
        }
    }

    pub fn in_stage(mut self, stage: ScheduleStage) -> Self {
        self.stage = stage;
        self
    }

    pub fn after(mut self, system_name: impl Into<String>) -> Self {
        self.after.push(system_name.into());
        self
    }

    pub fn before(mut self, system_name: impl Into<String>) -> Self {
        self.before.push(system_name.into());
        self
    }

    pub fn in_set(mut self, set_name: impl Into<String>) -> Self {
        self.set = Some(set_name.into());
        self
    }

    pub fn after_set(mut self, set_name: impl Into<String>) -> Self {
        self.after_sets.push(set_name.into());
        self
    }

    pub fn before_set(mut self, set_name: impl Into<String>) -> Self {
        self.before_sets.push(set_name.into());
        self
    }

    pub fn stage(&self) -> ScheduleStage {
        self.stage
    }

    pub fn set_name(&self) -> Option<&str> {
        self.set.as_deref()
    }

    pub fn after_dependencies(&self) -> &[String] {
        &self.after
    }

    pub fn before_dependencies(&self) -> &[String] {
        &self.before
    }

    pub fn after_set_dependencies(&self) -> &[String] {
        &self.after_sets
    }

    pub fn before_set_dependencies(&self) -> &[String] {
        &self.before_sets
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation or planning failure for `SimpleScheduler`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    DuplicateSystem(String),
    MissingDependency {
        system: String,
        dependency: String,
    },
    MissingSet {
        system: String,
        set: String,
    },
    CrossStageDependency {
        system: String,
        dependency: String,
        system_stage: ScheduleStage,
        dependency_stage: ScheduleStage,
    },
    DependencyCycle {
        stage: ScheduleStage,
        systems: Vec<String>,
    },
}

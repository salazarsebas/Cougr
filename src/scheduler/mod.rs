mod types;

pub use types::{ScheduleError, ScheduleStage, SystemConfig};

use crate::simple_world::SimpleWorld;
use crate::system::{AppSystem, SimpleSystem, SystemContext, SystemSpec, WorldSystem};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use soroban_sdk::Env;

struct SimpleSystemEntry {
    name: String,
    config: SystemConfig,
    system: Box<dyn SimpleSystem>,
}

/// Scheduler for the Soroban-first `SimpleWorld` runtime.
///
/// Systems can be grouped into stages and ordered relative to each other using
/// `SystemConfig::before()` and `SystemConfig::after()`. Each system receives
/// a deferred `CommandQueue` through `SystemContext`; queued commands are
/// applied after the system finishes.
///
/// # Example
/// ```
/// use cougr_core::scheduler::{ScheduleStage, SimpleScheduler, SystemConfig};
/// use cougr_core::simple_world::SimpleWorld;
/// use soroban_sdk::{symbol_short, Bytes, Env};
///
/// fn physics_system(world: &mut SimpleWorld, env: &Env) {
///     let entity = world.spawn_entity();
///     world.add_component(entity, symbol_short!("physics"), Bytes::new(env));
/// }
/// fn scoring_system(world: &mut SimpleWorld, env: &Env) {
///     let entity = world.spawn_entity();
///     world.add_component(entity, symbol_short!("scoring"), Bytes::new(env));
/// }
///
/// let env = Env::default();
/// let mut world = SimpleWorld::new(&env);
/// let mut scheduler = SimpleScheduler::new();
/// scheduler.add_system("physics", physics_system);
/// scheduler.add_system_with_config(
///     "scoring",
///     scoring_system,
///     SystemConfig::new()
///         .in_stage(ScheduleStage::Update)
///         .after("physics"),
/// );
/// scheduler.run_all(&mut world, &env).unwrap();
/// assert_eq!(scheduler.system_count(), 2);
/// ```
pub struct SimpleScheduler {
    systems: Vec<SimpleSystemEntry>,
}

/// Group of one or more runtime systems that can be registered together.
pub trait SystemGroup {
    fn register(self, scheduler: &mut SimpleScheduler);
    fn register_in_stage(self, scheduler: &mut SimpleScheduler, stage: ScheduleStage);
}

impl<S> SystemGroup for SystemSpec<S>
where
    S: AppSystem + 'static,
{
    fn register(self, scheduler: &mut SimpleScheduler) {
        scheduler.add_system_spec(self);
    }

    fn register_in_stage(self, scheduler: &mut SimpleScheduler, stage: ScheduleStage) {
        scheduler.add_system_spec(self.in_stage(stage));
    }
}

impl<A: SystemGroup, B: SystemGroup> SystemGroup for (A, B) {
    fn register(self, scheduler: &mut SimpleScheduler) {
        self.0.register(scheduler);
        self.1.register(scheduler);
    }

    fn register_in_stage(self, scheduler: &mut SimpleScheduler, stage: ScheduleStage) {
        self.0.register_in_stage(scheduler, stage);
        self.1.register_in_stage(scheduler, stage);
    }
}

impl<A: SystemGroup, B: SystemGroup, C: SystemGroup> SystemGroup for (A, B, C) {
    fn register(self, scheduler: &mut SimpleScheduler) {
        self.0.register(scheduler);
        self.1.register(scheduler);
        self.2.register(scheduler);
    }

    fn register_in_stage(self, scheduler: &mut SimpleScheduler, stage: ScheduleStage) {
        self.0.register_in_stage(scheduler, stage);
        self.1.register_in_stage(scheduler, stage);
        self.2.register_in_stage(scheduler, stage);
    }
}

impl SimpleScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    /// Add a world/env system to the `Update` stage.
    pub fn add_system<F>(&mut self, name: &'static str, system: F)
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.add_system_with_config(name, system, SystemConfig::default());
    }

    /// Add a world/env system with explicit scheduling rules.
    pub fn add_system_with_config<F>(&mut self, name: &'static str, system: F, config: SystemConfig)
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.push_entry(name, config, Box::new(WorldSystem::new(system)));
    }

    /// Add a world/env system directly to a specific stage.
    pub fn add_system_in_stage<F>(&mut self, stage: ScheduleStage, name: &'static str, system: F)
    where
        F: FnMut(&mut SimpleWorld, &Env) + 'static,
    {
        self.add_system_with_config(name, system, SystemConfig::new().in_stage(stage));
    }

    /// Add a context-aware system to the `Update` stage.
    pub fn add_context_system<F>(&mut self, name: &'static str, system: F)
    where
        F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>) + 'static,
    {
        self.add_context_system_with_config(name, system, SystemConfig::default());
    }

    /// Add a context-aware system with explicit scheduling rules.
    pub fn add_context_system_with_config<F>(
        &mut self,
        name: &'static str,
        system: F,
        config: SystemConfig,
    ) where
        F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>) + 'static,
    {
        self.push_entry(
            name,
            config,
            Box::new(crate::system::ContextSystem::new(system)),
        );
    }

    /// Add a context-aware system directly to a specific stage.
    pub fn add_context_system_in_stage<F>(
        &mut self,
        stage: ScheduleStage,
        name: &'static str,
        system: F,
    ) where
        F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>) + 'static,
    {
        self.add_context_system_with_config(name, system, SystemConfig::new().in_stage(stage));
    }

    /// Add any pre-built runtime system to the default `Update` stage.
    pub fn add_simple_system<S>(&mut self, name: &'static str, system: S)
    where
        S: AppSystem + 'static,
    {
        self.add_simple_system_with_config(name, system, SystemConfig::default());
    }

    /// Add any pre-built runtime system with explicit scheduling rules.
    pub fn add_simple_system_with_config<S>(
        &mut self,
        name: &'static str,
        system: S,
        config: SystemConfig,
    ) where
        S: AppSystem + 'static,
    {
        self.push_entry(name, config, Box::new(system));
    }

    /// Add any pre-built runtime system directly to a specific stage.
    pub fn add_simple_system_in_stage<S>(
        &mut self,
        stage: ScheduleStage,
        name: &'static str,
        system: S,
    ) where
        S: AppSystem + 'static,
    {
        self.add_simple_system_with_config(name, system, SystemConfig::new().in_stage(stage));
    }

    /// Add a declarative runtime system spec.
    pub fn add_system_spec<S>(&mut self, spec: SystemSpec<S>)
    where
        S: AppSystem + 'static,
    {
        let (name, config, system) = spec.into_parts();
        self.push_entry(name, config, Box::new(system));
    }

    /// Add one or more runtime systems using the modern declarative API.
    pub fn add_systems<G>(&mut self, group: G)
    where
        G: SystemGroup,
    {
        group.register(self);
    }

    /// Add one or more runtime systems while forcing them into a stage.
    pub fn add_systems_in_stage<G>(&mut self, stage: ScheduleStage, group: G)
    where
        G: SystemGroup,
    {
        group.register_in_stage(self, stage);
    }

    /// Update the scheduling config for an already-registered system.
    pub fn configure_system(
        &mut self,
        name: &str,
        config: SystemConfig,
    ) -> Result<(), ScheduleError> {
        for entry in &mut self.systems {
            if entry.name == name {
                entry.config = config;
                return Ok(());
            }
        }

        Err(ScheduleError::MissingDependency {
            system: name.to_string(),
            dependency: name.to_string(),
        })
    }

    fn push_entry(
        &mut self,
        name: &'static str,
        config: SystemConfig,
        system: Box<dyn SimpleSystem>,
    ) {
        self.systems.push(SimpleSystemEntry {
            name: name.to_string(),
            config,
            system,
        });
    }

    /// Validate and execute the full schedule.
    pub fn run_all(&mut self, world: &mut SimpleWorld, env: &Env) -> Result<(), ScheduleError> {
        for stage in ScheduleStage::ordered() {
            self.run_stage(stage, world, env)?;
        }
        Ok(())
    }

    /// Validate and execute only a single stage.
    pub fn run_stage(
        &mut self,
        stage: ScheduleStage,
        world: &mut SimpleWorld,
        env: &Env,
    ) -> Result<(), ScheduleError> {
        let plan = self.execution_plan_for_stage(stage)?;
        for index in plan {
            let mut commands = crate::commands::CommandQueue::new();
            let entry = &mut self.systems[index];
            let mut context = SystemContext::new(world, env, &mut commands);
            entry.system.run(&mut context);
            commands.apply(world);
        }
        Ok(())
    }

    fn execution_plan_for_stage(&self, stage: ScheduleStage) -> Result<Vec<usize>, ScheduleError> {
        self.validate_unique_names()?;
        let mut stage_indexes = Vec::new();
        for index in 0..self.systems.len() {
            if self.systems[index].config.stage() == stage {
                stage_indexes.push(index);
            }
        }

        self.validate_stage_dependencies(stage, &stage_indexes)?;
        self.topological_order(stage, &stage_indexes)
    }

    fn validate_unique_names(&self) -> Result<(), ScheduleError> {
        for left in 0..self.systems.len() {
            for right in (left + 1)..self.systems.len() {
                if self.systems[left].name == self.systems[right].name {
                    return Err(ScheduleError::DuplicateSystem(
                        self.systems[left].name.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_stage_dependencies(
        &self,
        stage: ScheduleStage,
        stage_indexes: &[usize],
    ) -> Result<(), ScheduleError> {
        for &index in stage_indexes {
            let entry = &self.systems[index];

            for dependency in entry.config.after_dependencies() {
                let dependency_index = self.find_system_index(dependency).ok_or_else(|| {
                    ScheduleError::MissingDependency {
                        system: entry.name.clone(),
                        dependency: dependency.clone(),
                    }
                })?;
                let dependency_stage = self.systems[dependency_index].config.stage();
                if dependency_stage != stage {
                    return Err(ScheduleError::CrossStageDependency {
                        system: entry.name.clone(),
                        dependency: dependency.clone(),
                        system_stage: stage,
                        dependency_stage,
                    });
                }
            }

            for dependency in entry.config.before_dependencies() {
                let dependency_index = self.find_system_index(dependency).ok_or_else(|| {
                    ScheduleError::MissingDependency {
                        system: entry.name.clone(),
                        dependency: dependency.clone(),
                    }
                })?;
                let dependency_stage = self.systems[dependency_index].config.stage();
                if dependency_stage != stage {
                    return Err(ScheduleError::CrossStageDependency {
                        system: entry.name.clone(),
                        dependency: dependency.clone(),
                        system_stage: stage,
                        dependency_stage,
                    });
                }
            }

            for set in entry.config.after_set_dependencies() {
                if !self.stage_has_set(stage_indexes, set) {
                    return Err(ScheduleError::MissingSet {
                        system: entry.name.clone(),
                        set: set.clone(),
                    });
                }
            }

            for set in entry.config.before_set_dependencies() {
                if !self.stage_has_set(stage_indexes, set) {
                    return Err(ScheduleError::MissingSet {
                        system: entry.name.clone(),
                        set: set.clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn topological_order(
        &self,
        stage: ScheduleStage,
        stage_indexes: &[usize],
    ) -> Result<Vec<usize>, ScheduleError> {
        let mut remaining = Vec::new();
        let mut indegree = Vec::new();
        let mut outgoing: Vec<Vec<usize>> = Vec::new();

        for &system_index in stage_indexes {
            remaining.push(system_index);
            indegree.push(0usize);
            outgoing.push(Vec::new());
        }

        for i in 0..stage_indexes.len() {
            let source_index = stage_indexes[i];
            let source = &self.systems[source_index];

            for dependency in source.config.before_dependencies() {
                let target_local = self.find_stage_local_index(stage_indexes, dependency);
                if let Some(target_local) = target_local {
                    outgoing[i].push(target_local);
                    indegree[target_local] += 1;
                }
            }

            for dependency in source.config.after_dependencies() {
                let dependency_local = self.find_stage_local_index(stage_indexes, dependency);
                if let Some(dependency_local) = dependency_local {
                    outgoing[dependency_local].push(i);
                    indegree[i] += 1;
                }
            }

            for set in source.config.before_set_dependencies() {
                for target_local in self.find_stage_set_members(stage_indexes, set) {
                    if target_local != i {
                        outgoing[i].push(target_local);
                        indegree[target_local] += 1;
                    }
                }
            }

            for set in source.config.after_set_dependencies() {
                for dependency_local in self.find_stage_set_members(stage_indexes, set) {
                    if dependency_local != i {
                        outgoing[dependency_local].push(i);
                        indegree[i] += 1;
                    }
                }
            }
        }

        let mut queue = Vec::new();
        for (i, degree) in indegree.iter().enumerate() {
            if *degree == 0 {
                queue.push(i);
            }
        }

        let mut ordered = Vec::new();
        while !queue.is_empty() {
            let local_index = queue.remove(0);
            ordered.push(stage_indexes[local_index]);

            for &target_local in outgoing[local_index].iter() {
                indegree[target_local] -= 1;
                if indegree[target_local] == 0 {
                    queue.push(target_local);
                }
            }
        }

        if ordered.len() != stage_indexes.len() {
            let mut names = Vec::new();
            for &index in stage_indexes {
                names.push(self.systems[index].name.clone());
            }
            return Err(ScheduleError::DependencyCycle {
                stage,
                systems: names,
            });
        }

        Ok(ordered)
    }

    fn find_system_index(&self, name: &str) -> Option<usize> {
        (0..self.systems.len()).find(|&index| self.systems[index].name == name)
    }

    fn find_stage_local_index(&self, stage_indexes: &[usize], name: &str) -> Option<usize> {
        for (local_index, system_index) in stage_indexes.iter().enumerate() {
            if self.systems[*system_index].name == name {
                return Some(local_index);
            }
        }
        None
    }

    fn stage_has_set(&self, stage_indexes: &[usize], set: &str) -> bool {
        for &system_index in stage_indexes {
            if self.systems[system_index].config.set_name() == Some(set) {
                return true;
            }
        }
        false
    }

    fn find_stage_set_members(&self, stage_indexes: &[usize], set: &str) -> Vec<usize> {
        let mut members = Vec::new();
        for (local_index, system_index) in stage_indexes.iter().enumerate() {
            if self.systems[*system_index].config.set_name() == Some(set) {
                members.push(local_index);
            }
        }
        members
    }

    /// Returns the number of registered systems.
    pub fn system_count(&self) -> usize {
        self.systems.len()
    }

    /// Returns the system names in registration order.
    pub fn system_names(&self) -> Vec<&str> {
        self.systems
            .iter()
            .map(|entry| entry.name.as_str())
            .collect()
    }

    /// Returns the system names assigned to a given stage in execution order.
    pub fn stage_system_names(&self, stage: ScheduleStage) -> Result<Vec<&str>, ScheduleError> {
        let mut names = Vec::new();
        for index in self.execution_plan_for_stage(stage)? {
            names.push(self.systems[index].name.as_str());
        }
        Ok(names)
    }
}

impl Default for SimpleScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::{named_context_system, named_system};
    use soroban_sdk::{symbol_short, Bytes, Env};

    #[test]
    fn test_simple_scheduler_empty() {
        let mut scheduler = SimpleScheduler::new();
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        scheduler.run_all(&mut world, &env).unwrap();
        assert_eq!(scheduler.system_count(), 0);
    }

    fn test_system_a(world: &mut SimpleWorld, env: &Env) {
        let e1 = world.spawn_entity();
        let data = Bytes::from_array(env, &[0xAA]);
        world.add_component(e1, symbol_short!("sys_a"), data);
    }

    fn test_system_b(world: &mut SimpleWorld, env: &Env) {
        let e2 = world.spawn_entity();
        let data = Bytes::from_array(env, &[0xBB]);
        world.add_component(e2, symbol_short!("sys_b"), data);
    }

    #[test]
    fn test_simple_scheduler_execution_order() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system("system_a", test_system_a);
        scheduler.add_system("system_b", test_system_b);
        assert_eq!(scheduler.system_count(), 2);

        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        scheduler.run_all(&mut world, &env).unwrap();

        assert!(world.has_component(1, &symbol_short!("sys_a")));
        assert!(world.has_component(2, &symbol_short!("sys_b")));
    }

    #[test]
    fn test_simple_scheduler_names() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system("physics", test_system_a);
        scheduler.add_system("scoring", test_system_b);

        let names = scheduler.system_names();
        assert_eq!(names, alloc::vec!["physics", "scoring"]);
    }

    #[test]
    fn test_stage_ordering_and_dependencies() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system_with_config(
            "cleanup",
            test_system_b,
            SystemConfig::new().in_stage(ScheduleStage::Cleanup),
        );
        scheduler.add_system_with_config(
            "physics",
            test_system_a,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .before("scoring"),
        );
        scheduler.add_system_with_config(
            "scoring",
            test_system_b,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .after("physics"),
        );

        assert_eq!(
            scheduler.stage_system_names(ScheduleStage::Update).unwrap(),
            alloc::vec!["physics", "scoring"]
        );
        assert_eq!(
            scheduler
                .stage_system_names(ScheduleStage::Cleanup)
                .unwrap(),
            alloc::vec!["cleanup"]
        );
    }

    #[test]
    fn test_context_system_applies_deferred_commands() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_context_system("spawn_marker", |context| {
            context.commands().spawn();
        });

        scheduler.run_all(&mut world, &env).unwrap();
        assert_eq!(world.next_entity_id, 2);
    }

    #[test]
    fn test_detects_cross_stage_dependency() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system_with_config(
            "startup",
            test_system_a,
            SystemConfig::new().in_stage(ScheduleStage::Startup),
        );
        scheduler.add_system_with_config(
            "update",
            test_system_b,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .after("startup"),
        );

        let err = scheduler
            .stage_system_names(ScheduleStage::Update)
            .unwrap_err();
        assert!(matches!(err, ScheduleError::CrossStageDependency { .. }));
    }

    #[test]
    fn test_set_ordering() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system_with_config(
            "apply_input",
            test_system_a,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .in_set("input"),
        );
        scheduler.add_system_with_config(
            "physics",
            test_system_b,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .in_set("simulation")
                .after_set("input"),
        );

        assert_eq!(
            scheduler.stage_system_names(ScheduleStage::Update).unwrap(),
            alloc::vec!["apply_input", "physics"]
        );
    }

    #[test]
    fn test_missing_set_dependency() {
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_system_with_config(
            "physics",
            test_system_a,
            SystemConfig::new()
                .in_stage(ScheduleStage::Update)
                .after_set("missing"),
        );

        let err = scheduler
            .stage_system_names(ScheduleStage::Update)
            .unwrap_err();
        assert!(matches!(err, ScheduleError::MissingSet { .. }));
    }

    #[test]
    fn test_grouped_system_registration() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let mut scheduler = SimpleScheduler::new();
        scheduler.add_systems((
            named_system("spawn_a", test_system_a).in_stage(ScheduleStage::Startup),
            named_system("spawn_b", test_system_b).in_stage(ScheduleStage::Update),
        ));

        assert_eq!(
            scheduler
                .stage_system_names(ScheduleStage::Startup)
                .unwrap(),
            alloc::vec!["spawn_a"]
        );
        assert_eq!(
            scheduler.stage_system_names(ScheduleStage::Update).unwrap(),
            alloc::vec!["spawn_b"]
        );

        scheduler.run_all(&mut world, &env).unwrap();
        assert!(world.has_component(1, &symbol_short!("sys_a")));
        assert!(world.has_component(2, &symbol_short!("sys_b")));
    }

    #[test]
    fn test_simple_scheduler_execution_order_with_context_and_world_systems() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let mut scheduler = SimpleScheduler::new();

        scheduler.add_systems((
            named_system("spawn_entity", |world: &mut SimpleWorld, env: &Env| {
                let entity = world.spawn_entity();
                world.add_component(entity, symbol_short!("seed"), Bytes::from_array(env, &[1]));
            })
            .in_stage(ScheduleStage::Update),
            named_context_system("mark_seed", |context| {
                let entities = context
                    .world()
                    .get_entities_with_component(&symbol_short!("seed"), context.env());
                let env = context.env().clone();
                for i in 0..entities.len() {
                    let entity = entities.get(i).unwrap();
                    context.commands().add_component(
                        entity,
                        symbol_short!("grown"),
                        Bytes::from_array(&env, &[1]),
                    );
                }
            })
            .in_stage(ScheduleStage::Update)
            .after("spawn_entity"),
        ));

        scheduler.run_all(&mut world, &env).unwrap();
        assert!(world.has_component(1, &symbol_short!("grown")));
    }
}

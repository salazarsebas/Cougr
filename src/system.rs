use crate::commands::CommandQueue;
use crate::scheduler::{ScheduleStage, SystemConfig};
use crate::simple_world::SimpleWorld;

/// Execution context for the Soroban-first `SimpleWorld` system API.
pub struct SystemContext<'w, 'e, 'c> {
    world: &'w mut SimpleWorld,
    env: &'e soroban_sdk::Env,
    commands: &'c mut CommandQueue,
}

impl<'w, 'e, 'c> SystemContext<'w, 'e, 'c> {
    pub fn new(
        world: &'w mut SimpleWorld,
        env: &'e soroban_sdk::Env,
        commands: &'c mut CommandQueue,
    ) -> Self {
        Self {
            world,
            env,
            commands,
        }
    }

    pub fn world(&self) -> &SimpleWorld {
        self.world
    }

    pub fn world_mut(&mut self) -> &mut SimpleWorld {
        self.world
    }

    pub fn env(&self) -> &soroban_sdk::Env {
        self.env
    }

    pub fn commands(&mut self) -> &mut CommandQueue {
        self.commands
    }
}

/// System trait for the `SimpleWorld`/Soroban runtime.
pub trait SimpleSystem {
    fn run(&mut self, context: &mut SystemContext<'_, '_, '_>);
}

/// Marker trait for systems that participate in the Soroban-first app runtime.
///
/// This is the preferred system model for `GameApp` and `SimpleScheduler`.
pub trait AppSystem: SimpleSystem {}

impl<T: SimpleSystem + ?Sized> AppSystem for T {}

/// Declarative registration spec for the Soroban-first runtime system APIs.
///
/// This is the preferred way to package a system together with its scheduler
/// metadata before handing it to `GameApp::add_systems()` or
/// `SimpleScheduler::add_systems()`.
pub struct SystemSpec<S> {
    name: &'static str,
    system: S,
    config: SystemConfig,
}

impl<S> SystemSpec<S> {
    pub fn new(name: &'static str, system: S) -> Self {
        Self {
            name,
            system,
            config: SystemConfig::default(),
        }
    }

    pub fn in_stage(mut self, stage: ScheduleStage) -> Self {
        self.config = self.config.in_stage(stage);
        self
    }

    pub fn after(mut self, system_name: impl Into<alloc::string::String>) -> Self {
        self.config = self.config.after(system_name);
        self
    }

    pub fn before(mut self, system_name: impl Into<alloc::string::String>) -> Self {
        self.config = self.config.before(system_name);
        self
    }

    pub fn in_set(mut self, set_name: impl Into<alloc::string::String>) -> Self {
        self.config = self.config.in_set(set_name);
        self
    }

    pub fn after_set(mut self, set_name: impl Into<alloc::string::String>) -> Self {
        self.config = self.config.after_set(set_name);
        self
    }

    pub fn before_set(mut self, set_name: impl Into<alloc::string::String>) -> Self {
        self.config = self.config.before_set(set_name);
        self
    }

    pub fn with_config(mut self, config: SystemConfig) -> Self {
        self.config = config;
        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn config(&self) -> &SystemConfig {
        &self.config
    }

    pub(crate) fn into_parts(self) -> (&'static str, SystemConfig, S) {
        (self.name, self.config, self.system)
    }
}

/// Adapter for context-aware closures.
pub struct ContextSystem<F> {
    function: F,
}

impl<F> ContextSystem<F> {
    pub fn new(function: F) -> Self {
        Self { function }
    }
}

impl<F> SimpleSystem for ContextSystem<F>
where
    F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>),
{
    fn run(&mut self, context: &mut SystemContext<'_, '_, '_>) {
        (self.function)(context);
    }
}

/// Adapter for world/env systems to preserve the original onboarding path.
pub struct WorldSystem<F> {
    function: F,
}

impl<F> WorldSystem<F> {
    pub fn new(function: F) -> Self {
        Self { function }
    }
}

impl<F> SimpleSystem for WorldSystem<F>
where
    F: FnMut(&mut SimpleWorld, &soroban_sdk::Env),
{
    fn run(&mut self, context: &mut SystemContext<'_, '_, '_>) {
        let env = context.env().clone();
        (self.function)(context.world_mut(), &env);
    }
}

/// Wrap a `FnMut(&mut SystemContext)` closure as a runtime system.
pub fn context_system<F>(function: F) -> ContextSystem<F>
where
    F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>),
{
    ContextSystem::new(function)
}

/// Wrap a `FnMut(&mut SimpleWorld, &Env)` closure as a runtime system.
pub fn world_system<F>(function: F) -> WorldSystem<F>
where
    F: FnMut(&mut SimpleWorld, &soroban_sdk::Env),
{
    WorldSystem::new(function)
}

/// Wrap a world/env closure together with its registration name.
pub fn named_system<F>(name: &'static str, function: F) -> SystemSpec<WorldSystem<F>>
where
    F: FnMut(&mut SimpleWorld, &soroban_sdk::Env),
{
    SystemSpec::new(name, world_system(function))
}

/// Wrap a context-aware closure together with its registration name.
pub fn named_context_system<F>(name: &'static str, function: F) -> SystemSpec<ContextSystem<F>>
where
    F: for<'w, 'e, 'c> FnMut(&mut SystemContext<'w, 'e, 'c>),
{
    SystemSpec::new(name, context_system(function))
}

/// Wrap a pre-built runtime system together with its registration name.
pub fn named_app_system<S>(name: &'static str, system: S) -> SystemSpec<S>
where
    S: AppSystem,
{
    SystemSpec::new(name, system)
}

#[cfg(test)]
mod tests {
    use super::{
        named_context_system, named_system, AppSystem, ContextSystem, SimpleSystem, SystemContext,
        WorldSystem,
    };
    use crate::commands::CommandQueue;
    use crate::simple_world::SimpleWorld;
    use soroban_sdk::{symbol_short, Bytes, Env};

    #[test]
    fn world_system_wraps_world_and_env_closure() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let mut commands = CommandQueue::new();
        let mut system = WorldSystem::new(|world: &mut SimpleWorld, env: &Env| {
            let entity = world.spawn_entity();
            world.add_component(entity, symbol_short!("tag"), Bytes::from_array(env, &[1]));
        });
        let mut context = SystemContext::new(&mut world, &env, &mut commands);

        system.run(&mut context);

        assert!(world.has_component(1, &symbol_short!("tag")));
    }

    #[test]
    fn context_system_wraps_context_closure() {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let mut commands = CommandQueue::new();
        let mut system = ContextSystem::new(|context: &mut SystemContext<'_, '_, '_>| {
            context.commands().spawn();
        });
        let mut context = SystemContext::new(&mut world, &env, &mut commands);

        system.run(&mut context);
        let spawned = commands.apply(&mut world);

        assert_eq!(spawned.len(), 1);
    }

    #[test]
    fn named_helpers_preserve_registration_name() {
        let world_spec = named_system("tick", |_world: &mut SimpleWorld, _env: &Env| {});
        let context_spec = named_context_system("ctx", |_context| {});

        assert_eq!(world_spec.name(), "tick");
        assert_eq!(context_spec.name(), "ctx");
    }

    fn accepts_app_system<S: AppSystem>(_system: &S) {}

    #[test]
    fn runtime_adapters_implement_app_system() {
        let world_system = WorldSystem::new(|_world: &mut SimpleWorld, _env: &Env| {});
        let context_system = ContextSystem::new(|_context: &mut SystemContext<'_, '_, '_>| {});

        accepts_app_system(&world_system);
        accepts_app_system(&context_system);
    }
}

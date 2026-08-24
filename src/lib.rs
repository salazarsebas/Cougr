#![no_std]
#![doc = r#"
Cougr is a monolithic-on-the-outside ECS framework for Soroban-compatible applications.

The public API is intentionally split into:

- root re-exports for the onboarding path
- `app` for the default gameplay runtime surface
- `auth` for beta account and session flows
- `privacy` for stable and experimental privacy surfaces
- `ops` for stable operational standards
- `accounts` for account abstraction and session flows
- `zk::stable` for stable privacy primitives
- `zk::experimental` for fast-moving proof-verification APIs
- `circuits` for pre-built ZK game circuit builders (Experimental)
- `session` for end-to-end session UX (`SessionManager`, `SessionStatus`; Beta)
- `cors` for HTTP gateway policy validation and dynamic origin allowlists
- `test` for the on-chain game sandbox (`GameHarness`, `Scenario`, `WorldFixture`; `testutils` feature)

# Golden Path

```rust
use cougr_core::app::{named_system, GameApp, ScheduleStage};
use cougr_core::{Position, SystemConfig};
use soroban_sdk::Env;

let env = Env::default();
let mut app = GameApp::new(&env);

app.add_systems((
    named_system("spawn_player", |world, env| {
        let player = world.spawn_entity();
        world.set_typed(env, player, &Position::new(1, 2));
    })
    .in_stage(ScheduleStage::Startup),
    named_system("tick", |_world, _env| {})
        .with_config(SystemConfig::new().in_stage(ScheduleStage::Update)),
));

app.run(&env).unwrap();
```

# Stability

- ECS runtime and storage: Stable
- `app`: Stable
- `standards`: Stable
- Accounts: Beta
- `zk::stable`: Stable
- `zk::experimental`: Experimental
"#]

extern crate alloc;

// Global allocator for WASM
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Macros must be declared before modules that use them
#[macro_use]
pub mod macros;

// Public product domains
pub mod accounts;
pub mod archetype_world;
mod change_tracker;
pub mod circuits;
pub mod commands;
pub mod component;
/// HTTP gateway policy tooling: CORS configuration validation and a dynamic
/// origin allowlist for servers in front of a game contract.
pub mod cors;
#[cfg(feature = "debug")]
#[doc(hidden)]
pub mod debug;
pub mod ecs;
pub mod ecs_events;
pub mod error;
pub mod event;
pub mod game;
mod hooks;
mod incremental;
mod observers;
pub mod plugin;
pub mod query;
pub mod resource;
pub mod rich_component;
pub mod scheduler;
pub mod session;
pub mod simple_world;
pub mod standards;
mod system;
pub mod zk;

#[cfg(feature = "testutils")]
pub mod test;

// Root-level golden path re-exports.
pub use archetype_world::{ArchetypeQuery, ArchetypeQueryBuilder, ArchetypeWorld};
pub use commands::CommandQueue;
pub use component::{Component, ComponentId, ComponentStorage, ComponentTrait, Position};
pub use ecs::{RuntimeWorld, RuntimeWorldMut, WorldBackend};
pub use error::{CougrError, CougrResult};
pub use event::{Event, EventReader, EventWriter};
pub use plugin::{GameApp, Plugin, PluginGroup};
pub use query::{QueryStorage, SimpleQuery, SimpleQueryBuilder};
pub use resource::Resource;
pub use resource::ResourceTrait;
pub use scheduler::{ScheduleError, ScheduleStage, SimpleScheduler, SystemConfig, SystemGroup};
pub use simple_world::SimpleWorld;

/// Default gameplay runtime surface for new Cougr projects.
pub mod app {
    pub use super::{
        CommandQueue, GameApp, Plugin, PluginGroup, Resource, ResourceTrait, RuntimeWorld,
        RuntimeWorldMut, ScheduleError, ScheduleStage, SimpleQuery, SimpleQueryBuilder,
        SimpleScheduler, SimpleWorld, SystemConfig, SystemGroup,
    };
    pub use crate::ecs_events::ObservableComponentTrait;
    pub use crate::rich_component::RichComponentTrait;
    pub use crate::system::{
        context_system, named_app_system, named_context_system, named_system, world_system,
        AppSystem, SimpleSystem, SystemContext, SystemSpec,
    };
}

/// Beta account and session surface.
///
/// This namespace mirrors [`accounts`] but makes its product role explicit.
pub mod auth {
    pub use super::accounts::*;
}

/// Privacy surface split by maturity tier.
///
/// New code should prefer [`privacy::stable`] for defended contracts and only
/// opt into [`privacy::experimental`] knowingly.
pub mod privacy {
    pub use super::zk::{
        experimental, stable, G1Point, G2Point, Groth16Proof, Scalar, VerificationKey, ZKError,
    };
}

/// Stable operational and contract standards.
///
/// This namespace mirrors [`standards`] while making the adoption boundary
/// clearer for application code.
pub mod ops {
    pub use super::standards::*;
}

/// Common ECS imports for the default onboarding path.
pub mod prelude {
    pub use super::simple_world::EntityId;
    pub use super::{
        ArchetypeWorld, CommandQueue, Component, ComponentStorage, ComponentTrait, GameApp,
        PluginGroup, Position, QueryStorage, Resource, RuntimeWorld, RuntimeWorldMut, SimpleQuery,
        SimpleQueryBuilder, SimpleWorld, WorldBackend,
    };
    pub use crate::ecs_events::ObservableComponentTrait;
    pub use crate::game::SorobanGame;
    pub use crate::rich_component::RichComponentTrait;
    pub use crate::system::SystemContext;
}

/// Advanced runtime primitives that remain supported but are not part of the
/// smallest onboarding surface.
pub mod runtime {
    pub use super::ecs_events::ObservableComponentTrait;
    pub use super::observers::ComponentEvent;
    pub use super::rich_component::RichComponentTrait;
    pub use super::{
        archetype_world::{ArchetypeQueryCache, ArchetypeQueryState},
        change_tracker::{ChangeTracker, TrackedWorld},
        hooks::{HookRegistry, HookedWorld},
        incremental::{StorageWorld, WorldMetadata},
        observers::{ObservedWorld, ObserverRegistry},
        query::{SimpleQueryCache, SimpleQueryState},
        resource::Resource,
        system::{
            context_system, named_app_system, named_context_system, named_system, world_system,
            AppSystem, SimpleSystem, SystemContext, SystemSpec,
        },
        Event, EventReader, EventWriter, Plugin, PluginGroup, QueryStorage, RuntimeWorld,
        RuntimeWorldMut, ScheduleError, ScheduleStage, SimpleQuery, SimpleQueryBuilder,
        SimpleScheduler, SystemConfig, WorldBackend,
    };
}

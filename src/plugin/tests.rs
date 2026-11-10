use super::*;
use crate::resource::GameState as RuntimeGameState;
use soroban_sdk::{symbol_short, Bytes, Env};

struct TestPlugin;

impl Plugin for TestPlugin {
    fn name(&self) -> &'static str {
        "test_plugin"
    }

    fn build(&self, app: &mut GameApp) {
        app.add_system("test_system", test_system_fn);
    }
}

struct HookPlugin;

impl Plugin for HookPlugin {
    fn name(&self) -> &'static str {
        "hook_plugin"
    }

    fn build(&self, app: &mut GameApp) {
        app.add_hook_on_add(symbol_short!("pos"), noop_add_hook);
    }
}

fn test_system_fn(world: &mut SimpleWorld, env: &Env) {
    let e = world.spawn_entity();
    let data = Bytes::from_array(env, &[0xFF]);
    world.add_component(e, symbol_short!("marker"), data);
}

fn noop_add_hook(
    _entity_id: crate::simple_world::EntityId,
    _component_type: &Symbol,
    _data: &Bytes,
) {
}

#[test]
fn test_game_app_new() {
    let env = Env::default();
    let app = GameApp::new(&env);
    assert_eq!(app.plugin_count(), 0);
    assert_eq!(app.system_count(), 0);
}

#[test]
fn test_add_plugin() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(TestPlugin);

    assert_eq!(app.plugin_count(), 1);
    assert!(app.has_plugin("test_plugin"));
    assert_eq!(app.system_count(), 1);
}

#[test]
fn test_duplicate_plugin_skipped() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(TestPlugin);
    app.add_plugin(TestPlugin);

    assert_eq!(app.plugin_count(), 1);
    assert_eq!(app.system_count(), 1);
}

#[test]
fn test_plugin_configures_hooks() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(HookPlugin);

    assert_eq!(app.plugin_count(), 1);
    assert_eq!(app.hooks().add_hook_count(), 1);
}

#[test]
fn test_run_executes_systems() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(TestPlugin);
    app.run(&env).unwrap();

    assert!(app.world().has_component(1, &symbol_short!("marker")));
}

#[test]
fn test_into_world() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(TestPlugin);
    app.run(&env).unwrap();

    let world = app.into_world();
    assert!(world.has_component(1, &symbol_short!("marker")));
}

#[test]
fn test_with_world() {
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let e1 = world.spawn_entity();
    let data = Bytes::from_array(&env, &[1]);
    world.add_component(e1, symbol_short!("pre"), data);

    let app = GameApp::with_world(world);
    assert!(app.world().has_component(e1, &symbol_short!("pre")));
}

#[test]
fn test_add_system_directly() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_system("direct", test_system_fn);

    assert_eq!(app.system_count(), 1);
    assert_eq!(app.plugin_count(), 0);
}

#[test]
fn test_multiple_plugins() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugin(TestPlugin);
    app.add_plugin(HookPlugin);

    assert_eq!(app.plugin_count(), 2);
    assert!(app.has_plugin("test_plugin"));
    assert!(app.has_plugin("hook_plugin"));
}

#[test]
fn test_startup_system_runs_once() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_startup_system("startup", test_system_fn);

    app.run(&env).unwrap();
    app.run(&env).unwrap();

    assert!(app.world().has_component(1, &symbol_short!("marker")));
    assert_eq!(app.world().component_count(&symbol_short!("marker")), 1);
}

#[test]
fn test_add_plugins_group_and_resources() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_plugins((TestPlugin, HookPlugin));

    let mut state = RuntimeGameState::new();
    state.increment_score(42);
    app.insert_resource(&env, &state);

    let loaded = app.get_resource::<RuntimeGameState>(&env).unwrap();
    assert_eq!(loaded.score, 42);
    assert_eq!(app.plugin_count(), 2);
    assert_eq!(app.resources().len(), 1);
}

#[test]
fn test_add_system_in_stage_and_context_variants() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_system_in_stage(ScheduleStage::PreUpdate, "spawn_a", test_system_fn);
    app.add_context_system_in_stage(ScheduleStage::PostUpdate, "spawn_b", |context| {
        let entity = context.world_mut().spawn_entity();
        let data = Bytes::from_array(context.env(), &[0x0A]);
        context
            .world_mut()
            .add_component(entity, symbol_short!("ctx"), data);
    });

    app.run(&env).unwrap();

    assert!(app.world().has_component(1, &symbol_short!("marker")));
    assert!(app.world().has_component(2, &symbol_short!("ctx")));
}

#[test]
fn test_add_simple_system_and_configure_system() {
    let env = Env::default();
    let mut app = GameApp::new(&env);
    app.add_simple_system(
        "spawn_ctx",
        crate::system::context_system(|context| {
            let entity = context.world_mut().spawn_entity();
            let data = Bytes::from_array(context.env(), &[0xAB]);
            context
                .world_mut()
                .add_component(entity, symbol_short!("simp"), data);
        }),
    );
    app.configure_system(
        "spawn_ctx",
        SystemConfig::new().in_stage(ScheduleStage::Cleanup),
    )
    .unwrap();

    app.run(&env).unwrap();

    assert!(app.world().has_component(1, &symbol_short!("simp")));
}

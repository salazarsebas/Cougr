//! Space Invaders systems and Cougr scheduler integration helpers.

#[cfg(test)]
use cougr_core::{GameApp, ScheduleStage, SimpleWorld, SystemConfig};
#[cfg(test)]
use soroban_sdk::Env;

/// Collision test used by the update loop for position-based entities.
pub(crate) fn check_collision(x1: i32, y1: i32, x2: i32, y2: i32, tolerance: i32) -> bool {
    (x1 - x2).abs() < tolerance && (y1 - y2).abs() < tolerance
}

/// Exercises the Cougr `GameApp` tick path for this transitional example.
#[cfg(test)]
pub(crate) fn run_gameapp_tick(env: &Env) {
    let mut app = GameApp::new(env);
    app.add_system_with_config(
        "space_invaders_tick_boundary",
        |_world: &mut SimpleWorld, _env: &Env| {},
        SystemConfig::new().in_stage(ScheduleStage::Update),
    );
    app.run(env).unwrap();
}

use super::*;
use soroban_sdk::testutils::Address as _;

fn setup() -> (Env, Address, SokobanContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(SokobanContract, ());
    let client = SokobanContractClient::new(&env, &contract_id);
    let player = Address::generate(&env);
    (env, player, client)
}

fn tiles(env: &Env, codes: &[u32]) -> Vec<u32> {
    Vec::from_slice(env, codes)
}

/// A 3x2 grid: player, one crate, one goal two tiles east of the player on
/// the bottom row, with an empty row above. Pushing east once shoves the
/// crate onto the goal and solves the level; pushing north moves into the
/// empty row above without leaving the grid.
fn corridor_level(env: &Env, client: &SokobanContractClient<'static>, player: &Address) {
    let grid = tiles(
        env,
        &[
            TILE_PLAYER,
            TILE_CRATE,
            TILE_GOAL,
            TILE_EMPTY,
            TILE_EMPTY,
            TILE_EMPTY,
        ],
    );
    client.init_level(player, &3, &2, &grid);
}

// ─── Initialization ──────────────────────────────────────────────────────────

#[test]
fn init_level_sets_player_position_and_meta() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    let pos = client.player_position();
    assert_eq!((pos.x, pos.y), (0, 0));

    let meta = client.level_meta();
    assert_eq!(meta.move_count, 0);
    assert!(!meta.solved);
    assert_eq!((meta.width, meta.height), (3, 2));
}

// ─── Happy path / rule and invariant ─────────────────────────────────────────

#[test]
fn pushing_crate_onto_goal_solves_level() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    client.push(&player, &EAST);

    let meta = client.level_meta();
    assert!(meta.solved);
    assert_eq!(meta.move_count, 1);
    let pos = client.player_position();
    assert_eq!((pos.x, pos.y), (1, 0));
}

#[test]
fn move_into_empty_space_increments_move_count_without_solving() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    client.push(&player, &NORTH);

    let pos = client.player_position();
    assert_eq!((pos.x, pos.y), (0, 1));
    let meta = client.level_meta();
    assert_eq!(meta.move_count, 1);
    assert!(!meta.solved);
}

// ─── Blocked pushes ──────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "blocked by wall")]
fn push_into_wall_panics() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, TILE_WALL, TILE_CRATE, TILE_GOAL]);
    client.init_level(&player, &4, &1, &grid);

    client.push(&player, &EAST);
}

#[test]
#[should_panic(expected = "crate blocked by wall")]
fn push_crate_into_wall_panics() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, TILE_CRATE, TILE_WALL, TILE_GOAL]);
    client.init_level(&player, &4, &1, &grid);

    client.push(&player, &EAST);
}

#[test]
#[should_panic(expected = "crate blocked by another crate")]
fn push_crate_into_crate_panics() {
    let (env, player, client) = setup();
    let grid = tiles(
        &env,
        &[TILE_PLAYER, TILE_CRATE, TILE_CRATE, TILE_GOAL, TILE_GOAL],
    );
    client.init_level(&player, &5, &1, &grid);

    client.push(&player, &EAST);
}

// ─── Bounds checking ──────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "out of bounds")]
fn push_player_out_of_bounds_panics() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    client.push(&player, &WEST);
}

#[test]
#[should_panic(expected = "out of bounds")]
fn push_crate_out_of_bounds_panics() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_CRATE, TILE_PLAYER, TILE_GOAL]);
    client.init_level(&player, &3, &1, &grid);

    client.push(&player, &WEST);
}

// ─── Post-win and authorization ──────────────────────────────────────────────

#[test]
#[should_panic(expected = "level already solved")]
fn push_after_solving_panics() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    client.push(&player, &EAST);
    client.push(&player, &NORTH);
}

#[test]
#[should_panic(expected = "caller is not the registered player")]
fn push_from_wrong_caller_panics() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    let impostor = Address::generate(&env);
    client.push(&impostor, &EAST);
}

#[test]
#[should_panic(expected = "level already initialized by another player")]
fn init_level_from_second_player_on_live_level_panics() {
    let (env, player, client) = setup();
    corridor_level(&env, &client, &player);

    let other = Address::generate(&env);
    let grid = tiles(&env, &[TILE_PLAYER, TILE_CRATE, TILE_GOAL]);
    client.init_level(&other, &3, &1, &grid);
}

// ─── Invalid levels ───────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "level has zero dimension")]
fn init_level_rejects_zero_dimension() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[]);
    client.init_level(&player, &0, &1, &grid);
}

#[test]
#[should_panic(expected = "level exceeds maximum dimension")]
fn init_level_rejects_oversized_dimension() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[0; 21]);
    client.init_level(&player, &21, &1, &grid);
}

#[test]
#[should_panic(expected = "tile count does not match dimensions")]
fn init_level_rejects_mismatched_tile_count() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, TILE_EMPTY]);
    client.init_level(&player, &3, &1, &grid);
}

#[test]
#[should_panic(expected = "level must have exactly one player tile")]
fn init_level_rejects_missing_player() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_CRATE, TILE_GOAL]);
    client.init_level(&player, &2, &1, &grid);
}

#[test]
#[should_panic(expected = "level must have exactly one player tile")]
fn init_level_rejects_two_players() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, TILE_PLAYER, TILE_CRATE, TILE_GOAL]);
    client.init_level(&player, &4, &1, &grid);
}

#[test]
#[should_panic(expected = "level must have at least one crate and one goal")]
fn init_level_rejects_no_crates_or_goals() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, TILE_EMPTY]);
    client.init_level(&player, &2, &1, &grid);
}

#[test]
#[should_panic(expected = "crate count must match goal count")]
fn init_level_rejects_mismatched_crate_goal_count() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, TILE_CRATE, TILE_CRATE, TILE_GOAL]);
    client.init_level(&player, &4, &1, &grid);
}

#[test]
#[should_panic(expected = "unknown tile code")]
fn init_level_rejects_unknown_tile_code() {
    let (env, player, client) = setup();
    let grid = tiles(&env, &[TILE_PLAYER, 99, TILE_CRATE, TILE_GOAL]);
    client.init_level(&player, &4, &1, &grid);
}

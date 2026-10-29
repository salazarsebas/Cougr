//! Benchmark suite for the Soroban-first ECS path.
//!
//! Uses `std::time::Instant` since `criterion` is not compatible with `#![no_std]`.
//! Run with: `cargo bench`

use std::time::Instant;
use std::vec::Vec as StdVec;

use cougr_core::archetype_world::ArchetypeWorld;
use cougr_core::component::ComponentStorage;
use cougr_core::query::{SimpleQueryBuilder, SimpleQueryCache};
use cougr_core::scheduler::{ScheduleStage, SimpleScheduler, SystemConfig};
use cougr_core::simple_world::SimpleWorld;
use soroban_sdk::{symbol_short, Bytes, Env, Symbol};

const SPAWN_COUNTS: [usize; 3] = [100, 250, 500];
const STORAGE_COUNTS: [usize; 3] = [50, 100, 150];
const QUERY_COUNTS: [usize; 2] = [50, 100];
const QUERY_CACHE_COUNTS: [usize; 2] = [50, 100];
const MUTATION_COUNTS: [usize; 2] = [50, 100];

fn per_op_us(elapsed: std::time::Duration, ops: usize) -> f64 {
    elapsed.as_micros() as f64 / ops as f64
}

fn print_header(title: &str) {
    println!("\n--- {title} ---");
}

fn bench_spawn_entities() {
    print_header("Spawn Entities");
    for count in SPAWN_COUNTS {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);

        let start = Instant::now();
        for _ in 0..count {
            world.spawn_entity();
        }
        let elapsed = start.elapsed();
        println!(
            "  SimpleWorld spawn {count}: {elapsed:?} ({:.1} us/entity)",
            per_op_us(elapsed, count)
        );
    }
}

fn bench_component_insert_and_lookup() {
    print_header("Component Insert / Lookup");
    for count in STORAGE_COUNTS {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let entities: StdVec<u32> = (0..count).map(|_| world.spawn_entity()).collect();
        let data = Bytes::from_slice(&env, &[0u8; 16]);

        let start = Instant::now();
        for &eid in &entities {
            world.add_component(eid, symbol_short!("pos"), data.clone());
            world.add_component_with_storage(
                eid,
                symbol_short!("tag"),
                data.clone(),
                ComponentStorage::Sparse,
            );
        }
        let insert_elapsed = start.elapsed();

        let start = Instant::now();
        for &eid in &entities {
            let _ = world.get_component(eid, &symbol_short!("pos"));
            let _ = world.get_component(eid, &symbol_short!("tag"));
        }
        let get_elapsed = start.elapsed();

        println!(
            "  {count} entities insert: {insert_elapsed:?} ({:.1} us/entity), lookup: {get_elapsed:?} ({:.1} us/entity)",
            per_op_us(insert_elapsed, count),
            per_op_us(get_elapsed, count),
        );
    }
}

fn bench_query_paths() {
    print_header("Query Paths");
    for count in QUERY_COUNTS {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let data = Bytes::from_slice(&env, &[1u8; 16]);

        for index in 0..count {
            let entity = world.spawn_entity();
            world.add_component(entity, symbol_short!("pos"), data.clone());
            if index % 2 == 0 {
                world.add_component(entity, symbol_short!("vel"), data.clone());
            }
            if index % 5 == 0 {
                world.add_component_with_storage(
                    entity,
                    symbol_short!("tag"),
                    data.clone(),
                    ComponentStorage::Sparse,
                );
            }
        }

        let simple_query = SimpleQueryBuilder::new(&env)
            .with_component(symbol_short!("pos"))
            .with_component(symbol_short!("vel"))
            .build();
        let sparse_query = SimpleQueryBuilder::new(&env)
            .with_any_component(symbol_short!("tag"))
            .include_sparse()
            .build();
        let mut cache = SimpleQueryCache::from_query(simple_query.clone(), &env);

        let start = Instant::now();
        let simple_result_len = simple_query.execute(&world, &env).len();
        let simple_elapsed = start.elapsed();

        let start = Instant::now();
        let cached_first_len = cache.execute(&world, &env).len();
        let cached_first_elapsed = start.elapsed();

        let start = Instant::now();
        let cached_second_len = cache.execute(&world, &env).len();
        let cached_second_elapsed = start.elapsed();

        let start = Instant::now();
        let sparse_len = sparse_query.execute(&world, &env).len();
        let sparse_elapsed = start.elapsed();

        println!("  {count} entities simple query: {simple_elapsed:?} ({simple_result_len} hits)");
        println!(
            "  {count} entities cached query: first {cached_first_elapsed:?}, second {cached_second_elapsed:?} ({cached_first_len}/{cached_second_len} hits)"
        );
        println!(
            "  {count} entities sparse-inclusive query: {sparse_elapsed:?} ({sparse_len} hits)"
        );
    }
}

fn bench_query_cache_invalidation() {
    print_header("Query Cache Invalidation");
    for count in QUERY_CACHE_COUNTS {
        let env = Env::default();
        let mut world = SimpleWorld::new(&env);
        let data = Bytes::from_slice(&env, &[7u8; 16]);

        for index in 0..count {
            let entity = world.spawn_entity();
            world.add_component(entity, symbol_short!("pos"), data.clone());
            if index % 2 == 0 {
                world.add_component(entity, symbol_short!("vel"), data.clone());
            }
        }

        let query = SimpleQueryBuilder::new(&env)
            .with_component(symbol_short!("pos"))
            .with_component(symbol_short!("vel"))
            .build();
        let mut cache = SimpleQueryCache::from_query(query, &env);

        let _ = cache.execute(&world, &env);

        let start = Instant::now();
        let _ = cache.execute(&world, &env);
        let warm_elapsed = start.elapsed();

        let entity = world.spawn_entity();
        world.add_component(entity, symbol_short!("pos"), data.clone());
        world.add_component(entity, symbol_short!("vel"), data);

        let start = Instant::now();
        let refreshed_len = cache.execute(&world, &env).len();
        let invalidated_elapsed = start.elapsed();

        println!(
            "  {count} entities cached warm read: {warm_elapsed:?}, after mutation refresh: {invalidated_elapsed:?} ({refreshed_len} hits)"
        );
    }
}

fn bench_scheduler_validation_and_execution() {
    print_header("Scheduler");
    let env = Env::default();
    let mut world = SimpleWorld::new(&env);
    let mut scheduler = SimpleScheduler::new();

    fn spawn_system(world: &mut SimpleWorld, env: &Env) {
        let entity = world.spawn_entity();
        world.add_component(
            entity,
            symbol_short!("spawned"),
            Bytes::from_slice(env, &[1]),
        );
    }

    scheduler.add_system_with_config(
        "spawn",
        spawn_system,
        SystemConfig::new().in_stage(ScheduleStage::PreUpdate),
    );
    scheduler.add_system_with_config(
        "mark",
        |world: &mut SimpleWorld, env: &Env| {
            let entities = world.get_entities_with_component(&symbol_short!("spawned"), env);
            for i in 0..entities.len() {
                let entity = entities.get(i).unwrap();
                world.add_component(entity, symbol_short!("seen"), Bytes::from_slice(env, &[2]));
            }
        },
        SystemConfig::new()
            .in_stage(ScheduleStage::PreUpdate)
            .after("spawn"),
    );

    let start = Instant::now();
    scheduler.run_all(&mut world, &env).unwrap();
    let elapsed = start.elapsed();

    println!(
        "  2-stage schedule validation + execution: {elapsed:?} ({} systems)",
        scheduler.system_count()
    );
}

fn populate_simple_world(
    env: &Env,
    count: usize,
    required: &[Symbol],
    optional: &[Symbol],
) -> SimpleWorld {
    let mut world = SimpleWorld::new(env);
    let bytes = Bytes::from_slice(env, &[0u8; 16]);

    for index in 0..count {
        let entity = world.spawn_entity();
        for component in required {
            world.add_component(entity, component.clone(), bytes.clone());
        }
        for (offset, component) in optional.iter().enumerate() {
            if (index + offset) % 2 == 0 {
                world.add_component(entity, component.clone(), bytes.clone());
            }
        }
    }

    world
}

fn populate_archetype_world(
    env: &Env,
    count: usize,
    required: &[Symbol],
    optional: &[Symbol],
) -> ArchetypeWorld {
    let mut world = ArchetypeWorld::new(env);
    let bytes = Bytes::from_slice(env, &[0u8; 16]);

    for index in 0..count {
        let entity = world.spawn_entity();
        for component in required {
            world.add_component(entity, component.clone(), bytes.clone(), env);
        }
        for (offset, component) in optional.iter().enumerate() {
            if (index + offset) % 2 == 0 {
                world.add_component(entity, component.clone(), bytes.clone(), env);
            }
        }
    }

    world
}

fn bench_backend_query_comparison() {
    print_header("Backend Query Comparison");
    let required = [symbol_short!("pos"), symbol_short!("vel")];
    let optional = [
        symbol_short!("hp"),
        symbol_short!("team"),
        symbol_short!("buff"),
    ];

    for count in QUERY_COUNTS {
        let simple_env = Env::default();
        let archetype_env = Env::default();
        let simple = populate_simple_world(&simple_env, count, &required, &optional);
        let archetype = populate_archetype_world(&archetype_env, count, &required, &optional);

        let query = SimpleQueryBuilder::new(&simple_env)
            .with_component(symbol_short!("pos"))
            .with_component(symbol_short!("vel"))
            .with_component(symbol_short!("hp"))
            .build();

        let start = Instant::now();
        let simple_hits = query.execute(&simple, &simple_env).len();
        let simple_elapsed = start.elapsed();

        let start = Instant::now();
        let archetype_hits = archetype
            .query(
                &[
                    symbol_short!("pos"),
                    symbol_short!("vel"),
                    symbol_short!("hp"),
                ],
                &archetype_env,
            )
            .len();
        let archetype_elapsed = start.elapsed();

        println!(
            "  {count} entities three-component query: SimpleWorld {simple_elapsed:?} ({simple_hits} hits), ArchetypeWorld {archetype_elapsed:?} ({archetype_hits} hits)"
        );
    }
}

fn bench_backend_mutation_comparison() {
    print_header("Backend Structural Mutation Comparison");
    for count in MUTATION_COUNTS {
        let simple_env = Env::default();
        let archetype_env = Env::default();
        let simple_bytes = Bytes::from_slice(&simple_env, &[3u8; 16]);
        let archetype_bytes = Bytes::from_slice(&archetype_env, &[3u8; 16]);

        let mut simple = SimpleWorld::new(&simple_env);
        let simple_entities: StdVec<u32> = (0..count).map(|_| simple.spawn_entity()).collect();
        for &entity in &simple_entities {
            simple.add_component(entity, symbol_short!("pos"), simple_bytes.clone());
        }

        let mut archetype = ArchetypeWorld::new(&archetype_env);
        let archetype_entities: StdVec<u32> =
            (0..count).map(|_| archetype.spawn_entity()).collect();
        for &entity in &archetype_entities {
            archetype.add_component(
                entity,
                symbol_short!("pos"),
                archetype_bytes.clone(),
                &archetype_env,
            );
        }

        let start = Instant::now();
        for &entity in &simple_entities {
            simple.add_component(entity, symbol_short!("vel"), simple_bytes.clone());
        }
        let simple_add_elapsed = start.elapsed();

        let start = Instant::now();
        for &entity in &archetype_entities {
            archetype.add_component(
                entity,
                symbol_short!("vel"),
                archetype_bytes.clone(),
                &archetype_env,
            );
        }
        let archetype_add_elapsed = start.elapsed();

        let start = Instant::now();
        for &entity in &simple_entities {
            let _ = simple.remove_component(entity, &symbol_short!("vel"));
        }
        let simple_remove_elapsed = start.elapsed();

        let start = Instant::now();
        for &entity in &archetype_entities {
            let _ = archetype.remove_component(entity, &symbol_short!("vel"), &archetype_env);
        }
        let archetype_remove_elapsed = start.elapsed();

        println!(
            "  {count} entities add vel: SimpleWorld {simple_add_elapsed:?} ({:.1} us/entity), ArchetypeWorld {archetype_add_elapsed:?} ({:.1} us/entity)",
            per_op_us(simple_add_elapsed, count),
            per_op_us(archetype_add_elapsed, count),
        );
        println!(
            "  {count} entities remove vel: SimpleWorld {simple_remove_elapsed:?} ({:.1} us/entity), ArchetypeWorld {archetype_remove_elapsed:?} ({:.1} us/entity)",
            per_op_us(simple_remove_elapsed, count),
            per_op_us(archetype_remove_elapsed, count),
        );
    }
}

fn main() {
    println!("=== Cougr ECS Benchmarks ===");
    bench_spawn_entities();
    bench_component_insert_and_lookup();
    bench_query_paths();
    bench_query_cache_invalidation();
    bench_scheduler_validation_and_execution();
    bench_backend_query_comparison();
    bench_backend_mutation_comparison();
    println!("\n=== Done ===");
}

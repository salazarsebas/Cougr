# Cougr Performance Guide

## Purpose

This document explains the current performance model for Cougr's Soroban-first ECS path.

It is not a promise of fixed gas costs. It is a guide to the data structures and tradeoffs that determine query and scheduling behavior.

The practical question it should answer is:

- which backend should I use
- where should a component live
- what kinds of mutations are cheap versus expensive

## SimpleWorld Query Model

`SimpleWorld` now maintains direct component indexes:

- `table_index` for table-backed components
- `all_index` for table + sparse lookups

That changes the expected behavior of the common query paths:

- `get_table_entities_with_component()` uses the direct table index
- `get_all_entities_with_component()` uses the all-storage index
- `SimpleQuery` selects the narrowest available required component index before filtering

This is the default performance story for gameplay loops.

Use `SimpleWorld` by default when:

- your hot loop is dominated by one- or two-component scans
- you mutate entity composition often
- you rely on table vs sparse placement to control scan scope

Use `ArchetypeWorld` when:

- your hot loop is dominated by repeated multi-component queries
- entity compositions are relatively stable after setup
- you are willing to pay more for add/remove migrations to get tighter query scopes

## Storage Tradeoffs

Table storage:

- optimized for repeated scans
- should back components that appear in hot gameplay loops

Sparse storage:

- better for infrequent markers or tags
- excluded from table-only scans by default

If a sparse component starts showing up in tick-critical queries, it is usually a signal that the component belongs in table storage.

Prescriptive rule:

- if you scan it every tick, it probably belongs in table storage
- if you mostly address it directly or use it as a sparse marker, keep it sparse

## Scheduler Tradeoffs

`SimpleScheduler` now validates stage-local dependencies before execution.

Costs introduced by the stronger model:

- dependency validation during run planning
- topological ordering within each stage

Benefits:

- explicit execution order
- early detection of invalid schedules
- safer composition as system counts grow

This is a good trade in Soroban-oriented contracts because schedule size is typically small relative to the cost of incorrect execution order.

## Benchmark Focus Areas

Benchmarks should answer these practical questions:

- how many entities can the indexed query path scan efficiently
- when does `ArchetypeWorld` outperform `SimpleWorld`
- what is the cost of adding/removing indexed components
- what is the cost of stage validation and deferred command application

The current benchmark suite in `benches/ecs_bench.rs` covers these paths directly.

It now includes:

- entity spawn cost
- component insert / lookup cost
- indexed query vs sparse-inclusive query cost
- cache warm-read vs invalidated-read behavior
- scheduler validation + execution cost
- `SimpleWorld` vs `ArchetypeWorld` multi-component query comparison
- `SimpleWorld` vs `ArchetypeWorld` structural mutation comparison

## Reading The Current Benchmarks

Interpret the benchmark output in this order:

1. `Query Paths`
   If plain indexed queries and cached queries are already cheap enough, stay on `SimpleWorld`.
2. `Backend Query Comparison`
   If `ArchetypeWorld` is materially better on your real multi-component query shape, it may be worth adopting.
3. `Backend Structural Mutation Comparison`
   If archetype migration is significantly more expensive for your workload, do not switch just because query numbers look better in isolation.
4. `Query Cache Invalidation`
   If your world mutates every tick, cache benefits may collapse; optimize data shape first.

## Decision Heuristics

Choose `SimpleWorld` when:

- gameplay writes are frequent
- entity compositions change often
- table/sparse separation gives you enough control
- your queries are broad but predictable

Choose `ArchetypeWorld` when:

- the same multi-component query runs constantly
- compositions are mostly fixed after startup
- entity migration cost is amortized over many reads

Keep `GameApp`, `SimpleWorld`, and `SimpleQuery` as the default performance story for new Soroban gameplay code.

## Interpretation Rules

Use benchmark output to compare patterns, not to claim universal throughput numbers.

For real contracts, evaluate:

- data shape
- component cardinality
- table vs sparse placement
- how often the world mutates between repeated queries

Performance guidance should always be tied back to those conditions.

If benchmark results and your data shape disagree, trust the data shape first.

## Background

`SimpleWorld` writes the whole world in one instance entry. `src/incremental` (`StorageWorld`) writes dirty entities only. That split is real in the core crate and invisible in `cougr new`. Game authors who need it read library source.

## Objective

A new CLI template persists a small match with `StorageWorld`, and a test shows that an unchanged entity is not rewritten on flush.

## Scope

| In scope | Out of scope |
|---|---|
| Template `incremental` (name can change if it collides, say so in the PR) | Rewriting `SimpleWorld` |
| A match with more than one entity so partial updates mean something | A general migration of every example |
| Test that compares writes for a touched entity and an untouched one | A new storage engine |
| README tradeoff: cheaper partial writes, more expensive full scans | |
| CLI registration and `cougr check` | |

## Direction

Use `StorageWorld` as it exists. Do not fork it into the template. The game can be small (two entities, one numeric field each). The point is the storage behavior, proven in tests, not a new ruleset.

## Review

Wait for assignment before opening a pull request. The PR description must include `Closes #<this issue>`. Explain the approach in the PR body, and include test output for the paths this issue names.

## Community

Telegram: https://t.me/cougrengine
A star on https://github.com/salazarsebas/Cougr helps the project keep growing.
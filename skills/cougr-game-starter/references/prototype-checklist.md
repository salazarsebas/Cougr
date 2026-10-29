# Prototype Checklist

Use this checklist before considering the first prototype complete.

## Scope

- The game has a clearly defined playable loop.
- The prototype focuses on one coherent mode of play.
- Secondary systems were deferred unless they were required for the loop.

## Structure

- `cougr-core` is used to shape the state and logic, not only listed as a dependency.
- Components and systems are named by gameplay responsibility.
- Contract entrypoints are narrow and readable.
- The directory structure is understandable without tribal knowledge.

## Build And Tooling

- Commands and docs use `wasm32v1-none`.
- The project includes `cargo fmt --check`.
- The project includes `cargo clippy --all-targets --all-features -- -D warnings`.
- The project includes `cargo test`.

## Tests

- Initialization is covered.
- The main legal gameplay path is covered.
- At least one invalid action is rejected in tests.
- Terminal or round-transition behavior is covered when applicable.

## Documentation

- The `README` explains what the game currently does.
- The `README` explains how to build and test it.
- The `README` states current limitations when the prototype is intentionally incomplete.
- Documentation avoids stale generated reports and temporary planning notes.

## Delivery Standard

The prototype should be easy for another engineer to:

- run
- understand
- extend
- refactor safely

If one of those is missing, the implementation is not done yet.

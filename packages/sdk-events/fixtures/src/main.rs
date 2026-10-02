//! Emits one `COUGR` event of each family through the real `cougr-core` event
//! types, then writes their XDR to stdout.
//!
//! This is the source of `packages/sdk-events/fixtures/events.json`. It runs on
//! `Env::default()` with `testutils`, so nothing here touches a network:
//!
//! ```sh
//! npm run fixtures   # cargo run --manifest-path fixtures/Cargo.toml | node fixtures/build.mjs > fixtures/events.json
//! ```
//!
//! The decoder tests never execute Rust; they read the committed JSON.

use cougr_core::ecs_events::{ComponentRemovedEvent, ComponentSetEvent, RichComponentChangedEvent};
use soroban_sdk::{
    contract, contractimpl,
    testutils::Events,
    xdr::{ContractEventBody, Limits, WriteXdr},
    Bytes, Env, Symbol,
};

/// Minimal registered contract so `as_contract` gives the events a contract
/// context, exactly like a real invocation would.
#[contract]
pub struct FixtureContract;

#[contractimpl]
impl FixtureContract {}

fn main() {
    let env = Env::default();
    let contract_id = env.register(FixtureContract, ());

    env.as_contract(&contract_id, || {
        ComponentSetEvent {
            component_type: Symbol::new(&env, "position"),
            entity_id: 7,
            data: Bytes::from_array(&env, &[1, 2, 3, 4]),
        }
        .publish(&env);

        ComponentRemovedEvent {
            component_type: Symbol::new(&env, "position"),
            entity_id: 7,
        }
        .publish(&env);

        RichComponentChangedEvent {
            component_type: Symbol::new(&env, "player_profile"),
            entity_id: 7,
        }
        .publish(&env);
    });

    let all = env.events().all();
    let kinds = ["set", "del", "rich"];
    for (index, event) in all.events().iter().enumerate() {
        let ContractEventBody::V0(body) = &event.body;
        println!("{}", kinds[index]);
        for topic in body.topics.iter() {
            println!("topic {}", topic.to_xdr_base64(Limits::none()).unwrap());
        }
        println!("value {}", body.data.to_xdr_base64(Limits::none()).unwrap());
    }
}

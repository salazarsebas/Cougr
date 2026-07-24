# Cougr Terminology Glossary

This glossary defines core technical and architectural terms used across the Cougr repository and documentation suite.

## Terminology Definitions

### Component
An immutable or stateful data structure attached to an entity within the Entity Component System (ECS) architecture. Represented as a Soroban storage cell.

### System
Stateless logic functions that operate on entities possessing specific component subscriptions. Executed deterministically on-chain.

### World
The global state container and contract entry point managing entity registrations, component definitions, and system invocation boundaries.

### Archetype
A unique bitset composition signature representing a distinct combination of components attached to entities in the world container.

### Session Key
A temporary, restricted authorization key allowing seamless user transaction signing without requiring repeated main wallet prompt confirmations.

### Observed Component
A component configured with indexing triggers to emit real-time state change events consumed by off-chain indexers and showcase previews.

### Curated Surface
The verified, high-level developer API surface exposed by core Cougr modules, distinguished from low-level internal storage primitives.

// StorageWorld match types and pure helpers.
//
// StorageWorld persists entity components individually in Soroban storage.
// Only the components that changed on a given move are rewritten, so a
// "partial update" is observable: submit a move for entity A and entity B's
// fields remain byte-identical to what they were before the call.
//
// NOTE: These types are written against the interface documented in issue #330
// / PR #358 (not yet merged as of this writing). "storageworld" is the working
// template name; it may change when that PR lands. Confirm method names and
// field names against the final template README before deploying.

// ── Entity component types ──────────────────────────────────────────────────

/** A single entity stored in the StorageWorld match. */
export interface Entity {
  /** Contract-assigned entity id (u32). */
  id: number;
  /** Display name stored as a component on this entity. */
  name: string;
  /** Numeric value component – used to demonstrate a partial update. */
  value: number;
  /** Owner address stored as a component on this entity. */
  owner: string;
}

/** Full match state returned by get_state. */
export interface MatchState {
  /** Ordered list of entities in this match. Length >= 2 is required so that
   *  partial updates are meaningful (only one entity's component changes per
   *  move call). */
  entities: Entity[];
  /** Match-level move counter, incremented by every make_move call. */
  move_count: number;
  /** True once the match has been initialised (init_match called). */
  active: boolean;
}

// ── Type guards ─────────────────────────────────────────────────────────────

function isEntity(value: unknown): value is Entity {
  if (!value || typeof value !== "object") return false;
  const e = value as Partial<Entity>;
  return (
    typeof e.id === "number" &&
    Number.isInteger(e.id) &&
    e.id >= 0 &&
    typeof e.name === "string" &&
    typeof e.value === "number" &&
    typeof e.owner === "string"
  );
}

export function isMatchState(value: unknown): value is MatchState {
  if (!value || typeof value !== "object") return false;
  const s = value as Partial<MatchState>;
  return (
    Array.isArray(s.entities) &&
    s.entities.every(isEntity) &&
    typeof s.move_count === "number" &&
    Number.isInteger(s.move_count) &&
    s.move_count >= 0 &&
    typeof s.active === "boolean"
  );
}

// ── Pure helpers ─────────────────────────────────────────────────────────────

/** Returns the entity whose owner matches the given address, or undefined. */
export function entityForAddress(
  entities: Entity[],
  address: string
): Entity | undefined {
  return entities.find((e) => e.owner === address);
}

/** Returns true if the connected address owns any entity in the match. */
export function isParticipant(entities: Entity[], address: string): boolean {
  return entities.some((e) => e.owner === address);
}

/** Format an address for display: first 8 chars + ellipsis + last 6 chars. */
export function shortAddress(address: string): string {
  if (address.length <= 14) return address;
  return `${address.slice(0, 8)}…${address.slice(-6)}`;
}

/** Human-readable label for the match status. */
export function matchStatusLabel(state: MatchState): string {
  if (!state.active) return "Not started";
  return `In progress · ${state.move_count} move${state.move_count === 1 ? "" : "s"}`;
}

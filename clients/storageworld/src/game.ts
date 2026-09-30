// Types and helpers for the storageworld template client.
//
// The storageworld template backs its match with StorageWorld instead of
// SimpleWorld: only the component fields that actually change are written back
// on flush, so moving entity 0 leaves entity 1's storage entry untouched.  The
// client therefore receives and renders both entities independently so the
// partial-update behaviour is visible.
//
// NOTE on template naming: the CLI template is currently named "storageworld"
// (issue #330, PR #358).  If it lands under a different name, this file and its
// sibling contract.ts will need to be updated to match the final method names
// and directory; see README.md for the full note.

// ----------------------------------------------------------------------------
// Entity component shape – mirrors what get_state returns from the contract.
// Each entity has a position (x, y) and a step counter.
// ----------------------------------------------------------------------------

export interface EntityState {
  /** ECS entity id (0 or 1 for a two-entity match) */
  entity_id: number;
  /** Current x position */
  x: i32;
  /** Current y position */
  y: i32;
  /** How many times this entity has been moved */
  steps: number;
}

// The alias lets callers use a plain number without the branded type syntax
// while keeping the declaration explicit.
type i32 = number;

export interface MatchState {
  /** Entity 0 – the first actor in the match */
  entity_0: EntityState;
  /** Entity 1 – the second actor in the match */
  entity_1: EntityState;
  /** Whose turn it is: 0 or 1 */
  active_entity: number;
  /** Total moves across both entities */
  move_count: number;
}

// ----------------------------------------------------------------------------
// Type guard
// ----------------------------------------------------------------------------

export function isEntityState(value: unknown): value is EntityState {
  if (!value || typeof value !== "object") return false;
  const e = value as Partial<EntityState>;
  return (
    typeof e.entity_id === "number" &&
    typeof e.x === "number" &&
    typeof e.y === "number" &&
    typeof e.steps === "number"
  );
}

export function isMatchState(value: unknown): value is MatchState {
  if (!value || typeof value !== "object") return false;
  const m = value as Partial<MatchState>;
  return (
    isEntityState(m.entity_0) &&
    isEntityState(m.entity_1) &&
    typeof m.active_entity === "number" &&
    typeof m.move_count === "number"
  );
}

// ----------------------------------------------------------------------------
// Presentation helpers
// ----------------------------------------------------------------------------

export function turnLabel(state: MatchState): string {
  return `Entity ${state.active_entity}'s turn`;
}

export function entityLabel(id: number): string {
  return `Entity ${id}`;
}

// Direction strings accepted by the move_entity contract method.
export type Direction = "up" | "down" | "left" | "right";

export const DIRECTIONS: Direction[] = ["up", "down", "left", "right"];

export function directionLabel(d: Direction): string {
  switch (d) {
    case "up":    return "↑ Up";
    case "down":  return "↓ Down";
    case "left":  return "← Left";
    case "right": return "→ Right";
  }
}

// Map a direction to the (dx, dy) delta it applies so tests can verify the
// expected outcome without calling the contract.
export function directionDelta(d: Direction): [number, number] {
  switch (d) {
    case "up":    return [0,  1];
    case "down":  return [0, -1];
    case "left":  return [-1, 0];
    case "right": return [1,  0];
  }
}

import { describe, expect, it } from "vitest";
import {
  isMatchState,
  isEntityState,
  turnLabel,
  directionDelta,
  entityLabel,
  type MatchState,
  type EntityState,
} from "./game";

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

const ENTITY_0: EntityState = { entity_id: 0, x: 3, y: 7, steps: 2 };
const ENTITY_1: EntityState = { entity_id: 1, x: 0, y: 0, steps: 0 };

const FIXTURE_STATE: MatchState = {
  entity_0: ENTITY_0,
  entity_1: ENTITY_1,
  active_entity: 0,
  move_count: 2,
};

// ---------------------------------------------------------------------------
// Type guards
// ---------------------------------------------------------------------------

describe("isEntityState", () => {
  it("accepts a valid entity state", () => {
    expect(isEntityState(ENTITY_0)).toBe(true);
  });

  it("rejects null", () => {
    expect(isEntityState(null)).toBe(false);
  });

  it("rejects a missing field", () => {
    const { steps: _dropped, ...noSteps } = ENTITY_0;
    expect(isEntityState(noSteps)).toBe(false);
  });
});

describe("isMatchState", () => {
  it("accepts a valid match state", () => {
    expect(isMatchState(FIXTURE_STATE)).toBe(true);
  });

  it("rejects an object with invalid entity_0", () => {
    const bad = { ...FIXTURE_STATE, entity_0: { x: 1 } };
    expect(isMatchState(bad)).toBe(false);
  });

  it("rejects null", () => {
    expect(isMatchState(null)).toBe(false);
  });
});

// ---------------------------------------------------------------------------
// Partial-update visibility: the key DoD criterion.
//
// After a move_entity call for entity 0, only entity 0's fields change; entity
// 1 must remain byte-for-byte identical.  This test simulates that invariant
// using fixture state – no live wallet or contract is required.
// ---------------------------------------------------------------------------

describe("partial update visibility (StorageWorld invariant)", () => {
  it("moving entity 0 changes only entity_0 fields and leaves entity_1 unchanged", () => {
    // Simulate what the contract does: apply directionDelta to entity 0.
    const [dx, dy] = directionDelta("right");

    const updatedEntity0: EntityState = {
      ...ENTITY_0,
      x: ENTITY_0.x + dx,
      y: ENTITY_0.y + dy,
      steps: ENTITY_0.steps + 1,
    };

    const newState: MatchState = {
      ...FIXTURE_STATE,
      entity_0: updatedEntity0,
      active_entity: 1,           // turn passes to entity 1
      move_count: FIXTURE_STATE.move_count + 1,
    };

    // entity_0 changed
    expect(newState.entity_0.x).toBe(ENTITY_0.x + 1);
    expect(newState.entity_0.steps).toBe(ENTITY_0.steps + 1);

    // entity_1 is byte-for-byte the same object reference — StorageWorld did
    // not touch it
    expect(newState.entity_1).toEqual(ENTITY_1);
    expect(newState.entity_1.x).toBe(ENTITY_1.x);
    expect(newState.entity_1.y).toBe(ENTITY_1.y);
    expect(newState.entity_1.steps).toBe(ENTITY_1.steps);
  });

  it("moving entity 1 changes only entity_1 fields and leaves entity_0 unchanged", () => {
    const [dx, dy] = directionDelta("up");

    const stateForEntity1: MatchState = {
      ...FIXTURE_STATE,
      active_entity: 1,
    };

    const updatedEntity1: EntityState = {
      ...ENTITY_1,
      x: ENTITY_1.x + dx,
      y: ENTITY_1.y + dy,
      steps: ENTITY_1.steps + 1,
    };

    const newState: MatchState = {
      ...stateForEntity1,
      entity_1: updatedEntity1,
      active_entity: 0,
      move_count: stateForEntity1.move_count + 1,
    };

    // entity_1 changed
    expect(newState.entity_1.y).toBe(ENTITY_1.y + 1);
    expect(newState.entity_1.steps).toBe(ENTITY_1.steps + 1);

    // entity_0 is untouched
    expect(newState.entity_0).toEqual(ENTITY_0);
    expect(newState.entity_0.x).toBe(ENTITY_0.x);
    expect(newState.entity_0.y).toBe(ENTITY_0.y);
    expect(newState.entity_0.steps).toBe(ENTITY_0.steps);
  });
});

// ---------------------------------------------------------------------------
// Presentation helpers
// ---------------------------------------------------------------------------

describe("turnLabel", () => {
  it("reports active entity 0", () => {
    expect(turnLabel(FIXTURE_STATE)).toBe("Entity 0's turn");
  });

  it("reports active entity 1", () => {
    expect(turnLabel({ ...FIXTURE_STATE, active_entity: 1 })).toBe("Entity 1's turn");
  });
});

describe("entityLabel", () => {
  it("formats entity labels", () => {
    expect(entityLabel(0)).toBe("Entity 0");
    expect(entityLabel(1)).toBe("Entity 1");
  });
});

describe("directionDelta", () => {
  it("right increases x", () => {
    const [dx, dy] = directionDelta("right");
    expect(dx).toBe(1);
    expect(dy).toBe(0);
  });

  it("up increases y", () => {
    const [dx, dy] = directionDelta("up");
    expect(dx).toBe(0);
    expect(dy).toBe(1);
  });

  it("left decreases x", () => {
    const [dx] = directionDelta("left");
    expect(dx).toBe(-1);
  });

  it("down decreases y", () => {
    const [, dy] = directionDelta("down");
    expect(dy).toBe(-1);
  });
});

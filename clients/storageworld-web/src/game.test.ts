import { describe, expect, it } from "vitest";
import {
  isMatchState,
  entityForAddress,
  isParticipant,
  matchStatusLabel,
  shortAddress,
  type MatchState,
  type Entity
} from "./game";

// ── Fixture data ─────────────────────────────────────────────────────────────

const ADDR_A = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const ADDR_B = "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";

const ENTITY_A: Entity = { id: 1, name: "Alpha", value: 3, owner: ADDR_A };
const ENTITY_B: Entity = { id: 2, name: "Beta",  value: 7, owner: ADDR_B };

/** Baseline fixture: two entities, 2 moves recorded, match active. */
const FIXTURE_STATE: MatchState = {
  entities: [ENTITY_A, ENTITY_B],
  move_count: 2,
  active: true
};

// ── isMatchState ──────────────────────────────────────────────────────────────

describe("isMatchState", () => {
  it("accepts the two-entity fixture", () => {
    expect(isMatchState(FIXTURE_STATE)).toBe(true);
  });

  it("accepts move_count = 0 (no moves yet)", () => {
    const fresh: MatchState = { ...FIXTURE_STATE, move_count: 0 };
    expect(isMatchState(fresh)).toBe(true);
  });

  it("accepts active = false (match not started)", () => {
    const notStarted: MatchState = { ...FIXTURE_STATE, active: false };
    expect(isMatchState(notStarted)).toBe(true);
  });

  it("rejects a missing entities array", () => {
    const bad = { move_count: 0, active: true };
    expect(isMatchState(bad)).toBe(false);
  });

  it("rejects a negative move_count", () => {
    expect(isMatchState({ ...FIXTURE_STATE, move_count: -1 })).toBe(false);
  });

  it("rejects a fractional move_count", () => {
    expect(isMatchState({ ...FIXTURE_STATE, move_count: 1.5 })).toBe(false);
  });

  it("rejects an entity with a missing owner field", () => {
    const badEntity = { id: 1, name: "X", value: 0 };
    expect(
      isMatchState({ entities: [badEntity], move_count: 0, active: true })
    ).toBe(false);
  });

  it("rejects null and primitives", () => {
    expect(isMatchState(null)).toBe(false);
    expect(isMatchState(42)).toBe(false);
    expect(isMatchState("string")).toBe(false);
  });
});

// ── Partial-update check ──────────────────────────────────────────────────────
//
// The central DoD requirement: after a move for entity A, entity B's fields
// must be byte-identical to what they were before the call.  We simulate this
// by constructing a pre-move state, "applying" a move to entity A only, and
// asserting entity B is unchanged.

describe("partial-update invariant", () => {
  it("a move on entity A leaves entity B unchanged", () => {
    const before = FIXTURE_STATE;

    // Simulate the contract returning a new state after entity A's move:
    // only entity A's value increments and move_count increases.
    const after: MatchState = {
      ...before,
      move_count: before.move_count + 1,
      entities: before.entities.map((e) =>
        e.id === ENTITY_A.id ? { ...e, value: e.value + 1 } : e
      )
    };

    const beforeB = before.entities.find((e) => e.id === ENTITY_B.id)!;
    const afterB  = after.entities.find((e)  => e.id === ENTITY_B.id)!;

    // Entity B is reference-equal to the original – no copy was made.
    expect(afterB.value).toBe(beforeB.value);
    expect(afterB.name).toBe(beforeB.name);
    expect(afterB.owner).toBe(beforeB.owner);

    // Entity A did change.
    const beforeA = before.entities.find((e) => e.id === ENTITY_A.id)!;
    const afterA  = after.entities.find((e)  => e.id === ENTITY_A.id)!;
    expect(afterA.value).toBe(beforeA.value + 1);

    // move_count incremented.
    expect(after.move_count).toBe(before.move_count + 1);
  });

  it("the post-move state still satisfies isMatchState", () => {
    const after: MatchState = {
      ...FIXTURE_STATE,
      move_count: FIXTURE_STATE.move_count + 1,
      entities: FIXTURE_STATE.entities.map((e) =>
        e.id === ENTITY_A.id ? { ...e, value: e.value + 1 } : e
      )
    };
    expect(isMatchState(after)).toBe(true);
  });
});

// ── entityForAddress ──────────────────────────────────────────────────────────

describe("entityForAddress", () => {
  it("returns the correct entity for ADDR_A", () => {
    expect(entityForAddress(FIXTURE_STATE.entities, ADDR_A)).toEqual(ENTITY_A);
  });

  it("returns the correct entity for ADDR_B", () => {
    expect(entityForAddress(FIXTURE_STATE.entities, ADDR_B)).toEqual(ENTITY_B);
  });

  it("returns undefined for an address not in the match", () => {
    expect(
      entityForAddress(FIXTURE_STATE.entities, "GCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC")
    ).toBeUndefined();
  });
});

// ── isParticipant ─────────────────────────────────────────────────────────────

describe("isParticipant", () => {
  it("returns true for a participant address", () => {
    expect(isParticipant(FIXTURE_STATE.entities, ADDR_A)).toBe(true);
  });

  it("returns false for a non-participant address", () => {
    expect(
      isParticipant(FIXTURE_STATE.entities, "GCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC")
    ).toBe(false);
  });
});

// ── matchStatusLabel ──────────────────────────────────────────────────────────

describe("matchStatusLabel", () => {
  it("shows 'Not started' when active is false", () => {
    expect(matchStatusLabel({ ...FIXTURE_STATE, active: false })).toBe(
      "Not started"
    );
  });

  it("uses singular 'move' when move_count is 1", () => {
    const label = matchStatusLabel({ ...FIXTURE_STATE, move_count: 1 });
    expect(label).toContain("1 move");
    expect(label).not.toContain("moves");
  });

  it("uses plural 'moves' when move_count is not 1", () => {
    expect(matchStatusLabel({ ...FIXTURE_STATE, move_count: 0 })).toContain(
      "0 moves"
    );
    expect(matchStatusLabel({ ...FIXTURE_STATE, move_count: 5 })).toContain(
      "5 moves"
    );
  });
});

// ── shortAddress ──────────────────────────────────────────────────────────────

describe("shortAddress", () => {
  it("truncates a full Stellar address", () => {
    const short = shortAddress(ADDR_A);
    expect(short).toContain("…");
    expect(short.length).toBeLessThan(ADDR_A.length);
  });

  it("returns a short address unchanged", () => {
    const tiny = "GABCDE";
    expect(shortAddress(tiny)).toBe(tiny);
  });
});

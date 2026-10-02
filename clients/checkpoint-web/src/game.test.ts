import { describe, expect, it } from "vitest";
import {
  actionRejection,
  isActionResult,
  isMatchState,
  parseStateHash,
  statusLabel,
  hashStateSummary
} from "./game";
import type { ActionResult, MatchState } from "./game";

const PLAYER = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";

/** A checkpoint that was committed at tick 120 with score 7, match still running. */
export const COMMITTED_FIXTURE: MatchState = {
  player: PLAYER,
  last_tick: 120,
  last_hash: 48879,
  last_score: 7,
  status: 0
};

/** The ActionResult the contract returns when a checkpoint commit succeeds. */
export const COMMIT_OK: ActionResult = {
  success: true,
  match_state: COMMITTED_FIXTURE,
  message: "ok"
};

/** The ActionResult for a dispute inside the window whose claimed hash differs. */
export const DISPUTE_OK: ActionResult = {
  success: true,
  match_state: { ...COMMITTED_FIXTURE, status: 2 },
  message: "disputed"
};

/** The ActionResult for a dispute after the window closed: tx succeeds, action is rejected. */
export const DISPUTE_LATE: ActionResult = {
  success: false,
  match_state: COMMITTED_FIXTURE,
  message: "toolate"
};

describe("checkpoint client state handling", () => {
  it("accepts a committed checkpoint fixture", () => {
    expect(isMatchState(COMMITTED_FIXTURE)).toBe(true);
    expect(statusLabel(COMMITTED_FIXTURE.status)).toBe("Running");
    expect(COMMITTED_FIXTURE.last_tick).toBe(120);
    expect(COMMITTED_FIXTURE.last_score).toBe(7);
  });

  it("accepts a dispute result inside the window", () => {
    expect(isActionResult(DISPUTE_OK)).toBe(true);
    expect(DISPUTE_OK.success).toBe(true);
    expect(DISPUTE_OK.message).toBe("disputed");
    expect(statusLabel(DISPUTE_OK.match_state.status)).toBe("Disputed");
  });

  it("treats a late dispute as a rejection, not a crash", () => {
    expect(isActionResult(DISPUTE_LATE)).toBe(true);
    expect(DISPUTE_LATE.success).toBe(false);
    expect(actionRejection(DISPUTE_LATE.message)).toBe("The dispute window has already closed.");
  });

  it("accepts bigint u64 hashes from a real contract payload", () => {
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: 0n })).toBe(true);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: 48879n })).toBe(true);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: 18446744073709551615n })).toBe(true);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: "18446744073709551615" })).toBe(true);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: -1n })).toBe(false);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: 18446744073709551616n })).toBe(false);
  });

  it("accepts a bigint hash through isActionResult for a commit and a late dispute", () => {
    const committed: ActionResult = {
      success: true,
      match_state: { ...COMMITTED_FIXTURE, last_hash: 18446744073709551615n },
      message: "ok"
    };
    expect(isActionResult(committed)).toBe(true);
    expect(parseStateHash(committed.match_state.last_hash)?.label).toBe("0xffffffffffffffff");

    const late: ActionResult = {
      success: false,
      match_state: { ...COMMITTED_FIXTURE, last_hash: 48879n },
      message: "toolate"
    };
    expect(isActionResult(late)).toBe(true);
    expect(late.success).toBe(false);
    expect(actionRejection(late.message)).toBe("The dispute window has already closed.");
  });

  it("maps every contract rejection code to a safe UI message", () => {
    expect(actionRejection("notrun")).toBe("The match is not running.");
    expect(actionRejection("oldtick")).toBe("That tick is not newer than the last committed checkpoint.");
    expect(actionRejection("badtick")).toBe("The disputed tick does not match the committed checkpoint.");
    expect(actionRejection("nohash")).toBe("The claimed hash matches the committed checkpoint, so there is nothing to dispute.");
    expect(actionRejection("window")).toBe("The dispute window is still open, so the match cannot be finalised yet.");
    expect(actionRejection("notplay")).toBe("The connected wallet is not authorised for this match.");
    expect(actionRejection("")).toBe("The contract rejected the action.");
    expect(actionRejection("somethingnew")).toBe("somethingnew");
  });

  it("rejects malformed state payloads instead of trusting the contract", () => {
    expect(isMatchState(null)).toBe(false);
    expect(isMatchState({ ...COMMITTED_FIXTURE, status: 9 })).toBe(false);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_tick: -1 })).toBe(false);
    expect(isMatchState({ ...COMMITTED_FIXTURE, last_hash: 1.5 })).toBe(false);
    expect(isMatchState({ ...COMMITTED_FIXTURE, player: 42 })).toBe(false);
    expect(isActionResult({ ...DISPUTE_OK, success: "yes" })).toBe(false);
    expect(isActionResult({ ...DISPUTE_OK, match_state: null })).toBe(false);
  });

  it("parses u64 hashes exactly, including values above MAX_SAFE_INTEGER", () => {
    expect(parseStateHash(48879)?.label).toBe("0x000000000000beef");
    expect(parseStateHash(0)?.label).toBe("0x0000000000000000");
    expect(parseStateHash(48879n)?.label).toBe("0x000000000000beef");
    expect(parseStateHash(0n)?.label).toBe("0x0000000000000000");
    expect(parseStateHash(18446744073709551615n)?.label).toBe("0xffffffffffffffff");
    expect(parseStateHash(18446744073709551616n)).toBeNull();
    expect(parseStateHash(-1n)).toBeNull();
    expect(parseStateHash("18446744073709551615")?.label).toBe("0xffffffffffffffff");
    expect(parseStateHash("18446744073709551616")).toBeNull();
    expect(parseStateHash("-1")).toBeNull();
    expect(parseStateHash("12ab")).toBeNull();
    expect(parseStateHash(1.5)).toBeNull();
  });

  it("derives a stable u64 hash from an off-chain state summary", () => {
    expect(hashStateSummary("120:7")).toBe(hashStateSummary("120:7"));
    expect(hashStateSummary("120:7")).not.toBe(hashStateSummary("121:7"));
    expect(hashStateSummary("120:7")).toMatch(/^[0-9]+$/);
  });
});

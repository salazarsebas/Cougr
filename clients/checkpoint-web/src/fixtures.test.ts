import { describe, expect, it } from "vitest";
import {
  COMMITTED_FIXTURE,
  COMMIT_OK,
  DISPUTE_LATE,
  DISPUTE_OK
} from "./game.test";

describe("checkpoint client fixtures", () => {
  it("keeps the committed checkpoint and its action result consistent", () => {
    expect(COMMIT_OK.match_state).toEqual(COMMITTED_FIXTURE);
    expect(COMMIT_OK.success).toBe(true);
  });

  it("keeps the in-window dispute and late dispute outcomes distinct", () => {
    expect(DISPUTE_OK.success).toBe(true);
    expect(DISPUTE_OK.match_state.status).toBe(2);
    expect(DISPUTE_LATE.success).toBe(false);
    expect(DISPUTE_LATE.match_state.status).toBe(0);
    expect(DISPUTE_LATE.message).toBe("toolate");
  });
});

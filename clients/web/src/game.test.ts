import { describe, expect, it } from "vitest";
import { isGameState, moveRejection, statusLabel } from "./game";
import type { GameState } from "./game";

describe("turn-based client state handling", () => {
  it("maps contract move rejection codes to safe UI messages", () => {
    expect(moveRejection("notturn")).toBe("It is not your turn.");
    expect(moveRejection("occupied")).toBe("That cell is already occupied.");
    expect(moveRejection("gameover")).toBe("The match is already over.");
  });

  it("accepts a nine-cell fixture state and its win status", () => {
    const fixture: GameState = {
      cells: [1, 2, 1, 0, 1, 2, 0, 0, 1],
      player_x: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
      player_o: "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
      is_x_turn: false,
      move_count: 6,
      status: 1
    };
    expect(isGameState(fixture)).toBe(true);
    expect(statusLabel(fixture.status)).toBe("X wins");
  });
});

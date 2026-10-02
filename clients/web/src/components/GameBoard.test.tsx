import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { GameBoard } from "./GameBoard";
import type { GameState } from "../game";

const WIN_FIXTURE: GameState = {
  cells: [1, 2, 1, 0, 1, 2, 0, 0, 1],
  player_x: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
  player_o: "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
  is_x_turn: false,
  move_count: 6,
  status: 1
};

describe("GameBoard", () => {
  it("renders the fixture board used by the win state", () => {
    const html = renderToStaticMarkup(<GameBoard state={WIN_FIXTURE} disabled={false} onMove={() => undefined} />);
    expect(html).toContain(">X</button>");
    expect(html).toContain(">O</button>");
  });

  it("locks all cells after the fixture reaches a win", () => {
    const html = renderToStaticMarkup(<GameBoard state={WIN_FIXTURE} disabled={false} onMove={() => undefined} />);
    expect((html.match(/disabled=""/g) ?? []).length).toBe(9);
  });
});

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { MatchPanel } from "./MatchPanel";
import type { MatchState } from "../game";

const COMMITTED_FIXTURE: MatchState = {
  player: "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
  last_tick: 120,
  last_hash: 48879,
  last_score: 7,
  status: 0
};

describe("MatchPanel", () => {
  it("renders the fixture match with its committed checkpoint", () => {
    const html = renderToStaticMarkup(<MatchPanel state={COMMITTED_FIXTURE} busy={false} onRefresh={() => undefined} />);
    expect(html).toContain("Running");
    expect(html).toContain("120");
    expect(html).toContain("0x000000000000beef");
    expect(html).toContain("7");
  });

  it("renders a bigint u64 hash above MAX_SAFE_INTEGER exactly", () => {
    const html = renderToStaticMarkup(
      <MatchPanel
        state={{ ...COMMITTED_FIXTURE, last_hash: 18446744073709551615n }}
        busy={false}
        onRefresh={() => undefined}
      />
    );
    expect(html).toContain("0xffffffffffffffff");
  });

  it("shows the empty state before a match is loaded", () => {
    const html = renderToStaticMarkup(<MatchPanel state={null} busy={false} onRefresh={() => undefined} />);
    expect(html).toContain("No checkpoint match loaded");
  });
});

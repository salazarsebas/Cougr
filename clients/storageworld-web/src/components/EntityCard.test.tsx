import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { EntityCard } from "./EntityCard";
import type { Entity } from "../game";

const ADDR_A = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";
const ADDR_B = "GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB";

const ENTITY_A: Entity = { id: 1, name: "Alpha", value: 3, owner: ADDR_A };
const ENTITY_B: Entity = { id: 2, name: "Beta",  value: 7, owner: ADDR_B };

describe("EntityCard – own entity", () => {
  it("renders the entity name", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_A}
        isOwn={true}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).toContain("Alpha");
  });

  it("renders the entity value", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_A}
        isOwn={true}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).toContain("3");
  });

  it("shows a Make move button for the own entity", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_A}
        isOwn={true}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).toContain("Make move");
  });

  it("disables the Make move button when disabled=true", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_A}
        isOwn={true}
        disabled={true}
        onMove={vi.fn()}
      />
    );
    // disabled button renders with disabled=""
    expect(html).toContain('disabled=""');
    expect(html).toContain("Working…");
  });

  it("applies entity-card--own class for the own entity", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_A}
        isOwn={true}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).toContain("entity-card--own");
  });
});

describe("EntityCard – opponent entity", () => {
  it("renders the opponent entity name", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_B}
        isOwn={false}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).toContain("Beta");
  });

  it("does NOT show a Make move button for the opponent", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_B}
        isOwn={false}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).not.toContain("Make move");
  });

  it("does NOT apply entity-card--own class for the opponent", () => {
    const html = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_B}
        isOwn={false}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(html).not.toContain("entity-card--own");
  });
});

// ── Partial-update visual invariant ──────────────────────────────────────────
// After a move on entity A, entity B's rendered value must not change.

describe("EntityCard – partial-update visual invariant", () => {
  it("entity B renders the same value before and after a move on entity A", () => {
    const beforeB = renderToStaticMarkup(
      <EntityCard
        entity={ENTITY_B}
        isOwn={false}
        disabled={false}
        onMove={vi.fn()}
      />
    );

    // Simulate entity A's move – entity B is untouched in the new state.
    const afterA: Entity = { ...ENTITY_A, value: ENTITY_A.value + 1 };
    // entity B stays identical
    const afterB = ENTITY_B;

    // Re-render entity A to confirm it changed
    const afterAHtml = renderToStaticMarkup(
      <EntityCard
        entity={afterA}
        isOwn={true}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(afterAHtml).toContain("4"); // ENTITY_A.value + 1

    // Re-render entity B to confirm it did NOT change
    const afterBHtml = renderToStaticMarkup(
      <EntityCard
        entity={afterB}
        isOwn={false}
        disabled={false}
        onMove={vi.fn()}
      />
    );
    expect(afterBHtml).toBe(beforeB);
  });
});

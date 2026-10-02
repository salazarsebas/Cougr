import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { EntityCard } from "./EntityCard";
import type { EntityState } from "../game";

const ENTITY_0: EntityState = { entity_id: 0, x: 3, y: 7, steps: 2 };
const ENTITY_1: EntityState = { entity_id: 1, x: 0, y: 0, steps: 0 };

describe("EntityCard", () => {
  it("shows the active badge only for the active entity", () => {
    const active = renderToStaticMarkup(
      <EntityCard state={ENTITY_0} isActive={true} disabled={false} onMove={() => undefined} />
    );
    const idle = renderToStaticMarkup(
      <EntityCard state={ENTITY_1} isActive={false} disabled={false} onMove={() => undefined} />
    );
    expect(active).toContain("Active");
    expect(idle).not.toContain("Active");
  });

  it("disables move buttons when the entity is not active", () => {
    const html = renderToStaticMarkup(
      <EntityCard state={ENTITY_1} isActive={false} disabled={false} onMove={() => undefined} />
    );
    // All 4 direction buttons should be disabled
    expect((html.match(/disabled=""/g) ?? []).length).toBe(4);
  });

  it("renders entity 0 stat values correctly", () => {
    const html = renderToStaticMarkup(
      <EntityCard state={ENTITY_0} isActive={true} disabled={false} onMove={() => undefined} />
    );
    expect(html).toContain(">3<");  // x
    expect(html).toContain(">7<");  // y
    expect(html).toContain(">2<");  // steps
  });
});

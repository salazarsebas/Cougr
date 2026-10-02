import type { Entity } from "../game";
import { shortAddress } from "../game";

export interface EntityCardProps {
  entity: Entity;
  /** True when this entity is owned by the connected wallet. */
  isOwn: boolean;
  /** True when the UI is busy (transaction in flight). */
  disabled: boolean;
  /** Called when the user clicks "Make move" for this entity. */
  onMove: (entityId: number) => void;
}

/**
 * Renders one entity row.
 *
 * Shows the entity's name, id, value component, and owner address. If the
 * entity belongs to the connected wallet it also shows a "Make move" button so
 * the user can submit a move that increments that entity's value while leaving
 * the other entity's storage entry untouched (the partial-update demo).
 */
export function EntityCard({ entity, isOwn, disabled, onMove }: EntityCardProps) {
  return (
    <div
      className={`entity-card${isOwn ? " entity-card--own" : ""}`}
      aria-label={`Entity ${entity.id}: ${entity.name}`}
    >
      <div className="entity-card__header">
        <span className="label">
          {isOwn ? "Your entity" : "Opponent's entity"}
        </span>
        <code className="entity-card__id">id {entity.id}</code>
      </div>

      <p className="entity-card__name">{entity.name}</p>

      <div className="entity-card__row">
        <span className="label">Value</span>
        <strong>{entity.value}</strong>
      </div>

      <div className="entity-card__row">
        <span className="label">Owner</span>
        <code title={entity.owner}>{shortAddress(entity.owner)}</code>
      </div>

      {isOwn && (
        <button
          type="button"
          disabled={disabled}
          onClick={() => onMove(entity.id)}
          aria-label={`Make move for entity ${entity.id}`}
        >
          {disabled ? "Working…" : "Make move"}
        </button>
      )}
    </div>
  );
}

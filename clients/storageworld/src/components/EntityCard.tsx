import type { EntityState, Direction } from "../game";
import { DIRECTIONS, directionLabel } from "../game";

export interface EntityCardProps {
  state: EntityState;
  /** True when this entity is the active mover and buttons should be enabled */
  isActive: boolean;
  /** True when a transaction is in flight */
  disabled: boolean;
  onMove: (direction: Direction) => void;
}

/**
 * Renders one entity's state and movement controls.
 *
 * Both entity cards are always visible so the partial-update behaviour of
 * StorageWorld is directly apparent: moving entity 0 updates only its own
 * position and steps counter; entity 1 stays exactly as it was.
 */
export function EntityCard({ state, isActive, disabled, onMove }: EntityCardProps) {
  return (
    <div className={`entity-card${isActive ? " entity-card--active" : ""}`} aria-label={`Entity ${state.entity_id}`}>
      <div className="entity-header">
        <span className="label">Entity {state.entity_id}</span>
        {isActive && <span className="badge">Active</span>}
      </div>

      <div className="entity-stats">
        <div className="stat">
          <span className="stat-label">x</span>
          <span className="stat-value">{state.x}</span>
        </div>
        <div className="stat">
          <span className="stat-label">y</span>
          <span className="stat-value">{state.y}</span>
        </div>
        <div className="stat">
          <span className="stat-label">steps</span>
          <span className="stat-value">{state.steps}</span>
        </div>
      </div>

      <div className="move-controls" aria-label={`Move controls for entity ${state.entity_id}`}>
        {DIRECTIONS.map((dir) => (
          <button
            key={dir}
            type="button"
            disabled={disabled || !isActive}
            onClick={() => onMove(dir)}
            aria-label={`Move entity ${state.entity_id} ${dir}`}
          >
            {directionLabel(dir)}
          </button>
        ))}
      </div>
    </div>
  );
}

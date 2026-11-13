import type { Piece } from "../types";
import { statusVisuals } from "../game/statusVisuals";

export function StatusBadges({ piece, onSelect }: { piece: Piece; onSelect?: () => void }) {
  const statuses = statusVisuals(piece);
  if (!statuses.length) return null;
  return <span className={`board-statuses${statuses.length > 6 ? " dense" : ""}`} data-status-count={statuses.length}>
    {statuses.map((status) => <span key={status.kind} className={`board-status board-status-${status.kind}`}
      data-status={status.kind} title={`${status.label} · ${status.description}`}
      onClick={onSelect ? (event) => { event.stopPropagation(); onSelect(); } : undefined}>
      <span className="board-status-icon" aria-hidden="true">{status.icon}</span>
      {status.count > 1 && !["hunger", "aimed", "propellant"].includes(status.kind) &&
        <span className="board-status-count">{status.count}</span>}
    </span>)}
  </span>;
}

import { statusVisuals } from "./game/statusVisuals";

export function PieceStatuses({ statuses = [], compact = false, details = false }: { statuses?: string[]; compact?: boolean; details?: boolean }) {
  if (!statuses.length) return null;
  const descriptions = new Map(statusVisuals({ statuses, ward: false }).map((status) => [status.label, status.description]));
  return <div className={`piece-statuses ${compact ? "compact" : ""}`}>
    {statuses.map((status) => <span key={status} title={descriptions.get(status)} className={/기아|죄책감|광기|전투불능|약화|종소리|죽음의 문턱/.test(status) ? "harmful" : ""}>{status}{details && descriptions.get(status) && <small>{descriptions.get(status)}</small>}</span>)}
  </div>;
}

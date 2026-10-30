export function PieceStatuses({ statuses = [], compact = false }: { statuses?: string[]; compact?: boolean }) {
  if (!statuses.length) return null;
  return <div className={`piece-statuses ${compact ? "compact" : ""}`}>
    {statuses.map((status) => <span key={status} className={/기아|죄책감|광기|전투불능|약화|종소리/.test(status) ? "harmful" : ""}>{status}</span>)}
  </div>;
}

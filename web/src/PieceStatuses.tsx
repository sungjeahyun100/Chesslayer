export function PieceStatuses({ statuses = [], compact = false }: { statuses?: string[]; compact?: boolean }) {
  if (!statuses.length) return null;
  return <div className={`piece-statuses ${compact ? "compact" : ""}`}>
    {statuses.map((status) => <span key={status} title={status === "죽음의 문턱" ? "다음 전투 시작 시 살아 있다면 즉시 사망합니다." : undefined} className={/기아|죄책감|광기|전투불능|약화|종소리|죽음의 문턱/.test(status) ? "harmful" : ""}>{status}</span>)}
  </div>;
}

import type { Piece } from "../types";
import { PieceImage } from "../PieceImage";

export function BoardPiece({
  piece,
  focused = false,
  previewKey,
}: {
  piece: Piece;
  focused?: boolean;
  previewKey?: string;
}) {
  return (
    <span
      className={`piece owner-${piece.owner} ${piece.ward ? "warded" : ""} ${focused ? "rest-focused" : ""}`}
      style={{
        left: `${(piece.cell % 8) * 12.5}%`,
        top: `${Math.floor(piece.cell / 8) * 12.5}%`,
      }}
      data-piece-id={piece.id}
      data-cell={piece.cell}
      aria-hidden="true"
    >
      <PieceImage pieceKey={previewKey || piece.key} owner={piece.owner} />
      {piece.royal && <span className="royal-mark">★</span>}
      {piece.dice_roll && <span className="dice-mark" title={`주사위 ${piece.dice_roll} · 현재 행마`}>🎲{piece.dice_roll}</span>}
      {piece.ward && <span className="ward-mark" title="가호">✦</span>}
      {piece.temporary_upgrade && <span className="temporary-mark" title="일시 강화 · 이번 전투 한정">↑</span>}
      {piece.self_defense && <span className="self-defense-mark">◆</span>}
      {piece.limiter && <span className="limiter-mark" title={`리미터 해제: 남은 이동 ${piece.limiter.moves_left}회 · 0회가 되면 사망하고 턴 종료`}>{piece.limiter.moves_left}</span>}
      {piece.statuses?.some((status) => /전투불능/.test(status)) && <span className="status-mark disabled-mark" title="영구 전투불능 · 휴식에서 고해성사">×</span>}
      {piece.statuses?.some((status) => /종소리/.test(status)) && <span className="status-mark bell-mark" title="만찬의 종소리 대상">!</span>}
      {piece.statuses?.some((status) => /기아|굶주림|공복/.test(status)) && <span className="hunger-mark" title={piece.statuses.filter((status) => /기아|굶주림|공복/.test(status)).join(" ")}>◌</span>}
    </span>
  );
}

import type { Piece } from "../types";
import { PieceImage } from "../PieceImage";
import { StatusBadges } from "./StatusBadges";

export function BoardPiece({
  piece,
  focused = false,
  previewKey,
  onSelect,
}: {
  piece: Piece;
  focused?: boolean;
  previewKey?: string;
  onSelect?: () => void;
}) {
  return (
    <span
      className={`piece owner-${piece.owner} ${piece.royal ? "royal-piece" : ""} ${piece.ward ? "warded" : ""} ${focused ? "rest-focused" : ""}`}
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
      <StatusBadges piece={piece} onSelect={onSelect} />
      {piece.temporary_upgrade && <span className="temporary-mark" title="일시 강화 · 이번 전투 한정">↑</span>}
      {piece.self_defense && <span className="self-defense-mark">◆</span>}
      {piece.limiter && <span className="limiter-mark" title={`리미터 해제: 남은 이동 ${piece.limiter.moves_left}회 · 0회가 되면 사망하고 턴 종료`}>{piece.limiter.moves_left}</span>}
    </span>
  );
}

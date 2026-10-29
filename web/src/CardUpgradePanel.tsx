import { useEffect, useRef, useState } from "react";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import type { Card, Piece, UpgradePath } from "./types";

export function CardUpgradePanel({ card, piece, paths, onConfirm, onCancel }: {
  card: Card;
  piece: Piece;
  paths: UpgradePath[];
  onConfirm: (target: number) => void;
  onCancel: () => void;
}) {
  const [selected, setSelected] = useState<number>();
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    const reveal = () => {
      if (window.matchMedia("(max-width: 1000px)").matches)
        panel.current?.scrollIntoView({ block: "start", behavior: "smooth" });
    };
    reveal();
    window.addEventListener("resize", reveal);
    return () => window.removeEventListener("resize", reveal);
  }, [piece.id]);
  const path = paths.find((candidate) => candidate.from === piece.kind && candidate.to === selected);
  const preview = paths.find((candidate) => candidate.to === selected);

  return (
    <section className="card-upgrade-picker" ref={panel} aria-label={`${piece.name} 카드 강화`}>
      <div className="card-upgrade-heading">
        <h2>{card.definition.name}</h2>
        <button className="text-button" type="button" onClick={onCancel}>닫기 ×</button>
      </div>
      <div className="card-upgrade-piece">
        <div className="card-upgrade-portrait"><PieceImage pieceKey={piece.key} owner={piece.owner} name={piece.name} /></div>
        <strong>{piece.name} <span>· {piece.square}</span></strong>
        <span className="card-upgrade-cost">{card.definition.cost_per_target ? "1 마나/기" : `${card.definition.cost} 마나`}</span>
      </div>
      <UpgradeTree key={piece.id} piece={{ ...piece, origin: piece.cell, disabled:piece.status.disabled, hunger:piece.status.hunger, upgrades: [], can_upgrade: true, can_revive: false, dead: false }}
        paths={paths} acquired={[]} onSelect={setSelected} onConfirm={onConfirm}
        completed={false} selectionOnly inlineConfirm />
      {preview && <div className="card-upgrade-selection" aria-live="polite">
        <strong>{preview.to_name}</strong>
        <p>{preview.effect}</p>
        {path && !path.unlocked && <small>{path.requirement}</small>}
        {!path && <small>현재 기물에서 선택할 수 없는 경로</small>}
      </div>}
      <div className="card-upgrade-actions">
        {path?.unlocked && <button className="primary card-upgrade-confirm" type="button" onClick={() => onConfirm(path.to)}>강화 확정</button>}
        <button className="text-button card-upgrade-cancel" type="button" onClick={onCancel}>대상 재선택</button>
      </div>
    </section>
  );
}

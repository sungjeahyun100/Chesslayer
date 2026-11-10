import type { Ability, BossState, Piece } from "../types";
import { PieceImage } from "../PieceImage";
import { PieceStatuses } from "../PieceStatuses";
import { movementInfo } from "./pieceInfo";

export function PieceInspector({ piece: selectedPiece, enemy: inspectedEnemy, abilities, activeAbility, canUseAbility, inputEnabled, onAbility, boss, initiativeRemaining }: {
  piece: Piece; enemy: boolean; abilities: Ability[]; activeAbility: Ability | null; canUseAbility: boolean;
  inputEnabled: boolean; onAbility: (ability: Ability) => void; boss: BossState | null; initiativeRemaining: number;
}) {
  const selectedMovement = movementInfo(selectedPiece);
  return (
    <section className={`selected-piece-info ${inspectedEnemy ? "enemy-piece-info" : ""}`} aria-label={`${inspectedEnemy ? "적" : "아군"} 기물 정보`}>
      <PieceImage
        pieceKey={selectedPiece.key}
        owner={selectedPiece.owner}
      />
      <div>
        <strong>
          {selectedPiece.name} {selectedPiece.royal && <span className="royal-info">★ 로얄</span>}
        </strong>
        {selectedMovement && <p><b>행마:</b> {selectedMovement.movement}</p>}
        {selectedMovement?.special && <p>{selectedMovement.special}</p>}
        {selectedPiece.key === "tribal_dictator" && boss && <small className="effect-note">
          HP {boss.hp} · 샷건 {boss.ammo}/{boss.max_ammo}
          {boss.hook_name && ` · 갈고리 대상: ${boss.hook_name} (다음 적 턴에 끌어옴)`}
        </small>}
        {selectedPiece.key === "tribal_dictator" && boss?.learned.length ? <small className="effect-note">습득 행마: {boss.learned.join(", ")}</small> : null}
        {selectedPiece.key === "tribal_dictator" && boss?.learned_abilities.map((ability) => <small className="effect-note" key={ability.name}><b>제왕학 · {ability.name}</b> — {ability.description}</small>)}
        {canUseAbility && !inspectedEnemy && abilities.map((ability, index) => <button key={index} className={`ability-button ${activeAbility === ability ? "selected" : ""}`}
          title={ability.description} onClick={() => onAbility(ability)}
          disabled={!inputEnabled || ability.targets.length === 0}>{ability.name} <span>{ability.targets.length ? "대상 선택 →" : "대상 없음"}</span></button>)}
        <PieceStatuses statuses={selectedPiece.statuses} compact />
        {initiativeRemaining > 0 && <small className="effect-note">선공권 보호 · 상대의 포획 제한 · 남은 상대 턴 {initiativeRemaining}회</small>}
        {selectedPiece.temporary_upgrade && <small className="effect-note">일시 강화 · 현재 전투에서만 유지</small>}
        {selectedPiece.ward && <small className="effect-note">가호 · 다음 포획 시 공격 무효</small>}
        {selectedPiece.self_defense && <small className="effect-note">호신용품 · 직접 포획되면 포획한 적도 사망</small>}
        {selectedPiece.absorbed_moves.length > 0 && <small className="effect-note">흡수한 행마 · {selectedPiece.absorbed_moves.join(", ")}</small>}
        {selectedPiece.limiter && <small className="effect-note danger">리미터 해제 · 남은 이동 {selectedPiece.limiter.moves_left}회 · 0회가 되면 사망하고 턴 종료</small>}
      </div>
    </section>
  );
}

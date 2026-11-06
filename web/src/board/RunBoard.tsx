import type { RefObject } from "react";
import type { Snapshot } from "../types";
import type { GameInput } from "../game/useGameInput";
import type { BoardMove } from "../game/useGamePresentation";
import { PieceImage } from "../PieceImage";
import { PromotionPicker } from "../PromotionPicker";
import { ChessBoard, BoardSquare } from "./ChessBoard";
import { BoardPiece } from "./BoardPiece";
import { TargetPreview } from "./TargetPreview";
import { useBoardAnnotations } from "./useBoardAnnotations";

export function RunBoard({ state, input, rewardUpgradeOpen, lastEnemyMove, boardRevision, boardRef }: {
  state: Snapshot; input: GameInput; rewardUpgradeOpen: boolean; lastEnemyMove: BoardMove | null;
  boardRevision: number; boardRef: RefObject<HTMLDivElement | null>;
}) {
  const { selected, enemyInspection, activeAbility, abilityTargets, hoveredAbilityTarget, restChoice, promotion, targetCard, cardTargets, cardUpgrades } = input;
  const turn = state.turn === 0;
  const restView = state.phase === "rest" || rewardUpgradeOpen;
  const boardPieces = rewardUpgradeOpen ? state.next_battle_pieces : state.pieces;
  const restPiece = state.rest_options.find((piece) => piece.id === restChoice?.id);
  const previewKey = state.upgrade_paths.find((path) => path.from === restPiece?.kind && path.to === restChoice?.target)?.to_key;
  const inspectedEnemy = state.phase === "combat" ? state.pieces.find((piece) => piece.id === enemyInspection?.piece_id && piece.cell === enemyInspection?.cell) : undefined;
  const bossPiece = state.boss ? state.pieces.find((piece) => piece.key === "tribal_dictator") : undefined;
  const bossRisk = new Set(state.boss?.plan?.area ?? []);
  const bossTargets = new Set(state.boss?.plan?.targets.map((target) => target.cell) ?? []);
  const enemyCaptures = new Set(enemyInspection?.moves.flatMap((move) => move.captures) ?? []);
  const enemyMoves = new Set(enemyInspection?.moves.map((move) => move.to) ?? []);
  const enemyRoutes = [...new Map((enemyInspection?.moves ?? []).filter((move) => move.route).map((move) => [move.route!.join(","), move.route!])).values()];
  const annotations = useBoardAnnotations(state.phase === "combat", input.policy.board, boardRevision);
  const { boardAnnotations, annotationPreview } = annotations;
  return (
    <div className="board-frame" ref={boardRef}>
      <div className="rank-labels">
        {"87654321".split("").map((r) => (
          <span key={r}>{r}</span>
        ))}
      </div>
      <ChessBoard role="group" aria-label="체스 보드" {...annotations.handlers}
        renderSquare={(i) => {
          const piece = boardPieces.find((p) => p.cell === i);
          const legal =
            !activeAbility &&
            selected !== null &&
            state.moves.some(
              (m) => m.from === selected && m.to === i,
            );
          const enemyCapture = !!inspectedEnemy && enemyCaptures.has(i);
          const enemyMove = !!inspectedEnemy && !piece && enemyMoves.has(i);
          const abilityTarget = activeAbility?.targets.find((target) => target.cell === i);
          const preview = activeAbility?.targets.find((target) => target.cell === hoveredAbilityTarget) ?? (activeAbility?.targets.length === 1 ? activeAbility.targets[0] : undefined);
          const fallout = state.fallout.find((effect) => effect.cell === i);
          const pathIndex = preview?.paths.findIndex((path) => path.includes(i)) ?? -1;
          const targetId = piece?.id ?? 10000 + i;
          const effectTarget = state.effect_choice?.targets.includes(targetId);
          const target = targetCard?.targets.includes(targetId) || !!abilityTarget || effectTarget;
          const staged = cardTargets.includes(targetId) || abilityTargets.includes(i) || (piece && cardUpgrades.some((u) => u.id === piece.id));
          const square = `${"abcdefgh"[i % 8]}${8 - Math.floor(i / 8)}`;
          return (
            <BoardSquare cell={i}
              key={i}
              data-square={square}
              data-legal={legal || undefined}
              data-enemy-move={enemyMove || undefined}
              data-enemy-capture={enemyCapture || undefined}
              data-last-enemy-move={state.phase === "combat" && (lastEnemyMove?.from === i || lastEnemyMove?.to === i) || undefined}
              className={`${state.phase === "combat" && (lastEnemyMove?.from === i || lastEnemyMove?.to === i) ? "last-enemy-move" : ""} ${(restView ? restChoice?.cell : selected) === i ? "selected" : ""} ${inspectedEnemy?.cell === i ? "enemy-selected" : ""} ${restView && restChoice?.cell === i ? "rest-selected" : ""} ${legal ? "legal" : ""} ${target ? "targetable" : ""} ${staged ? "card-staged" : ""} ${fallout ? "fallout-cell" : ""} ${preview?.blast.includes(i) ? "blast-preview" : ""} ${pathIndex >= 0 ? `pierce-path path-${pathIndex}` : ""} ${bossRisk.has(i) ? "boss-risk" : ""} ${bossTargets.has(i) ? "boss-target" : ""} ${state.boss?.hook_target === i ? "boss-hook-target" : ""}`}
              title={abilityTarget ? `${activeAbility?.name}${abilityTarget.chance !== null ? ` · 명중률 ${abilityTarget.chance}%` : ""}` : fallout ? `방사성 낙진 · 남은 ${fallout.remaining}라운드 · 턴 종료 시 제거` : piece?.statuses?.join(" · ") || (piece?.self_defense ? "호신용품: 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다." : undefined)}
              onMouseEnter={() => { if (abilityTarget) input.hoverAbility(i); }}
              onMouseLeave={() => { if (hoveredAbilityTarget === i) input.hoverAbility(null); }}
              onClick={() => input.clickSquare(i)}
              disabled={
                !input.policy.board || (!restView &&
                (state.phase !== "combat" || !!promotion || (!!state.effect_choice && !effectTarget) || (!state.effect_choice && !turn && piece?.owner !== 1 && !(!piece && inspectedEnemy))))
              }
              aria-label={`${square}${piece ? ` ${piece.owner === 0 ? "백" : piece.owner === 1 ? "흑" : "중립"} ${piece.name}${piece.royal ? " 로얄" : ""}${piece.dice_roll ? ` 주사위 ${piece.dice_roll}` : ""}${piece.ward ? " 가호" : ""}${piece.self_defense ? " 호신용품 보호 대상" : ""}${piece.limiter ? ` 리미터 해제 남은 이동 ${piece.limiter.moves_left}회 0회가 되면 사망하고 턴 종료` : ""}` : " 빈 칸"}${fallout ? ` 방사성 낙진 ${fallout.remaining}라운드` : ""}${bossRisk.has(i) ? " 보스 예고 위험 범위" : ""}${state.boss?.hook_target === i ? " 갈고리 반격 대상" : ""}${legal ? " 이동 가능" : ""}${enemyMove ? " 적 이동 가능" : ""}${enemyCapture ? " 적 포획 가능" : ""}${abilityTargets.includes(i) ? " 능력 선택됨" : cardTargets.includes(targetId) ? " 카드 선택됨" : ""}`}
              aria-pressed={abilityTargets.includes(i) || (restView
                ? restChoice?.cell
                : inspectedEnemy?.cell ?? selected) === i}
            >
              <TargetPreview legal={legal} occupied={!!piece} enemyMove={enemyMove} enemyCapture={enemyCapture}
                bossTarget={bossTargets.has(i)} hookTarget={state.boss?.hook_target === i}
                abilityTarget={abilityTarget} fallout={fallout} staged={!!staged}
                selectionNumber={abilityTargets.includes(i) ? abilityTargets.indexOf(i) + 1 : cardTargets.indexOf(targetId) + 1 || "✓"} />
            </BoardSquare>
          );
        }}
        paths={<>
          {inspectedEnemy && enemyRoutes.length > 0 && <svg className="enemy-routes" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
          {enemyRoutes.map((route) => <polyline key={route.join(",")} points={route.map((cell) => `${cell % 8 + 0.5},${Math.floor(cell / 8) + 0.5}`).join(" ")} />)}
        </svg>}
        {bossPiece && state.boss?.hook_target !== null && state.boss?.hook_target !== undefined && <svg className="boss-hook-line" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
          <line x1={bossPiece.cell % 8 + .5} y1={Math.floor(bossPiece.cell / 8) + .5} x2={state.boss.hook_target % 8 + .5} y2={Math.floor(state.boss.hook_target / 8) + .5} />
          <circle cx={state.boss.hook_target % 8 + .5} cy={Math.floor(state.boss.hook_target / 8) + .5} r=".17" />
        </svg>}
        </>}
        pieces={<div className="piece-layer">
          {boardPieces.map((p) => (
            <BoardPiece
              key={p.id}
              piece={p}
              focused={
                restView && restChoice?.id === p.id
              }
              previewKey={
                restChoice?.id === p.id ? previewKey : undefined
              }
            />
          ))}
          {state.phase === "rest" &&
            restPiece?.dead &&
            restChoice && (
              <span
                className="piece rest-ghost rest-focused"
                style={{
                  left: `${(restChoice.cell % 8) * 12.5}%`,
                  top: `${Math.floor(restChoice.cell / 8) * 12.5}%`,
                }}
              >
                <PieceImage pieceKey={restPiece.key} />
                <small>미리보기</small>
              </span>
            )}
        </div>}
        annotations={state.phase === "combat" && (boardAnnotations.length > 0 || annotationPreview) &&
          <svg className="board-annotations" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
            <defs><marker id="board-annotation-head" markerWidth=".42" markerHeight=".42" refX=".36" refY=".21" viewBox="0 0 .42 .42" orient="auto" markerUnits="userSpaceOnUse"><path d="M 0 0 L .42 .21 L 0 .42 Z" /></marker></defs>
            {[...boardAnnotations, ...(annotationPreview ? [annotationPreview] : [])].map(({ from, to }, index) =>
              from === to
                ? <circle key={`${from}-${to}-${index}`} className={`board-annotation-mark ${index === boardAnnotations.length ? "preview" : ""}`} data-cell={from} cx={from % 8 + .5} cy={Math.floor(from / 8) + .5} r=".38" />
                : <line key={`${from}-${to}-${index}`} className={`board-annotation-arrow ${index === boardAnnotations.length ? "preview" : ""}`} data-from={from} data-to={to}
                    x1={from % 8 + .5} y1={Math.floor(from / 8) + .5} x2={to % 8 + .5} y2={Math.floor(to / 8) + .5} markerEnd="url(#board-annotation-head)" />)}
          </svg>}
      />
      {promotion && (
        <PromotionPicker
          options={promotion.options}
          square={`${"abcdefgh"[promotion.to % 8]}${8 - Math.floor(promotion.to / 8)}`}
          onChoose={input.confirmPromotion}
          onCancel={input.cancelPromotion}
        />
      )}
      <div className="file-labels">
        {"abcdefgh".split("").map((f) => (
          <span key={f}>{f}</span>
        ))}
      </div>
    </div>
  );
}

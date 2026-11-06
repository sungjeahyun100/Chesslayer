import type { ReactNode } from "react";
import type { Snapshot } from "../types";
import type { GameInput } from "../game/useGameInput";
import { BattleHeading } from "../components/BattleHeading";
import { ActionHint } from "../components/ActionHint";
import { CardHand } from "../components/CardHand";
import { PieceInspector } from "../components/PieceInspector";
import { RelicList } from "../components/RelicList";
import { CardUpgradePanel } from "../CardUpgradePanel";

export function BattleScreen({ state, input, board }: { state: Snapshot; input: GameInput; board: ReactNode }) {
  const { enemyInspection, selected, activeAbility, targetCard, focusedUpgrade } = input;
  const inspectedEnemy = state.phase === "combat" ? state.pieces.find((piece) => piece.id === enemyInspection?.piece_id && piece.cell === enemyInspection?.cell) : undefined;
  const selectedPiece = inspectedEnemy || state.pieces.find((piece) => piece.cell === selected);
  const selectedAbilities = state.abilities?.filter((ability) => ability.piece_id === selectedPiece?.id) ?? [];
  const initiativeRemaining = selectedPiece?.owner === 1 ? state.enemy_initiative_turns_left ?? 0 : state.initiative_turns_left ?? 0;
  const latestBattleStart = state.log.map((line, index) => line.includes("전투 시작") ? index : -1).filter((index) => index >= 0).pop() ?? -1;
  const recentBossEvent = state.boss ? state.log.slice(latestBattleStart + 1).reverse().find((line) => /부족의 독재자 HP|찢고 죽여라|제왕학|갈고리 반격|샷건/.test(line)) : undefined;
  return <>
    <div className="battle-column">
      <section className="battle-area">
        <BattleHeading state={state} />
        {board}
        {state.phase === "combat" && state.boss && <section className="boss-intent-panel" aria-label="부족의 독재자 상태와 행동 예고" aria-live="polite">
          <strong key={state.boss.hp} className="boss-hp-value">부족의 독재자 · HP {state.boss.hp} · 샷건 {state.boss.ammo}/{state.boss.max_ammo}</strong>
          <span>독재자 행동 예고: {state.boss.plan?.name ?? "행동 없음"}</span>
          {state.boss.plan?.targets.length ? <span>독재자 현재 대상: {state.boss.plan.targets.map((target) => `${target.name} ${"abcdefgh"[target.cell % 8]}${8 - Math.floor(target.cell / 8)} (${target.chance}%)`).join(" · ")}</span> : null}
          <span>갈고리 반격: {state.boss.hook_name ? `${state.boss.hook_name} 지정 · 다음 적 턴에 끌어옴` : "피해를 준 공격자를 다음 턴에 끌어옴"}</span>
          <small>제왕학 · 주변 8칸 사망 시 행마 습득 / 찢고 죽여라 · 주변 사망마다 HP +1</small>
          {state.boss.learned.length > 0 && <small>습득한 행마: {state.boss.learned.join(", ")}</small>}
          {recentBossEvent && <small key={state.log.length} className="boss-event-flash">최근 효과: {recentBossEvent}</small>}
        </section>}

        <ActionHint state={state} input={input} />
      </section>
      {state.phase === "combat" && <CardHand state={state} selectedCardId={targetCard?.id} selectedCount={input.stagedCount}
        onChoose={input.chooseCard} disabled={!input.policy.act} />}
    </div>
    <aside className="sidebar">
      {targetCard?.definition.selection_mode === "upgrade" && focusedUpgrade && <CardUpgradePanel
        key={focusedUpgrade.id} card={targetCard} piece={focusedUpgrade} paths={state.upgrade_paths}
        onConfirm={input.confirmCardUpgrade} onCancel={input.closeCardUpgrade} />}
      {selectedPiece && <PieceInspector piece={selectedPiece} enemy={!!inspectedEnemy} abilities={selectedAbilities}
        activeAbility={activeAbility} canUseAbility={state.turn === 0} inputEnabled={input.policy.act} onAbility={input.chooseAbility}
        boss={state.boss} initiativeRemaining={initiativeRemaining} />}
      <div className="combat-resource" aria-label={state.ammo !== null ? `탄약 ${state.ammo}` : `마나 ${state.mana}`}>
        <span className="mana-orb">{state.ammo ?? state.mana}</span>
        <span>{state.ammo !== null ? `탄약 / ${state.ammo_capacity}` : "마나"}</span>
      </div>
      <RelicList relics={state.relics} />
    </aside>
  </>;
}

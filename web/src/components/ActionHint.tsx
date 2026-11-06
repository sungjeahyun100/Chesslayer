import type { Snapshot } from "../types";
import type { GameInput } from "../game/useGameInput";

function cardSelectionHint(kind: string): string {
  const hints: Record<string, string> = {
    body_slam: "아군과 적을 같은 수만큼 선택하세요. 로얄은 대상이 아닙니다.",
    masquerade: "위치를 교환할 아군 비로얄 2기를 선택하세요.",
    intermission_waltz: "서로 인접한 아군 비로얄 2기를 선택하세요.",
    clear_tablecloth: "시체를 먼저, 이동할 인접한 빈 칸을 다음으로 선택하세요.",
    delivered_supper: "시체를 먹을 아군과 2칸 이내의 시체를 선택하세요.",
    last_supper: "소비할 시체 최대 3개와 민첩을 받을 아군을 선택하세요.",
    frontline: "재배치할 아군과 목적지 빈 칸을 차례로 선택하세요.",
    infiltration: "이동할 아군과 목적지 빈 칸을 선택하세요.",
    extraction: "구출할 아군과 목적지 빈 칸을 선택하세요.",
    emergency_conscription: "사망한 폰을 아래에서 고르고 아군 진영의 빈칸을 선택하세요.",
    assault_order: "전방 빈칸으로 이동할 아군 1~3기를 순서대로 선택하세요.",
    rapid_reload: "사수와 사격할 적을 선택하세요.",
    focus_fire: "사수 두 기와 같은 적 하나를 선택하세요.",
  };
  return hints[kind] ?? "대상을 선택한 뒤 확인을 누르세요. 다시 누르면 선택이 해제됩니다.";
}

export function ActionHint({ state, input }: { state: Snapshot; input: GameInput }) {
  const { targetCard, activeAbility, abilityTargets, cardTargets, cardUpgrades, stagedCount, stagedCost, send, confirmCard, cancelCard } = input;
  return <>
    {state.phase === "combat" && (state.effect_choice || targetCard || activeAbility) && (
      <div
        className={`action-hint ${targetCard ? "target-hint" : ""}`}
        aria-live="polite"
      >
        {state.effect_choice ? <><strong>{state.effect_choice.name}</strong><span>표시된 대상을 선택하세요.</span>{state.effect_choice.can_finish && <button className="primary" onClick={() => send({ type: "resolve_effect", target: 0 })}>배치 완료</button>}</> : targetCard ? (
          <>
            <strong>{targetCard.definition.name}</strong>
            {targetCard.definition.kind === "limiter_release" && <span>세 번째 이동 직후 대상 사망 · 턴 종료 · 다음 전투 시작 시 살아 있다면 즉사</span>}
            <span>선택 {stagedCount}개 · 비용 {stagedCost} / {state.ammo !== null ? `탄약 ${state.ammo}` : `마나 ${state.mana}`}</span>
            {targetCard.definition.selection_mode === "multiple" && <span className="card-target-instruction">{cardSelectionHint(targetCard.definition.kind)}</span>}
            {targetCard.definition.kind === "emergency_conscription" && <span className="staged-upgrades">{state.rest_options.filter((piece) => piece.dead && targetCard.targets.includes(piece.id)).map((piece) => <button key={piece.id} onClick={() => input.chooseDeadPawn(piece.id)}>{cardTargets.includes(piece.id) ? "✓ " : ""}{piece.name} #{piece.id}</button>)}</span>}
            {targetCard.definition.selection_mode === "upgrade" && cardUpgrades.length > 0 && <span className="staged-upgrades">{cardUpgrades.map((choice) => <button key={choice.id} onClick={() => input.removeCardUpgrade(choice.id)}>{state.pieces.find((p) => p.id === choice.id)?.name} ×</button>)}</span>}
            {stagedCount > 0 && (["multiple", "upgrade"].includes(targetCard.definition.selection_mode) || ["limiter_release", "sacred_domain"].includes(targetCard.definition.kind)) ? <button className="primary" onClick={confirmCard} disabled={stagedCost > state.mana}>확인</button> : null}
            <button
              className="text-button"
              onClick={cancelCard}
            >
              취소 Esc
            </button>
          </>
        ) : activeAbility ? <><strong>{activeAbility.name}</strong><span>{activeAbility.kind === "king_reposition" ? `적 기물 ${abilityTargets.length}개 선택 · ${state.pieces.find((piece) => piece.id === activeAbility.piece_id)?.key === "tempest_king" ? "2개" : "2~6개"} 필요` : "표시된 대상 칸을 선택하세요."}</span>{activeAbility.kind === "king_reposition" && <button className="primary" disabled={abilityTargets.length < 2 || (state.pieces.find((piece) => piece.id === activeAbility.piece_id)?.key === "tempest_king" && abilityTargets.length !== 2)} onClick={input.confirmAbility}>확정</button>}<button className="text-button" onClick={input.cancelAbility}>취소 Esc</button></> : null}
      </div>
    )}
  </>;
}

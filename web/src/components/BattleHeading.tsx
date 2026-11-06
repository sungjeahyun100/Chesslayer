import type { Snapshot } from "../types";

export function BattleHeading({ state, rewardUpgradeOpen = false }: { state: Snapshot; rewardUpgradeOpen?: boolean }) {
  const turn = state.turn === 0;
  const restView = state.phase === "rest" || rewardUpgradeOpen;
  return (
    <div className="battle-heading">
      <div>
        <p className="eyebrow">
          EXPEDITION / {state.character}
        </p>
        <h1>{state.encounter}</h1>
      </div>
      <div
        className={`turn-pill ${turn ? "your-turn" : ""}`}
        data-testid="turn"
      >
        {state.phase === "combat" ? (
          <>
            <i />
            {turn ? "나의 턴" : "적의 턴"}
            <small>{state.actions_left}회 행동</small>
          </>
        ) : restView ? (
          rewardUpgradeOpen ? "전투 보상 · 기물 강화" : "쉼터 · 행동 1회"
        ) : (
          "진형 계승"
        )}
      </div>
    </div>
  );
}

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
        {state.phase === "combat" && state.boss && <div className="boss-status" role="status" aria-label="부족의 독재자 상태와 행동 예고">
          <span>샷건 {state.boss.ammo}/{state.boss.max_ammo}</span>
          <small>다음 행동: {state.boss.plans?.map((plan) => plan.name).join(" → ") || state.boss.plan?.name || "행동 없음"}</small>
        </div>}
      </div>
      <div className="battle-heading-indicators">
        {state.phase === "combat" && state.boss && <div className="boss-health" role="status" aria-label="부족의 독재자 체력" aria-atomic="true">
          <span className="boss-health-icon" aria-hidden="true">♥</span>
          <div>
            <small>보스 체력</small>
            <strong key={state.boss.hp} className="boss-hp-value">HP {state.boss.hp}</strong>
          </div>
        </div>}
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
    </div>
  );
}

import type { RefObject } from "react";
import type { Command, Snapshot } from "../types";
import { LocationPanel } from "../LocationPanel";
import { PieceImage } from "../PieceImage";
import { RelicIcon } from "../ContentArt";
import { formatEndlessRecord, type EndlessRecord } from "../endless";
import { rarityNames } from "../rarity";

export function OutcomeScreen({ state, send, onRestart, outcomeRef, endlessBest, endlessRunBest }: {
  state: Snapshot; send: (command: Command) => void; onRestart: () => void; outcomeRef: RefObject<HTMLElement | null>;
  endlessBest: EndlessRecord; endlessRunBest: EndlessRecord;
}) {
  const reachedStage = (state.floor - 1) * 15 + (state.map.nodes.find(n => n.current)?.row ?? 0) + 1;
  const newRecord = reachedStage > (endlessRunBest.reachedStage ?? endlessRunBest.clearedStages);
  return (
    <section className="outcome" ref={outcomeRef} aria-live="polite">
      <LocationPanel state={state} send={send} />
      {state.phase === "treasure" && (
        <>
          <p className="eyebrow">A GIFT ALONG THE WAY</p>
          <h2>남겨진 보물</h2>
          {state.node_reward && <div className="treasure-relic" data-rarity={state.node_reward.rarity ?? "common"}><RelicIcon id={state.node_reward.id} name={state.node_reward.name} /></div>}
          <h3>{state.node_reward?.name}{state.node_reward && <small className="relic-rarity" data-rarity={state.node_reward.rarity ?? "common"}>{rarityNames[state.node_reward.rarity ?? "common"]}</small>}</h3>
          <p>{state.node_reward?.description}</p>
          <div className="treasure-actions">
            <button className="primary large" onClick={() => send({ type: "claim_treasure" })}>
              보물 받고 맵으로 →
            </button>
            <button className="text-button" onClick={() => send({ type: "skip_treasure" })}>
              유물을 받지 않고 떠나기 →
            </button>
          </div>
        </>
      )}
      {(state.phase === "victory" || state.phase === "defeat") && (
        <>
          <span className="end-symbol">
            <PieceImage
              pieceKey="king"
              owner={state.phase === "victory" ? 0 : 1}
            />
          </span>
          <p className="eyebrow">
            EXPEDITION{" "}
            {state.phase === "victory" ? "COMPLETE" : "ENDED"}
          </p>
          <h2>
            {state.phase === "victory"
              ? state.final_battle ? "마지막 왕관까지." : `${state.floor}층 수복 완료`
              : state.relics.some(r => r.id === "omnipotence") ? "적 로얄이 결승선에 도착했습니다." : "왕이 쓰러졌습니다."}
          </h2>
          <p>
            {state.phase === "victory"
              ? state.final_battle ? "부족의 독재자를 쓰러뜨리고 모든 영토를 수복했습니다." : `${state.encounter} 보스를 쓰러뜨렸습니다. 해금된 부대와 새로운 원정을 시작하세요.`
              : "진형은 여기에서 멈추지만, 다음 원정의 한 수는 달라질 수 있습니다."}
          </p>
          {state.mode === "endless" && <div className="endless-result">
            {newRecord && <strong className="eyebrow">신기록!</strong>}
            <p><strong>이번 무한 원정: {formatEndlessRecord({ clearedStages: state.cleared_stages, highestFloor: state.floor, reachedStage })}</strong></p>
            <p>사용 부대 {state.character} · 보스 {state.bosses_defeated ?? 0}회 격파 · 최종 덱 {state.deck_count}장 · 유물 {state.relics.length}개</p>
            {endlessBest.achievedAt && <p>기록 부대 {endlessBest.squad} · {new Date(endlessBest.achievedAt).toLocaleString()}</p>}
            <p>원정 시작 전 최고기록: {formatEndlessRecord(endlessRunBest)}<br />최고기록: {formatEndlessRecord(endlessBest)}</p>
          </div>}
          <button
            className="primary large"
            onClick={onRestart}
          >
            새 원정 시작 →
          </button>
        </>
      )}
    </section>
  );
}

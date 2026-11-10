import type { RefObject } from "react";
import type { Command, Snapshot } from "../types";
import { LocationPanel } from "../LocationPanel";
import { PieceImage } from "../PieceImage";
import { RelicIcon } from "../ContentArt";
import { formatEndlessRecord, type EndlessRecord } from "../endless";

export function OutcomeScreen({ state, send, onRestart, outcomeRef, endlessBest, endlessRunBest }: {
  state: Snapshot; send: (command: Command) => void; onRestart: () => void; outcomeRef: RefObject<HTMLElement | null>;
  endlessBest: EndlessRecord; endlessRunBest: EndlessRecord;
}) {
  const newRecord = state.cleared_stages > 0 && (state.cleared_stages > endlessRunBest.clearedStages || state.floor > endlessRunBest.highestFloor);
  return (
    <section className="outcome" ref={outcomeRef} aria-live="polite">
      <LocationPanel state={state} send={send} />
      {state.phase === "treasure" && (
        <>
          <p className="eyebrow">A GIFT ALONG THE WAY</p>
          <h2>남겨진 보물</h2>
          {state.node_reward && <div className="treasure-relic"><RelicIcon id={state.node_reward.id} name={state.node_reward.name} /></div>}
          <h3>{state.node_reward?.name}</h3>
          <p>{state.node_reward?.description}</p>
          <button
            className="primary large"
            onClick={() => send({ type: "claim_treasure" })}
          >
            보물 받고 맵으로 →
          </button>
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
              : "왕이 쓰러졌습니다."}
          </h2>
          <p>
            {state.phase === "victory"
              ? state.final_battle ? "부족의 독재자를 쓰러뜨리고 모든 영토를 수복했습니다." : `${state.encounter} 보스를 쓰러뜨렸습니다. 해금된 부대와 새로운 원정을 시작하세요.`
              : "진형은 여기에서 멈추지만, 다음 원정의 한 수는 달라질 수 있습니다."}
          </p>
          {state.mode === "endless" && <div className="endless-result">
            {newRecord && <strong className="eyebrow">신기록!</strong>}
            <p><strong>이번 무한 원정: {formatEndlessRecord({ clearedStages: state.cleared_stages, highestFloor: state.floor })}</strong></p>
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

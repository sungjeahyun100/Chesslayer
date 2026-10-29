import { useEffect, useRef, type KeyboardEvent } from "react";
import { createPortal } from "react-dom";
import { CardFace } from "./CardFace";
import { RelicIcon } from "./ContentArt";
import { PieceImage } from "./PieceImage";
import type { Command, Snapshot } from "./types";

export function BattleRewards({ state, send, revisit, onUpgrade }: { state: Snapshot; send: (command: Command) => void; revisit: boolean; onUpgrade: () => void }) {
  const choiceMade = state.reward_choice !== null;
  const special = state.node_kind === "elite" || state.node_kind === "boss";
  const panel = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const root = document.getElementById("root")!;
    const previousFocus = document.activeElement as HTMLElement | null;
    const previousOverflow = document.body.style.overflow;
    root.inert = true;
    document.body.style.overflow = "hidden";
    panel.current?.focus();
    return () => {
      root.inert = false;
      document.body.style.overflow = previousOverflow;
      if (previousFocus?.isConnected) previousFocus.focus();
    };
  }, []);
  function keepFocus(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key !== "Tab") return;
    const buttons = Array.from(panel.current?.querySelectorAll<HTMLButtonElement>("button:not(:disabled)") ?? []);
    const first = buttons[0];
    const last = buttons.at(-1);
    if (!first || !last) return;
    if (event.shiftKey && (document.activeElement === first || document.activeElement === panel.current)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
  return createPortal(
    <div className="reward-overlay">
    <div className="battle-rewards" ref={panel} role="dialog" aria-modal="true" aria-labelledby="battle-rewards-title" tabIndex={-1} onKeyDown={keepFocus}>
      <p className="eyebrow">ROYAL DEFEATED</p>
      <h2 id="battle-rewards-title">전투 승리</h2>
      <h3>전투 보상</h3>
      <p className="reward-intro">보상 하나를 선택하세요. {state.node_kind === "boss" ? "보상을 마치면 원정이 종료됩니다. 해금된 부대와 다음 원정에 나설 수 있습니다." : "마지막 클리어 지점에서 남겨둔 보상을 다시 확인할 수 있습니다."}</p>
      <div className="reward-cards">
        {state.rewards.map((reward, index) => {
          const claimed = state.reward_choice === index;
          return (
            <button
              className={`playing-card ${reward.type === "relic" ? "relic-reward" : ""} ${claimed ? "is-claimed" : ""}`}
              key={index}
              data-reward-type={reward.type}
              disabled={choiceMade}
              onClick={() => send({ type: "claim_reward", index })}
              title={reward.definition.description}
              aria-label={`${reward.type === "card" ? "카드" : "유물"} 보상 ${reward.definition.name} · ${reward.definition.description}${claimed ? " · 획득 완료" : ""}`}
            >
              {reward.type === "card" ? (
                <CardFace card={reward.definition} />
              ) : (
                <>
                  <div className="card-top"><span className="relic-type">유물</span><span>영구 효과</span></div>
                  <div className="card-symbol"><RelicIcon id={reward.definition.id} name={reward.definition.name} /></div>
                  <h3>{reward.definition.name}</h3>
                  <p>{reward.definition.description}</p>
                </>
              )}
              <span className="reward-select">{claimed ? "획득 완료 ✓" : choiceMade ? "다른 보상 선택됨" : reward.type === "card" ? "덱에 추가 +" : "유물 획득 +"}</span>
            </button>
          );
        })}
      </div>
      {special && <div className="elite-reward-group">
        <button className="primary large" onClick={onUpgrade} disabled={state.upgrade_claimed || !state.rest_options.some((piece) => piece.can_upgrade)}>기물 강화</button>
        {state.upgrade_claimed && <p role="status">강화 보상을 수령했습니다.</p>}
      </div>}
      {!!state.reinforcement_options?.length && <section className="postbattle-choice"><h3>충원</h3><p>전사한 기물 1기를 초기 상태로 되살립니다.</p>
        <div className="postbattle-options">{state.reinforcement_options.map((piece) => <button key={piece.id} onClick={() => send({ type: "reinforce", id: piece.id })}>
          <PieceImage pieceKey={piece.key} name={piece.name} /><span>{piece.name}</span>
        </button>)}</div>
      </section>}
      {!!state.hunger_protection_options?.length && <section className="postbattle-choice"><h3>마지막 한 접시</h3><p>이번 전투 후 허기 악화를 막을 기물 1기를 선택하세요.</p>
        <div className="postbattle-options">{state.hunger_protection_options.map((piece) => <button key={piece.id} onClick={() => send({ type: "protect_hunger", id: piece.id })}>
          <PieceImage pieceKey={piece.key} name={piece.name} /><span>{piece.name}</span><small>{piece.statuses?.filter((status) => /공복|굶주림|기아/.test(status)).join(" ")}</small>
        </button>)}</div>
      </section>}
      <button className="primary large reward-continue" onClick={() => send({ type: "continue_reward" })}>
        {state.node_kind === "boss" ? "원정 마치기 →" : revisit ? "맵으로 돌아가기 →" : "다음으로 진행 →"}
      </button>
    </div>
    </div>,
    document.body,
  );
}

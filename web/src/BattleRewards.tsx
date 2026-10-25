import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { createPortal } from "react-dom";
import { CardFace } from "./CardFace";
import { RelicIcon } from "./ContentArt";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import type { Command, Snapshot } from "./types";

export function BattleRewards({ state, send, revisit, history }: { state: Snapshot; send: (command: Command) => void; revisit: boolean; history: { id: number; target: number }[] }) {
  const choiceMade = state.reward_choice !== null;
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [selectedKind, setSelectedKind] = useState<number | null>(null);
  const selectedPiece = state.rest_options.find((piece) => piece.id === selectedId);
  const selectedPaths = state.upgrade_paths.filter((path) => path.to === selectedKind);
  const selectedPath = selectedPaths[0];
  const selectedName = selectedKind === selectedPiece?.kind ? selectedPiece.name
    : selectedPath?.to_name ?? state.upgrade_paths.find((path) => path.from === selectedKind)?.from_name;
  const selectedDescription = selectedKind === selectedPiece?.kind ? selectedPiece.description
    : selectedPath?.effect ?? "이 계열의 기본 기물입니다.";
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
      <p className="reward-intro">아래 세 보상 중 하나를 선택할 수 있습니다. 보상을 받지 않고 다음으로 진행해도 마지막 클리어 노드에서 다시 확인할 수 있습니다.</p>
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
      {special && <section className="elite-reward-group upgrade-reward-group" aria-label="기물 강화 보상">
        <h3>{state.node_kind === "elite" ? "엘리트" : "보스"} 기물 강화 · 1회</h3>
        <p>{state.upgrade_claimed ? "강화 보상을 수령했습니다." : "강화할 아군 기물을 고른 뒤 테크트리에서 강화 노드를 확정하세요. 이 보상은 나중에 다시 받을 수 있습니다."}</p>
        <div className="upgrade-reward-layout">
          <div className="upgrade-reward-aside"><div className="upgrade-reward-pieces" role="group" aria-label="강화 대상 기물">
            {state.rest_options.map((piece) => <button key={piece.id} className={selectedId === piece.id ? "is-selected" : ""}
              disabled={!piece.can_upgrade || state.upgrade_claimed} aria-pressed={selectedId === piece.id}
              onClick={() => { setSelectedId(piece.id); setSelectedKind(null); }} title={piece.can_upgrade ? `${piece.name} · ${piece.square}` : `${piece.name} · 강화 불가`}>
              <PieceImage pieceKey={piece.key} name={piece.name} />
              <span><strong>{piece.name}</strong><small>{piece.square} · {piece.dead ? "전투 불능" : piece.can_upgrade ? "강화 가능" : "강화 불가"}</small></span>
            </button>)}
          </div>
          {selectedPiece && selectedKind !== null && <div className="upgrade-reward-info" aria-live="polite">
            <strong>{selectedName}</strong><p>{selectedDescription}</p>
            {!!selectedPaths.length && <div className="skill-requirements"><strong>비용 · 선행 조건</strong>
              {selectedPaths.map((path) => <div className="skill-requirement-path" key={path.from}><p><span>{path.from_name} → {path.to_name}</span>{path.requirement}</p>
                {path.required_pieces.map((required) => <span className={`skill-owned ${required.owned ? "owned" : "missing"}`} key={required.kind}>{required.name} · {required.owned ? "보유" : "미보유"}</span>)}
              </div>)}
            </div>}
          </div>}
          </div>
          {selectedPiece && <div className="upgrade-reward-tree">
            <UpgradeTree key={selectedPiece.id} piece={selectedPiece} paths={state.upgrade_paths}
              acquired={history.filter((item) => item.id === selectedPiece.id).map((item) => item.target)}
              onSelect={(kind) => setSelectedKind(kind ?? null)} onConfirm={(target) => send({ type: "claim_upgrade", id: selectedPiece.id, target })}
              completed={state.upgrade_claimed} />
          </div>}
          {!selectedPiece && <p className="upgrade-reward-placeholder">기물을 선택하면 강화 테크트리가 이곳에 표시됩니다.</p>}
        </div>
      </section>}
      <button className="primary large reward-continue" onClick={() => send({ type: "continue_reward" })}>
        {revisit ? "맵으로 돌아가기 →" : "다음으로 진행 →"}
      </button>
    </div>
    </div>,
    document.body,
  );
}

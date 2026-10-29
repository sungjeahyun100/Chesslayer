import { useEffect, useRef } from "react";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import type { Command, Snapshot } from "./types";

export interface RestChoice {
  cell: number;
  id: number;
  target?: number;
}
export function RestPanel({ state, choice, onChoose, onCancel, send, history, onContinue, reward = false }: {
  state: Snapshot;
  choice: RestChoice | null;
  onChoose: (choice: RestChoice) => void;
  onCancel: () => void;
  send: (command: Command) => void;
  history: { id: number; target: number }[];
  onContinue: () => void;
  reward?: boolean;
}) {
  const piece = state.rest_options.find((p) => p.id === choice?.id);
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    if (piece && !piece.dead && matchMedia("(max-width: 900px)").matches)
      panel.current?.scrollIntoView({ block: "start", behavior: "smooth" });
  }, [piece?.id, piece?.dead]);
  const candidates = state.rest_options.filter((p) => p.can_revive && p.origin === choice?.cell);

  if (piece && !piece.dead) {
    const square = reward && choice ? `${"abcdefgh"[choice.cell % 8]}${8 - Math.floor(choice.cell / 8)}` : piece.square;
    const preview = state.upgrade_paths.find((p) => p.to === choice?.target);
    const base = state.upgrade_paths.find((p) => p.from === choice?.target);
    const studyKey = preview?.to_key ?? base?.from_key ?? piece.key;
    const studyName = preview?.to_name ?? base?.from_name ?? piece.name;
    const studyDescription = choice?.target !== undefined && choice.target !== piece.kind ? preview?.effect : undefined;
    const acquired = history.filter((c) => c.id === piece.id).map((c) => c.target);
    const incoming = state.upgrade_paths.filter((path) => path.to === choice?.target);
    const direct = incoming.find((path) => path.from === piece.kind);
    const reachable = new Set([piece.kind]);
    for (let i = 0; i < state.upgrade_paths.length; i++) {
      for (const path of state.upgrade_paths) if (reachable.has(path.from)) reachable.add(path.to);
    }
    const reason = choice?.target === undefined ? null
      : choice.target === piece.kind ? "현재 기물에 적용된 능력입니다."
        : acquired.includes(choice.target) ? "이미 획득한 강화입니다."
          : !incoming.length ? "이 계열의 시작점입니다. 이전 종류로 되돌리는 강화는 없습니다."
            : !reachable.has(choice.target) ? `현재 ${piece.name}에서 이 가지로 변경하는 강화 경로는 없습니다.`
              : direct && !direct.unlocked ? `영구 로스터 필요: ${direct.required_pieces.filter((required) => !required.owned).map((required) => required.name).join(", ")}`
                : !direct ? `선행 기물 ${incoming.map((path) => path.from_name).join(" 또는 ")} 상태가 필요합니다.` : null;
    return (
      <section className="rest-panel rest-workshop" ref={panel} aria-label={reward ? "보상 기물 강화" : "쉼터 기물 관리"}>
        <div className="rest-detail-heading">
          <h2>기물 강화</h2><span className="rest-budget">{reward ? "보상 1회" : "행동 1회"}</span>
          <button className="text-button" onClick={onCancel} aria-label="기물 선택 닫기">닫기 ×</button>
        </div>
        <div className="rest-detail rest-workshop-layout">
          <div className="rest-piece-study">
            <div className="rest-portrait"><PieceImage pieceKey={studyKey} name={studyName} />{piece.ward && <span className="portrait-ward">가호 유지</span>}</div>
            <h2>{studyName} <small>· {square}</small></h2>
            {studyDescription && <p className="piece-description">{studyDescription}</p>}
            {!state.upgrade_paths.some((path) => path.from === piece.kind || path.to === piece.kind) && <p className="rest-note">강화 경로 없음</p>}
            <details className="rest-piece-details"><summary>기물 상세</summary><p>{piece.description}</p></details>
            {choice?.target !== undefined && <div className="skill-inspector" aria-live="polite">
              {!!incoming.length && <div className="skill-requirements">
                <strong>선행 조건{incoming.length > 1 ? " (경로별)" : ""}</strong>
                {incoming.map((path) => <div key={path.from} className="skill-requirement-path"><p><span>{path.from_name} → {path.to_name}</span>{path.requirement}</p>
                  {path.required_pieces.map((required) => <span className={`skill-owned ${required.owned ? "owned" : "missing"}`} key={required.kind}>{required.name} · {required.owned ? "보유" : "미보유"}</span>)}
                </div>)}
              </div>}
              {reason && <p className="skill-lock-reason">{reason}</p>}
            </div>}
          </div>
          <UpgradeTree key={piece.id} piece={piece} paths={state.upgrade_paths} acquired={acquired}
            onSelect={(target) => onChoose({ ...choice!, target })}
            onConfirm={(target) => send(reward ? { type: "claim_upgrade", id: piece.id, target } : { type: "rest_upgrade", id: piece.id, target })}
            completed={false} />
        </div>
        <div className="rest-workshop-actions">
          {state.phase === "rest" && piece.disabled && <button className="primary" onClick={() => send({ type: "confess", id: piece.id })}>고해성사 · 전투불능 해제</button>}
          {state.ammo !== null && piece.key === "warehouse" && <button className="primary" onClick={() => send({ type: "warehouse_upgrade", id: piece.id })}>창고 강화 · 현재 탄약 +10%</button>}
          <button className="text-button" onClick={onCancel}>대상 재선택</button>
          <button className="text-button" onClick={() => reward ? onContinue() : send({ type: "rest_skip" })}>
            {reward ? "보상 화면으로 돌아가기 →" : "휴식을 마치고 맵으로 →"}
          </button>
        </div>
      </section>
    );
  }

  return (
    <section className="rest-panel" aria-label={reward ? "보상 기물 강화" : "쉼터 기물 관리"}>
      <div className="section-label">{reward ? "기물 강화" : "쉼터"} <span className="rest-budget">{reward ? "보상 1회" : "행동 1회"}</span></div>
      {!piece && state.phase === "rest" && state.rest_options.some(p => p.disabled) && <div className="confession-list"><p>고해성사 · 행동 1회</p>{state.rest_options.filter(p => p.disabled).map(p => <button key={p.id} onClick={() => send({ type: "confess", id: p.id })}>{p.name} · {p.square}</button>)}</div>}
      {!piece ? (
        <div className="rest-intro">
          <h2>{reward ? "강화할 기물 선택" : "기물 선택"}</h2>
          <p>{reward ? "보드에서 아군 기물을 선택하세요." : "아군 기물 또는 부활할 기물의 초기 칸을 선택하세요."}</p>
        </div>
      ) : (
        <div className="rest-detail" key={piece.id}>
          <div className="rest-detail-heading"><span className="square-chip">{piece.square}</span><span>부활 미리보기</span><button className="text-button" onClick={onCancel} aria-label="기물 선택 닫기">닫기 ×</button></div>
          <div className="rest-portrait revival"><PieceImage pieceKey={piece.key} name={piece.name} />{piece.ward && <span className="portrait-ward">가호 유지</span>}</div>
          <h2>{piece.name}</h2><p className="piece-description">{piece.description}</p>
          {!!piece.upgrades.length && <p className="rest-note">강화 이력 유지 · {piece.upgrades.length}회</p>}
          {candidates.length > 1 && <div className="revive-candidates" role="group" aria-label="부활 후보">{candidates.map((p) => <button key={p.id} aria-pressed={p.id === piece.id} onClick={() => onChoose({ cell: choice!.cell, id: p.id, target: undefined })}><PieceImage pieceKey={p.key} /><span>{p.name}</span></button>)}</div>}
          <p className="rest-question">이 기물을 되살리시겠습니까?</p>
          <p className="rest-note">초기 위치 {piece.square} · 쉼터 행동 1회<br />영구 진화·강화·성장 정보를 유지합니다.</p>
          <div className="rest-confirm"><button className="primary" onClick={() => send({ type: "rest_revive", id: piece.id })}>부활 확정</button><button onClick={onCancel}>취소</button></div>
        </div>
      )}
      <button className="text-button rest-depart" onClick={() => reward ? onContinue() : send({ type: "rest_skip" })}>{reward ? "보상 화면으로 돌아가기 →" : "휴식을 마치고 맵으로 →"}</button>
    </section>
  );
}

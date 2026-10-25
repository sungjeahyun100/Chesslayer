import { useEffect, useRef } from "react";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import type { Command, Snapshot } from "./types";

export interface RestChoice {
  cell: number;
  id: number;
  target?: number;
}
export function RestPanel({ state, choice, onChoose, onCancel, send, history, completed, onContinue, reward = false }: {
  state: Snapshot;
  choice: RestChoice | null;
  onChoose: (choice: RestChoice) => void;
  onCancel: () => void;
  send: (command: Command) => void;
  history: { id: number; target: number }[];
  completed: boolean;
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
    const studyDescription = choice?.target === undefined || choice.target === piece.kind
      ? piece.description : preview?.effect ?? "이 계열의 기본 기물입니다. 연결된 가지에서 강화 방향을 선택하세요.";
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
                : completed ? reward ? "이번 보상 강화를 사용했습니다." : "이번 쉼터의 행동을 사용했습니다. 다음 쉼터에서 다시 살펴보세요."
                  : !direct ? `선행 기물 ${incoming.map((path) => path.from_name).join(" 또는 ")} 상태가 필요합니다.` : null;
    return (
      <section className="rest-panel rest-workshop" ref={panel} aria-label={reward ? "보상 기물 강화" : "쉼터 기물 관리"}>
        <div className="section-label">{reward ? "BATTLE REWARD" : "A MOMENT OF REST"} <span className="rest-budget">{completed ? "강화 완료" : reward ? "보상 1회" : "행동 1회"}</span></div>
        <div className="rest-detail-heading">
          <span className="square-chip">{square}</span><span>기물 육성 · 강화 테크트리</span>
          <button className="text-button" onClick={onCancel} aria-label="기물 선택 닫기">닫기 ×</button>
        </div>
        {completed && <p className="skill-complete" role="status">◆ {piece.name} 강화 완료 · 선택한 경로가 활성화되었습니다.</p>}
        <div className="rest-detail rest-workshop-layout">
          <div className="rest-piece-study">
            <span className="eyebrow">{choice?.target !== undefined && choice.target !== piece.kind ? "살펴보는 기물" : "나의 기물"}</span>
            <div className="rest-portrait"><PieceImage pieceKey={studyKey} name={studyName} />{piece.ward && <span className="portrait-ward">가호 유지</span>}</div>
            <h2>{studyName}</h2>
            <p className="piece-description">{studyDescription}</p>
            {!state.upgrade_paths.some((path) => path.from === piece.kind || path.to === piece.kind) && <p className="rest-note">이 기물에 정의된 강화 경로가 없습니다.</p>}
            {choice?.target !== undefined && <div className="skill-inspector" aria-live="polite">
              {!!incoming.length && <div className="skill-requirements">
                <strong>비용 · 선행 조건{incoming.length > 1 ? " (진입 경로별)" : ""}</strong>
                {incoming.map((path) => <div key={path.from} className="skill-requirement-path"><p><span>{path.from_name} → {path.to_name}</span>{path.requirement}</p>
                  {path.required_pieces.map((required) => <span className={`skill-owned ${required.owned ? "owned" : "missing"}`} key={required.kind}>{required.name} · {required.owned ? "보유" : "미보유"}</span>)}
                </div>)}
              </div>}
              {reason && <p className="skill-lock-reason">{reason}</p>}
            </div>}
            <div className="rest-growth-history"><strong>현재 진형</strong><p>{piece.name} · {square}</p>
              {!!acquired.length && <><strong>획득한 강화</strong><ol>{acquired.map((kind, i) => <li key={i}>{state.upgrade_paths.find((p) => p.to === kind)?.to_name ?? "강화"}</li>)}</ol></>}
            </div>
            <p className="rest-note">연결된 가지를 따라<br />다음 성장 방향을 살펴보세요.<br />탐색 중에는 행동을 쓰지 않습니다.</p>
          </div>
          <UpgradeTree key={piece.id} piece={piece} paths={state.upgrade_paths} acquired={acquired}
            onSelect={(target) => onChoose({ ...choice!, target })}
            onConfirm={(target) => send(reward ? { type: "claim_upgrade", id: piece.id, target } : { type: "rest_upgrade", id: piece.id, target })}
            completed={completed} />
        </div>
        <button className="text-button rest-depart" onClick={() => reward || completed ? onContinue() : send({ type: "rest_skip" })}>
          {reward ? "보상 화면으로 돌아가기 →" : completed ? "맵으로 돌아가기 →" : "휴식을 마치고 맵으로 →"}
        </button>
      </section>
    );
  }

  return (
    <section className="rest-panel" aria-label={reward ? "보상 기물 강화" : "쉼터 기물 관리"}>
      <div className="section-label">{reward ? "BATTLE REWARD" : "A MOMENT OF REST"} <span className="rest-budget">{reward ? "보상 1회" : "행동 1회"}</span></div>
      {!piece ? (
        <div className="rest-intro">
          <h2>{reward ? "기물 강화" : "진형을 돌보는 시간"}</h2>
          <p>{reward ? "보드에서 강화할 아군 기물을 선택하세요." : <>보드에서 아군 기물이나<br />돌아올 기물의 초기 칸을 선택하세요.</>}</p>
          <div className="rest-instruction"><PieceImage pieceKey="tempest_pawn" /><div><strong>기물을 선택하면</strong><p>능력과 강화 경로를 살펴봅니다.</p></div></div>
          {!reward && <div className="rest-instruction"><span className="empty-tile" /><div><strong>초기 빈칸을 선택하면</strong><p>그 칸에서 시작한 사망 기물을 되살립니다.</p></div></div>}
          <p className="rest-note">{reward ? "강화 확정 전에는 보상이 사용되지 않습니다." : <>미리보기는 행동을 사용하지 않습니다.<br />쉼터마다 강화 또는 부활을 한 번 확정할 수 있습니다.</>}</p>
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

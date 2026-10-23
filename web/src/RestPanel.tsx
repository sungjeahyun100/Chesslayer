import { useEffect, useRef } from "react";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import type { Command, Snapshot } from "./types";

export interface RestChoice {
  cell: number;
  id: number;
  target?: number;
}
export function RestPanel({ state, choice, onChoose, onCancel, send, history, completed, onContinue }: {
  state: Snapshot;
  choice: RestChoice | null;
  onChoose: (choice: RestChoice) => void;
  onCancel: () => void;
  send: (command: Command) => void;
  history: Extract<Command, { type: "rest_upgrade" }>[];
  completed: boolean;
  onContinue: () => void;
}) {
  const piece = state.rest_options.find((p) => p.id === choice?.id);
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    if (piece && !piece.dead && matchMedia("(max-width: 900px)").matches)
      panel.current?.scrollIntoView({ block: "start", behavior: "smooth" });
  }, [piece?.id, piece?.dead]);
  const candidates = state.rest_options.filter((p) => p.can_revive && p.origin === choice?.cell);

  if (piece && !piece.dead) {
    const preview = state.upgrade_paths.find((p) => p.to === choice?.target);
    const base = state.upgrade_paths.find((p) => p.from === choice?.target);
    const studyKey = preview?.to_key ?? base?.from_key ?? piece.key;
    const studyName = preview?.to_name ?? base?.from_name ?? piece.name;
    const studyDescription = choice?.target === undefined || choice.target === piece.kind
      ? piece.description : preview?.effect ?? "이 계열의 기본 기물입니다. 연결된 가지에서 강화 방향을 선택하세요.";
    const acquired = history.filter((c) => c.id === piece.id).map((c) => c.target);
    return (
      <section className="rest-panel rest-workshop" ref={panel} aria-label="쉼터 기물 관리">
        <div className="section-label">A MOMENT OF REST <span className="rest-budget">{completed ? "강화 완료" : "행동 1회"}</span></div>
        <div className="rest-detail-heading">
          <span className="square-chip">{piece.square}</span><span>기물 육성 · 강화 테크트리</span>
          <button className="text-button" onClick={onCancel} aria-label="기물 선택 닫기">닫기 ×</button>
        </div>
        {completed && <p className="skill-complete" role="status">◆ {piece.name} 강화 완료 · 선택한 경로가 활성화되었습니다.</p>}
        <div className="rest-detail rest-workshop-layout">
          <div className="rest-piece-study">
            <span className="eyebrow">{choice?.target !== undefined && choice.target !== piece.kind ? "살펴보는 기물" : "나의 기물"}</span>
            <div className="rest-portrait"><PieceImage pieceKey={studyKey} name={studyName} />{piece.ward && <span className="portrait-ward">가호 유지</span>}</div>
            <h2>{studyName}</h2>
            <p className="piece-description">{studyDescription}</p>
            <div className="rest-growth-history"><strong>현재 진형</strong><p>{piece.name} · {piece.square}</p>
              {!!acquired.length && <><strong>쉼터에서 획득한 강화</strong><ol>{acquired.map((kind, i) => <li key={i}>{state.upgrade_paths.find((p) => p.to === kind)?.to_name ?? "강화"}</li>)}</ol></>}
            </div>
            <p className="rest-note">연결된 가지를 따라<br />다음 성장 방향을 살펴보세요.<br />탐색 중에는 행동을 쓰지 않습니다.</p>
          </div>
          <UpgradeTree key={piece.id} piece={piece} paths={state.upgrade_paths} acquired={acquired}
            selected={choice?.target} onSelect={(target) => onChoose({ ...choice!, target })}
            onConfirm={(target) => send({ type: "rest_upgrade", id: piece.id, target })}
            onCancel={onCancel} completed={completed} />
        </div>
        <button className="text-button rest-depart" onClick={() => completed ? onContinue() : send({ type: "rest_skip" })}>
          {completed ? "맵으로 돌아가기 →" : "휴식을 마치고 맵으로 →"}
        </button>
      </section>
    );
  }

  return (
    <section className="rest-panel" aria-label="쉼터 기물 관리">
      <div className="section-label">A MOMENT OF REST <span className="rest-budget">행동 1회</span></div>
      {!piece ? (
        <div className="rest-intro">
          <h2>진형을 돌보는 시간</h2>
          <p>보드에서 아군 기물이나<br />돌아올 기물의 초기 칸을 선택하세요.</p>
          <div className="rest-instruction"><PieceImage pieceKey="tempest_pawn" /><div><strong>기물을 선택하면</strong><p>능력과 강화 경로를 살펴봅니다.</p></div></div>
          <div className="rest-instruction"><span className="empty-tile" /><div><strong>초기 빈칸을 선택하면</strong><p>그 칸에서 시작한 사망 기물을 되살립니다.</p></div></div>
          <p className="rest-note">미리보기는 행동을 사용하지 않습니다.<br />쉼터마다 강화 또는 부활을 한 번 확정할 수 있습니다.</p>
        </div>
      ) : (
        <div className="rest-detail" key={piece.id}>
          <div className="rest-detail-heading"><span className="square-chip">{piece.square}</span><span>부활 미리보기</span><button className="text-button" onClick={onCancel} aria-label="기물 선택 닫기">닫기 ×</button></div>
          <div className="rest-portrait revival"><PieceImage pieceKey={piece.key} name={piece.name} />{piece.ward && <span className="portrait-ward">가호 유지</span>}</div>
          <h2>{piece.name}</h2><p className="piece-description">{piece.description}</p>
          {!!piece.upgrades.length && <p className="rest-note">강화 이력 유지 · {piece.upgrades.length}회</p>}
          {candidates.length > 1 && <div className="revive-candidates" role="group" aria-label="부활 후보">{candidates.map((p) => <button key={p.id} aria-pressed={p.id === piece.id} onClick={() => onChoose({ cell: choice!.cell, id: p.id, target: undefined })}><PieceImage pieceKey={p.key} /><span>{p.name}</span></button>)}</div>}
          <p className="rest-question">이 기물을 되살리시겠습니까?</p>
          <p className="rest-note">초기 위치 {piece.square} · 쉼터 행동 1회<br />종류·강화·이동 이력·지속 효과를 유지합니다.</p>
          <div className="rest-confirm"><button className="primary" onClick={() => send({ type: "rest_revive", id: piece.id })}>부활 확정</button><button onClick={onCancel}>취소</button></div>
        </div>
      )}
      <button className="text-button rest-depart" onClick={() => send({ type: "rest_skip" })}>휴식을 마치고 맵으로 →</button>
    </section>
  );
}

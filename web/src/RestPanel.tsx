import { PieceImage } from "./PieceImage";
import type { Command, Snapshot } from "./types";

export interface RestChoice {
  cell: number;
  id: number;
  target?: number;
}
export function RestPanel({
  state,
  choice,
  onChoose,
  onCancel,
  send,
}: {
  state: Snapshot;
  choice: RestChoice | null;
  onChoose: (choice: RestChoice) => void;
  onCancel: () => void;
  send: (command: Command) => void;
}) {
  const piece = state.rest_options.find((p) => p.id === choice?.id);
  const paths = piece
    ? state.upgrade_paths.filter(
        (p) => p.from === piece.kind || p.to === piece.kind,
      )
    : [];
  const path = paths.find(
    (p) => p.from === piece?.kind && p.to === choice?.target,
  );
  const preview = !!path;
  const reviving = !!piece?.dead;
  const candidates = state.rest_options.filter(
    (p) => p.can_revive && p.origin === choice?.cell,
  );
  return (
    <section className="rest-panel" aria-label="쉼터 기물 관리">
      <div className="section-label">
        A MOMENT OF REST <span className="rest-budget">행동 1회</span>
      </div>
      {!piece ? (
        <div className="rest-intro">
          <h2>진형을 돌보는 시간</h2>
          <p>
            보드에서 아군 기물이나
            <br />
            돌아올 기물의 초기 칸을 선택하세요.
          </p>
          <div className="rest-instruction">
            <PieceImage pieceKey="tempest_pawn" />
            <div>
              <strong>기물을 선택하면</strong>
              <p>능력과 강화 경로를 살펴봅니다.</p>
            </div>
          </div>
          <div className="rest-instruction">
            <span className="empty-tile" />
            <div>
              <strong>초기 빈칸을 선택하면</strong>
              <p>그 칸에서 시작한 사망 기물을 되살립니다.</p>
            </div>
          </div>
          <p className="rest-note">
            미리보기는 행동을 사용하지 않습니다.
            <br />
            강화 또는 부활을 확정하면 맵으로 돌아갑니다.
          </p>
        </div>
      ) : (
        <div className="rest-detail" key={`${piece.id}-${preview}`}>
          <div className="rest-detail-heading">
            <span className="square-chip">{piece.square}</span>
            <span>
              {reviving
                ? "부활 미리보기"
                : preview
                  ? "강화 결과 미리보기"
                  : "선택한 기물"}
            </span>
            <button
              className="text-button"
              onClick={onCancel}
              aria-label="기물 선택 닫기"
            >
              닫기 ×
            </button>
          </div>
          <div className={`rest-portrait ${reviving ? "revival" : ""}`}>
            <PieceImage
              pieceKey={preview ? path!.to_key : piece.key}
              name={preview ? path!.to_name : piece.name}
            />
            {piece.ward && <span className="portrait-ward">가호 유지</span>}
          </div>
          <h2>{preview ? path!.to_name : piece.name}</h2>
          <p className="piece-description">
            {preview ? path!.effect : piece.description}
          </p>
          {!!piece.upgrades.length && (
            <p className="rest-note">
              강화 이력 유지 · {piece.upgrades.length}회
            </p>
          )}
          {reviving ? (
            <>
              {candidates.length > 1 && (
                <div
                  className="revive-candidates"
                  role="group"
                  aria-label="부활 후보"
                >
                  {candidates.map((p) => (
                    <button
                      key={p.id}
                      aria-pressed={p.id === piece.id}
                      onClick={() =>
                        onChoose({
                          cell: choice!.cell,
                          id: p.id,
                          target: undefined,
                        })
                      }
                    >
                      <PieceImage pieceKey={p.key} />
                      <span>{p.name}</span>
                    </button>
                  ))}
                </div>
              )}
              <p className="rest-question">이 기물을 되살리시겠습니까?</p>
              <p className="rest-note">
                초기 위치 {piece.square} · 쉼터 행동 1회
                <br />
                종류·강화·이동 이력·지속 효과를 유지합니다.
              </p>
              <div className="rest-confirm">
                <button
                  className="primary"
                  onClick={() => send({ type: "rest_revive", id: piece.id })}
                >
                  부활 확정
                </button>
                <button onClick={onCancel}>취소</button>
              </div>
            </>
          ) : (
            <>
              <h3 className="tree-heading">강화 테크트리</h3>
              {!paths.length && (
                <p className="rest-note no-upgrades">
                  이 기물에 정의된 강화 경로가 없습니다.
                </p>
              )}
              {paths.map((u) => {
                const available = piece.can_upgrade && u.from === piece.kind;
                const completed = u.to === piece.kind;
                return (
                  <button
                    key={`${u.from}-${u.to}`}
                    className={`upgrade-path ${path === u ? "previewing" : ""}`}
                    disabled={!available}
                    aria-pressed={path === u}
                    onClick={() => onChoose({ ...choice!, target: u.to })}
                    aria-label={`${u.from_name} → ${u.to_name} 강화 미리보기`}
                  >
                    <span className="tree-images">
                      <span>
                        <PieceImage pieceKey={u.from_key} />
                        <b>{u.from_name}</b>
                      </span>
                      <span className="tree-arrow">→</span>
                      <span>
                        <PieceImage pieceKey={u.to_key} />
                        <b>{u.to_name}</b>
                      </span>
                    </span>
                    <span
                      className={`tree-status ${available ? "available" : "locked"}`}
                    >
                      {completed
                        ? "강화 완료"
                        : available
                          ? "선택 가능"
                          : "선행 조건 미충족"}
                    </span>
                    <span className="tree-effect">{u.effect}</span>
                    <small>{u.requirement}</small>
                  </button>
                );
              })}
              {preview ? (
                <div className="rest-confirm">
                  <button
                    className="primary"
                    onClick={() =>
                      send({ type: "rest_upgrade", id: piece.id, target: path!.to })
                    }
                  >
                    강화 확정
                  </button>
                  <button onClick={onCancel}>취소</button>
                </div>
              ) : (
                <button className="text-button rest-cancel" onClick={onCancel}>
                  선택 취소 · Esc
                </button>
              )}
            </>
          )}
        </div>
      )}
      <button
        className="text-button rest-depart"
        onClick={() => send({ type: "rest_skip" })}
      >
        휴식을 마치고 맵으로 →
      </button>
    </section>
  );
}

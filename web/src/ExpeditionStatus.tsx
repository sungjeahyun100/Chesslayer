import { useEffect, useRef } from "react";
import type { Snapshot } from "./types";
import { PieceImage } from "./PieceImage";
import { RelicIcon } from "./ContentArt";
import { CardFace } from "./CardFace";

export function ExpeditionStatus({ state, onClose }: { state: Snapshot; onClose: () => void }) {
  const closeButton = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    closeButton.current?.focus();
  }, []);
  const counts = new Map<string, { card: Snapshot["deck"][number]; count: number }>();
  for (const card of state.deck) {
    const entry = counts.get(card.kind);
    if (entry) entry.count++;
    else counts.set(card.kind, { card, count: 1 });
  }
  return <div className="expedition-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
    <section className="expedition-panel" role="dialog" aria-modal="true" aria-labelledby="expedition-title" onKeyDown={(event) => { if (event.key === "Escape") { event.stopPropagation(); onClose(); } }}>
      <header className="expedition-header">
        <div><p className="eyebrow">ACT {state.floor === 1 ? "I" : "II"} / {state.character}</p><h2 id="expedition-title">원정 현황</h2></div>
        <button ref={closeButton} className="text-button" onClick={onClose} aria-label="원정 현황 닫기">닫기 ×</button>
      </header>
      <div className="expedition-columns">
        <section className="expedition-position">
          <h3>다음 전투 시작 포지션 <strong>{state.next_battle_pieces.length}기</strong></h3>
          <div className="expedition-board" aria-label="다음 전투 아군 배치">
            {Array.from({ length: 64 }, (_, cell) => {
              const piece = state.next_battle_pieces.find((p) => p.cell === cell);
              return <div key={cell} className={(cell + Math.floor(cell / 8)) % 2 ? "dark" : "light"}>
                {piece && <span title={`${piece.name} · ${piece.square}`} data-piece-id={piece.id} data-cell={cell}>
                  <PieceImage pieceKey={piece.key} owner={0} name={piece.name} />
                  {piece.royal && <i>★</i>}
                </span>}
              </div>;
            })}
          </div>
          <p>생존 기물의 영구 강화와 시작 진형을 반영합니다. 선택한 경로의 적과 칸이 겹치면 전투 진입 시 역습이 적용됩니다.</p>
        </section>
        <section className="expedition-relics">
          <h3>보유 유물 <strong>{state.relics.length}</strong></h3>
          <div className="expedition-list">
            {state.relics.map((relic) => <article className="relic" key={relic.id}>
              <RelicIcon id={relic.id} name={relic.name} />
              <div><strong>{relic.name}</strong><p>{relic.description}</p>{relic.status && <small className="relic-status">{relic.status}</small>}</div>
            </article>)}
          </div>
        </section>
        <section className="expedition-deck">
          <h3>전체 카드 덱 <strong>{state.deck_count}장</strong></h3>
          <div className="expedition-cards">
            {Array.from(counts.values()).map(({ card, count }) => <article className="playing-card" key={card.kind}>
              <CardFace card={card} />
              {count > 1 && <b className="deck-quantity">×{count}</b>}
            </article>)}
            {!counts.size && <p>아직 획득한 카드가 없습니다.</p>}
          </div>
        </section>
      </div>
    </section>
  </div>;
}

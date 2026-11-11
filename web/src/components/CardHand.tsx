import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import type { Card, PileCard, Snapshot } from "../types";
import { CardFace } from "../CardFace";
import { useCardMotion } from "./useCardMotion";

type Movement = { sequence: number; before: Snapshot; next: Snapshot } | null;

export function CardHand({ state, movement, selectedCardId, selectedCount, disabled, onChoose }: {
  state: Snapshot; movement: Movement;
  selectedCardId?: number; selectedCount: number; disabled: boolean; onChoose: (card: Card) => void;
}) {
  const root = useRef<HTMLElement>(null);
  const closeButton = useRef<HTMLButtonElement>(null);
  const [openPile, setOpenPile] = useState<"draw" | "discard" | null>(null);
  const [visibleCounts, setVisibleCounts] = useState(() => ({ draw: state.draw_count,
    discard: state.discard_count, exhausted: state.exhausted_count }));
  useCardMotion(root, state, movement);
  useLayoutEffect(() => {
    const final = { draw: state.draw_count, discard: state.discard_count, exhausted: state.exhausted_count };
    if (!movement || movement.next !== state || window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setVisibleCounts(final);
      return;
    }
    const { before, next } = movement;
    const oldHand = new Set(before.hand.map((card) => card.id));
    const newHand = new Set(next.hand.map((card) => card.id));
    const entering = next.hand.filter((card) => !oldHand.has(card.id)).length;
    const leaving = before.hand.filter((card) => !newHand.has(card.id)).length;
    const reshuffled = before.draw_count === 0 && before.discard_count > next.discard_count;
    if (!entering && !leaving && before.draw_count === next.draw_count && before.discard_count === next.discard_count) {
      setVisibleCounts(final);
      return;
    }
    setVisibleCounts({ draw: before.phase === "combat" ? before.draw_count : next.draw_count + entering,
      discard: before.discard_count, exhausted: before.exhausted_count });
    const duration = Math.max(entering ? 480 + (entering - 1) * 90 + (reshuffled ? 340 : 0) : 0,
      leaving ? 420 + (leaving - 1) * 80 : 0,
      reshuffled ? 340 : 0);
    const timer = window.setTimeout(() => setVisibleCounts(final), duration);
    return () => window.clearTimeout(timer);
  }, [state, movement]);
  useEffect(() => {
    if (!openPile) return;
    const priorFocus = document.activeElement as HTMLElement | null;
    const priorOverflow = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    closeButton.current?.focus();
    const onKey = (event: KeyboardEvent) => { if (event.key === "Escape") setOpenPile(null); };
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("keydown", onKey);
      document.body.style.overflow = priorOverflow;
      priorFocus?.focus();
    };
  }, [openPile]);
  const cards: PileCard[] = openPile === "draw" ? state.draw_pile : state.discard_pile;
  return <section className="hand-area" ref={root}>
    <div className="card-management">
      <button className="card-pile" data-pile="draw" onClick={() => setOpenPile("draw")} aria-label={`덱 ${visibleCounts.draw}장, 목록 보기`}>
        <span className="pile-stack"><span className="pile-card-back" data-pile-anchor="draw" /></span>
        <span className="pile-caption">덱 <b data-testid="draw-count">{visibleCounts.draw}</b></span>
      </button>
      <div className="hand-center">
        <div className="hand-heading"><h2>손패 <span>{state.hand.length}/10</span></h2>
          <span className="exhaust-count" data-pile-anchor="exhausted">소멸 <b>{visibleCounts.exhausted}</b></span></div>
        <div className="hand" aria-label="손패">
          {state.hand.length === 0 && <p className="empty-hand">손패 없음</p>}
          {state.hand.map((card) => <button data-card-id={card.id}
            className={`playing-card ${selectedCardId === card.id ? "active-card" : ""}`}
            key={card.id} onClick={() => onChoose(card)} disabled={!!card.disabled_reason || disabled}
            title={card.disabled_reason || card.definition.description}>
            <CardFace card={card.definition} selectedCount={selectedCardId === card.id ? selectedCount : 0} />
            {card.disabled_reason && <span className="card-unavailable">{card.disabled_reason}</span>}
          </button>)}
        </div>
      </div>
      <button className="card-pile discard-pile" data-pile="discard" onClick={() => setOpenPile("discard")}
        aria-label={`묘지 ${visibleCounts.discard}장, 목록 보기`}>
        <span className="pile-stack"><span className="pile-card-back" data-pile-anchor="discard" /></span>
        <span className="pile-caption">묘지 <b>{visibleCounts.discard}</b></span>
      </button>
    </div>
    {openPile && createPortal(<div className="card-list-overlay" onMouseDown={(event) => {
      if (event.target === event.currentTarget) setOpenPile(null);
    }}><div className="card-list-dialog" role="dialog" aria-modal="true" aria-label={openPile === "draw" ? "덱 카드 목록" : "묘지 카드 목록"}>
      <header><h2>{openPile === "draw" ? "덱" : "묘지"} <span>{cards.length}</span></h2>
        <button ref={closeButton} onClick={() => setOpenPile(null)} aria-label="카드 목록 닫기">닫기 ×</button></header>
      <div className="card-list-grid">{cards.map((card) => <article className="playing-card" key={card.id}>
        <CardFace card={card.definition} /></article>)}
        {cards.length === 0 && <p className="empty-hand">카드 없음</p>}</div>
    </div></div>, document.body)}
  </section>;
}

import type { Card, Snapshot } from "../types";
import { CardFace } from "../CardFace";

export function CardHand({ state, selectedCardId, selectedCount, disabled, onChoose }: {
  state: Pick<Snapshot, "hand" | "draw_count" | "discard_count" | "exhausted_count" | "deck_count">;
  selectedCardId?: number; selectedCount: number; disabled: boolean; onChoose: (card: Card) => void;
}) {
  return (
    <section className="hand-area">
      <div className="hand-heading">
        <h2>손패 <span>{state.hand.length}/10</span></h2>
        <div className="piles">
          <span>
            드로우 <b data-testid="draw-count">{state.draw_count}</b>
          </span>
          <span>
            버림 <b>{state.discard_count}</b>
          </span>
          <span>
            소멸 <b>{state.exhausted_count}</b>
          </span>
          <span>
            전체 덱 <b data-testid="deck-count">{state.deck_count}</b>
          </span>
        </div>
      </div>
      <div className="hand">
        {state.hand.length === 0 && <p className="empty-hand">손패 없음</p>}
        {state.hand.map((card) => (
          <button
            data-card-id={card.id}
            className={`playing-card ${selectedCardId === card.id ? "active-card" : ""}`}
            key={card.id}
            onClick={() => onChoose(card)}
            disabled={!!card.disabled_reason || disabled}
            title={card.disabled_reason || card.definition.description}
          >
            <CardFace card={card.definition} selectedCount={selectedCardId === card.id ? selectedCount : 0} />
            {card.disabled_reason && (
              <span className="card-unavailable">
                {card.disabled_reason}
              </span>
            )}
          </button>
        ))}
      </div>
    </section>
  );
}

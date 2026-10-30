import type { CardDefinition } from "./types";
import { CardArt } from "./ContentArt";

export function CardFace({ card, selectedCount = 0 }: { card: CardDefinition; selectedCount?: number }) {
  return <>
    <div className="card-top">
      <span className="cost" aria-label={card.kind === "volley" ? "제거한 적 수만큼 탄약 소모" : card.cost_per_target ? `선택 비용 ${selectedCount}` : `비용 ${card.cost}`}>
        {card.kind === "volley" ? "N" : card.cost_per_target ? (selectedCount || "N") : card.cost}
      </span>
      <span>{["volley", "frontline"].includes(card.kind) ? "탄약 전술" : "전술 카드"}</span>
    </div>
    <div className="card-symbol"><CardArt kind={card.kind} name={card.name} /></div>
    <h3>{card.name}</h3>
    <p>{card.description}</p>
    <div className="tags">
      {card.innate && <span>선천성</span>}
      {card.exhaust && <span>소멸</span>}
      {card.retain && <span>보존</span>}
      {card.ethereal && <span>휘발성</span>}
    </div>
  </>;
}

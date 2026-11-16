import { useState } from "react";
import { CardFace } from "./CardFace";
import type { Command, Snapshot } from "./types";

export function LocationPanel({ state, send }: { state: Snapshot; send: (command: Command) => void }) {
  if (state.phase === "event" && state.event) return <>
    <p className="eyebrow">UNKNOWN ENCOUNTER · {{common:"일반", danger:"위험", rare:"희귀"}[state.event.grade]}</p><h2>{state.event.title}</h2>
    <p>{state.event.description}</p>
    {state.event.result && <p className="event-result" role="status">{state.event.result}</p>}
    <div className={state.event.choices.some(c => c.card) ? "event-card-grid" : "location-actions"}>
      {state.event.choices.map(c => <button key={c.id} disabled={c.disabled}
        className={c.card ? "playing-card" : "primary large"} data-rarity={c.card ? c.card.rarity ?? "common" : undefined} onClick={() => send({type:"event_choice",choice:c.id})}>
        {c.card ? <><CardFace card={c.card} /><span className="reward-select">{c.label}</span></> : c.label}
      </button>)}
    </div>
  </>;
  if (state.phase === "shop") return <ShopPanel state={state} send={send} />;
  return null;
}

function ShopPanel({ state, send }: { state: Snapshot; send: (command: Command) => void }) {
  const [burnOpen, setBurnOpen] = useState(false);
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const selected = state.deck.find((card) => card.id === selectedId);
  return <>
    <p className="eyebrow">SUPPLIES FOR THE ROAD</p><h2>원정 상점</h2>
    <div className="shop-tabs" role="group" aria-label="상점 기능">
      <button className={!burnOpen ? "active" : ""} onClick={() => setBurnOpen(false)} aria-pressed={!burnOpen}>전술 카드 · 20금화</button>
      <button className={burnOpen ? "active" : ""} onClick={() => setBurnOpen(true)} aria-pressed={burnOpen}>카드 소각 · {state.burn_cost}금화</button>
    </div>
    {burnOpen ? <div className="shop-burn">
      <p className="muted">현재 덱에서 카드 한 장을 영구적으로 제거합니다. 다음 소각 비용은 100금화 증가합니다.</p>
      <div className="shop-burn-layout">
        <div className="shop-burn-list" role="group" aria-label="소각할 카드 선택">
          {state.deck.map((card) => <button key={card.id} className={selectedId === card.id ? "selected" : ""}
            onClick={() => setSelectedId(card.id)} aria-pressed={selectedId === card.id}>
            <span>{card.definition.name}</span><small>#{card.id}</small>
          </button>)}
          {state.deck.length === 0 && <p className="muted">소각할 카드가 없습니다.</p>}
        </div>
        <div className="shop-burn-detail">
          {selected ? <>
            <article className="playing-card" data-card-kind={selected.definition.kind} data-rarity={selected.definition.rarity ?? "common"}><CardFace card={selected.definition} /></article>
            <p>소각 비용 <strong>{state.burn_cost}금화</strong></p>
            <button className="primary" disabled={state.gold < state.burn_cost}
              onClick={() => send({ type: "shop_burn", id: selected.id })}>
              {state.gold < state.burn_cost ? "금화가 부족합니다" : "이 카드 소각하기"}
            </button>
          </> : <p className="muted">소각할 카드를 선택하세요.</p>}
        </div>
      </div>
    </div> : <div className="shop-cards">{state.shop_offers.map((card, index) => {
      const bought = state.shop_bought.includes(index);
      return <button className="playing-card" data-rarity={card.rarity ?? "common"} key={index} disabled={bought || state.gold < 20}
        onClick={() => send({ type: "shop_buy", index })} title={bought ? "이미 구매했습니다." : state.gold < 20 ? "금화가 부족합니다." : `${card.name} 구매 · 20금화`}>
        <CardFace card={card} /><span className="reward-select">{bought ? "구매 완료 ✓" : "20금화 · 구매"}</span>
      </button>;
    })}</div>}
    <button className="primary large" onClick={() => send({ type: "shop_leave" })}>맵으로 돌아가기 →</button>
  </>;
}

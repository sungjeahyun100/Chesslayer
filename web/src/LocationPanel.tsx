import { useState } from "react";
import { CardFace } from "./CardFace";
import type { Command, Snapshot } from "./types";

export function LocationPanel({ state, send }: { state: Snapshot; send: (command: Command) => void }) {
  if (state.phase === "event") return <>
    <p className="eyebrow">A WARM MEAL</p><h2>작은 식당</h2>
    <p>오랜 원정 끝에 작은 식당을 발견했다.<br />주인은 말없이 따뜻한 음식을 내놓았다.</p>
    <p className="muted">아군 모두의 공복·굶주림·기아를 해소합니다.</p>
    <div className="location-actions"><button className="primary large" onClick={() => send({ type: "event_choice", choice: "eat" })}>식사하기 · 무료</button>
      <button className="text-button" onClick={() => send({ type: "event_choice", choice: "leave" })}>떠나기 →</button></div>
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
            <article className="playing-card" data-card-kind={selected.definition.kind}><CardFace card={selected.definition} /></article>
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
      return <button className="playing-card" key={index} disabled={bought || state.gold < 20}
        onClick={() => send({ type: "shop_buy", index })} title={bought ? "이미 구매했습니다." : state.gold < 20 ? "금화가 부족합니다." : `${card.name} 구매 · 20금화`}>
        <CardFace card={card} /><span className="reward-select">{bought ? "구매 완료 ✓" : "20금화 · 구매"}</span>
      </button>;
    })}</div>}
    <button className="primary large" onClick={() => send({ type: "shop_leave" })}>맵으로 돌아가기 →</button>
  </>;
}

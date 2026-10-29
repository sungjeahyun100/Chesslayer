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
  if (state.phase === "shop") return <>
    <p className="eyebrow">SUPPLIES FOR THE ROAD</p><h2>원정 상점</h2>
    <p>보유 금화 <strong>{state.gold}</strong> · 전술 카드 20금화</p>
    <div className="shop-cards">{state.shop_offers.map((card, index) => {
      const bought = state.shop_bought.includes(index);
      return <button className="playing-card" key={index} disabled={bought || state.gold < 20}
        onClick={() => send({ type: "shop_buy", index })} title={bought ? "이미 구매했습니다." : state.gold < 20 ? "금화가 부족합니다." : `${card.name} 구매 · 20금화`}>
        <CardFace card={card} /><span className="reward-select">{bought ? "구매 완료 ✓" : "20금화 · 구매"}</span>
      </button>;
    })}</div>
    <button className="primary large" onClick={() => send({ type: "shop_leave" })}>맵으로 돌아가기 →</button>
  </>;
  return null;
}

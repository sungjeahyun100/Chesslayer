import type { RelicId } from "./types";

const cards = import.meta.glob<string>("./assets/cards/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});
const relics = import.meta.glob<string>("./assets/relics/*.svg", {
  eager: true,
  query: "?url",
  import: "default",
});

export function CardArt({ kind, name }: { kind: string; name: string }) {
  return <img className="card-art" src={cards[`./assets/cards/${kind}.svg`] || cards["./assets/cards/ward.svg"]} alt={`${name} 일러스트`} draggable={false} />;
}

export function RelicIcon({ id, name, className = "" }: { id: RelicId; name: string; className?: string }) {
  return <img className={`relic-icon ${className}`} src={relics[`./assets/relics/${id}.svg`] || relics["./assets/relics/initiative.svg"]} alt={`${name} 아이콘`} draggable={false} />;
}

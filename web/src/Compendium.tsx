import { useMemo, useState } from "react";
import { CardFace } from "./CardFace";
import { RelicIcon } from "./ContentArt";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import type { CardDefinition, Character, RestOption, UpgradePath } from "./types";

export interface CompendiumSquad {
  id: string;
  cards: { definition: CardDefinition; starting: boolean; available: boolean }[];
  relics: { id: string; name: string; description: string; starting: boolean }[];
  pieces: CompendiumPiece[];
}
export interface CompendiumPiece {
  kind: number;
  key: string;
  name: string;
  description: string;
  movement: string;
  moves: number[];
  captures: number[];
  shots: { cell: number; chance: number }[];
  starting: boolean;
  upgrades: { from: number; from_key: string; from_name: string; to: number; to_key: string; to_name: string; effect: string; requirement: string; required_pieces: { kind: number; name: string }[] }[];
}

const categoryNames = { cards: "카드", relics: "유물", pieces: "기물" } as const;
type Category = keyof typeof categoryNames;

function MovementBoard({ piece }: { piece: CompendiumPiece }) {
  const moves = new Set(piece.moves);
  const captures = new Set(piece.captures);
  return <div className="compendium-board-wrap">
    <div className="board compendium-board" role="img" aria-label={`${piece.name} 이동과 포획 예시. 녹색 점은 빈 칸 이동, 주황색 고리는 적이 있을 때 포획 가능한 칸입니다.`}>
      {Array.from({ length: 64 }, (_, cell) => <div key={cell}
        className={`square ${(cell + Math.floor(cell / 8)) % 2 ? "dark" : "light"} ${cell === 35 ? "selected" : ""}`}>
        {moves.has(cell) && <span className="move-dot" />}
        {captures.has(cell) && <span className="capture-dot" />}
        {cell === 35 && <PieceImage pieceKey={piece.key} name={piece.name} />}
      </div>)}
    </div>
    <p className="muted">● 빈 칸 이동 · ◯ 적이 있을 때 포획 · 중앙 기물 기준</p>
    <p className="muted">보드 가장자리, 아군·적군 배치, 첫 이동 조건과 특수 능력에 따라 실제 가능한 수가 달라집니다.</p>
  </div>;
}

function ShootingBoard({ piece }: { piece: CompendiumPiece }) {
  const shots = new Map(piece.shots.map(({ cell, chance }) => [cell, chance]));
  return <div className="compendium-board-wrap">
    <div className="board compendium-board compendium-shooting-board" role="img" aria-label={`${piece.name} 사격 범위 예시. 붉은 칸은 사격 가능한 위치이며 숫자는 기본 명중률입니다.`}>
      {Array.from({ length: 64 }, (_, cell) => <div key={cell}
        className={`square ${(cell + Math.floor(cell / 8)) % 2 ? "dark" : "light"} ${cell === 35 ? "selected" : ""} ${shots.has(cell) ? "shot-range" : ""}`}>
        {shots.has(cell) && <span className="shot-chance">{shots.get(cell)}%</span>}
        {cell === 35 && <PieceImage pieceKey={piece.key} name={piece.name} />}
      </div>)}
    </div>
    <p className="muted">붉은 칸은 빈 보드에서의 사격 범위이며, 숫자는 기본 명중률입니다.</p>
    <p className="muted">기물에 막히는 사격과 유물 보너스는 실제 전투에서 적용됩니다.</p>
  </div>;
}

export function Compendium({ choices, entries, onBack }: { choices: Character[]; entries: CompendiumSquad[]; onBack: () => void }) {
  const [squadId, setSquadId] = useState("guardian");
  const [category, setCategory] = useState<Category>("cards");
  const [pieceKind, setPieceKind] = useState<number | null>(null);
  const squad = entries.find((entry) => entry.id === squadId);
  const character = choices.find((choice) => choice.id === squadId);
  const selectedPiece = squad?.pieces.find((piece) => piece.kind === pieceKind) ?? squad?.pieces[0];
  const allPaths: UpgradePath[] = useMemo(() => squad?.pieces.flatMap((piece) => piece.upgrades.map((edge) => ({
    ...edge, key: `${edge.from}-${edge.to}`, unlocked: false,
    required_pieces: edge.required_pieces.map((required) => ({ ...required, owned: false })),
  }))) ?? [], [squad]);
  const cardOwners = (kind: string) => entries.filter((entry) => entry.cards.some((card) => card.definition.kind === kind)).length;
  const relicOwners = (id: string) => entries.filter((entry) => entry.relics.some((relic) => relic.id === id)).length;
  const restPiece: RestOption | null = selectedPiece ? {
    id: 0, kind: selectedPiece.kind, key: selectedPiece.key, name: selectedPiece.name,
    description: selectedPiece.description, cell: 35, origin: 35, square: "d4", upgrades: [],
    ward: false, can_upgrade: false, can_revive: false, dead: false, statuses: [],
  } : null;
  return <main className="compendium-page">
    <button className="text-button" onClick={onBack}>← 처음으로</button>
    <p className="eyebrow">CHESSLAYER / FIELD GUIDE</p>
    <h1>도감</h1>
    <p className="muted">모든 부대의 구현된 전술과 기물을 열람할 수 있습니다. 도감 열람은 해금 기록이나 원정에 영향을 주지 않습니다.</p>
    <nav className="compendium-squads" aria-label="부대 선택">
      {choices.map((choice) => <button key={choice.id} type="button" className={squadId === choice.id ? "selected" : ""}
        aria-pressed={squadId === choice.id} onClick={() => { setSquadId(choice.id); setPieceKind(null); }}>
        {choice.name}
      </button>)}
    </nav>
    <section className="compendium-head"><p className="eyebrow">{character?.title}</p><h2>{character?.name}</h2><p>{character?.description}</p></section>
    <nav className="compendium-categories" aria-label="도감 종류">
      {(Object.keys(categoryNames) as Category[]).map((key) => <button key={key} type="button" className={category === key ? "selected" : ""}
        aria-pressed={category === key} onClick={() => setCategory(key)}>{categoryNames[key]} <small>{squad?.[key].length ?? 0}</small></button>)}
    </nav>
    {!squad && <p>도감 정보를 불러오는 중입니다.</p>}
    {squad && category === "cards" && <div className="compendium-grid">
      {squad.cards.map((card) => <article key={card.definition.kind} className="compendium-card">
        <div className="compendium-badges"><span>{cardOwners(card.definition.kind) > 1 ? "공용" : "부대 전용"}</span>{card.starting && <span>시작 덱</span>}{!card.available && <span>현재 획득 경로 없음</span>}</div>
        <CardFace card={card.definition} />
        <p className="compendium-condition">{card.definition.cost_per_target ? "대상 기물마다 코스트 1" : `코스트 ${card.definition.cost}`}
          {card.definition.targeted ? ` · ${card.definition.selection_mode === "single" ? "대상 1개 선택" : "복수 대상 선택"}` : ""}</p>
      </article>)}
      {!squad.cards.length && <p>현재 정의된 카드가 없습니다.</p>}
    </div>}
    {squad && category === "relics" && <div className="compendium-grid relics">
      {squad.relics.map((relic) => <article key={relic.id} className="compendium-relic">
        <div className="compendium-badges"><span>{relicOwners(relic.id) > 1 ? "공용" : "부대 전용"}</span>{relic.starting && <span>시작 유물</span>}</div>
        <RelicIcon id={relic.id} name={relic.name} /><h3>{relic.name}</h3><p>{relic.description}</p>
      </article>)}
      {!squad.relics.length && <p>현재 정의된 유물이 없습니다.</p>}
    </div>}
    {squad && category === "pieces" && <div className="compendium-pieces">
      <nav className="compendium-piece-list" aria-label="기물 선택">
        {squad.pieces.map((piece) => <button key={piece.kind} type="button" className={selectedPiece?.kind === piece.kind ? "selected" : ""}
          aria-pressed={selectedPiece?.kind === piece.kind} onClick={() => setPieceKind(piece.kind)}>
          <PieceImage pieceKey={piece.key} name={piece.name} /><span><strong>{piece.name}</strong><small>{piece.starting ? "시작 기물" : "강화·승격 기물"}</small></span>
        </button>)}
      </nav>
      {selectedPiece && restPiece && <article className="compendium-piece-detail" key={selectedPiece.kind}>
        <div className="compendium-piece-intro"><PieceImage pieceKey={selectedPiece.key} name={selectedPiece.name} /><div><p className="eyebrow">{selectedPiece.starting ? "시작 기물" : "강화·승격 기물"}</p><h3>{selectedPiece.name}</h3><p>{selectedPiece.description}</p></div></div>
        <h4>이동과 포획</h4><MovementBoard piece={selectedPiece} />
        {selectedPiece.shots.length > 0 && <><h4>사격 범위</h4><ShootingBoard piece={selectedPiece} /></>}
        {allPaths.some((path) => path.from === selectedPiece.kind || path.to === selectedPiece.kind) ?
          <UpgradeTree piece={restPiece} paths={allPaths} acquired={[]} onSelect={() => {}} onConfirm={() => {}} completed readOnly /> :
          <p className="muted">정의된 영구 강화 경로가 없습니다.</p>}
      </article>}
    </div>}
  </main>;
}

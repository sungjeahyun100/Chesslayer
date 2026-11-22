import { useMemo, useState } from "react";
import { CardFace } from "./CardFace";
import { RelicIcon } from "./ContentArt";
import { PieceImage } from "./PieceImage";
import { UpgradeTree } from "./UpgradeTree";
import { statusCatalog } from "./game/statusCatalog";
import { rarityNames } from "./rarity";
import type { CardDefinition, Character, Rarity, RestOption, UpgradePath } from "./types";

export interface CompendiumSquad {
  id: string;
  cards: { definition: CardDefinition; starting: boolean; available: boolean }[];
  relics: { id: string; name: string; description: string; rarity?: Rarity; starting: boolean }[];
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
  monster_only: boolean;
  upgrades: { from: number; from_key: string; from_name: string; to: number; to_key: string; to_name: string; effect: string; requirement: string; required_pieces: { kind: number; name: string }[] }[];
}

const categoryNames = { cards: "카드", relics: "유물", pieces: "기물", statuses: "상태이상" } as const;
const rarityOrder: Record<Rarity, number> = { common: 0, rare: 1, epic: 2, legendary: 3 };
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
  const squadId = "avenger";
  const [category, setCategory] = useState<Category>("cards");
  const [pieceKind, setPieceKind] = useState<number | null>(null);
  const [pieceTab, setPieceTab] = useState<"playable" | "monster">("playable");
  const [pieceQuery, setPieceQuery] = useState("");
  const [statusKind, setStatusKind] = useState<string>(statusCatalog[0].kind);
  const [statusQuery, setStatusQuery] = useState("");
  const squad = entries.find((entry) => entry.id === squadId);
  const sortedCards = useMemo(() => squad?.cards.slice().sort((a, b) =>
    rarityOrder[a.definition.rarity ?? "common"] - rarityOrder[b.definition.rarity ?? "common"]) ?? [], [squad]);
  const sortedRelics = useMemo(() => squad?.relics.slice().sort((a, b) =>
    rarityOrder[a.rarity ?? "common"] - rarityOrder[b.rarity ?? "common"] || a.name.localeCompare(b.name, "ko")) ?? [], [squad]);
  const visiblePieces = useMemo(() => squad?.pieces.filter((piece) =>
    piece.monster_only === (pieceTab === "monster") && piece.name.includes(pieceQuery.trim())) ?? [], [squad, pieceTab, pieceQuery]);
  const character = choices.find((choice) => choice.id === squadId);
  const selectedPiece = visiblePieces.find((piece) => piece.kind === pieceKind) ?? visiblePieces[0];
  const visibleStatuses = statusCatalog.filter((status) => status.name.includes(statusQuery.trim()));
  const selectedStatus = visibleStatuses.find((status) => status.kind === statusKind) ?? visibleStatuses[0];
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
    <p className="muted">복수귀 카드와 유물, 모든 기물과 상태이상을 열람할 수 있습니다.</p>
    <section className="compendium-head">{category === "pieces" ? <><p className="eyebrow">공용 게임 데이터</p><h2>전체 기물</h2><p>플레이 가능 부대와 관계없이 모든 기물의 정의와 강화 경로를 열람할 수 있습니다.</p></> : category === "statuses" ? <><p className="eyebrow">전투 효과</p><h2>상태이상</h2><p>기물에 적용되는 효과와 해제 시점을 확인할 수 있습니다.</p></> : <><p className="eyebrow">{character?.title}</p><h2>{character?.name}</h2><p>{character?.description}</p></>}</section>
    <nav className="compendium-categories" aria-label="도감 종류">
      {(Object.keys(categoryNames) as Category[]).map((key) => <button key={key} type="button" className={category === key ? "selected" : ""}
        aria-pressed={category === key} onClick={() => setCategory(key)}>{categoryNames[key]} <small>{key === "statuses" ? statusCatalog.length : squad?.[key].length ?? 0}</small></button>)}
    </nav>
    {!squad && <p>도감 정보를 불러오는 중입니다.</p>}
    {squad && category === "cards" && <div className="compendium-grid">
      {sortedCards.map((card) => <article key={card.definition.kind} className="compendium-card" data-rarity={card.definition.rarity ?? "common"}>
        <div className="compendium-badges"><span>{cardOwners(card.definition.kind) > 1 ? "공용" : "부대 전용"}</span>{card.starting && <span>시작 덱</span>}{!card.available && <span>현재 획득 경로 없음</span>}</div>
        <CardFace card={card.definition} />
        <p className="compendium-condition">{card.definition.cost_per_target ? "대상 기물마다 코스트 1" : `코스트 ${card.definition.cost}`}
          {card.definition.targeted ? ` · ${card.definition.selection_mode === "single" ? "대상 1개 선택" : "복수 대상 선택"}` : ""}</p>
      </article>)}
      {!squad.cards.length && <p>현재 정의된 카드가 없습니다.</p>}
    </div>}
    {squad && category === "relics" && <div className="compendium-grid relics">
      {sortedRelics.map((relic) => <article key={relic.id} className="compendium-relic" data-rarity={relic.rarity ?? "common"}>
        <div className="compendium-badges"><span>{rarityNames[relic.rarity ?? "common"]}</span><span>{relicOwners(relic.id) > 1 ? "공용" : "부대 전용"}</span>{relic.starting && <span>시작 유물</span>}</div>
        <RelicIcon id={relic.id} name={relic.name} /><h3>{relic.name}</h3><p>{relic.description}</p>
      </article>)}
      {!squad.relics.length && <p>현재 정의된 유물이 없습니다.</p>}
    </div>}
    {squad && category === "pieces" && <div className="compendium-pieces">
      <div className="compendium-piece-index"><nav className="compendium-piece-tabs" aria-label="기물 분류">
        <button type="button" aria-pressed={pieceTab === "playable"} className={pieceTab === "playable" ? "selected" : ""} onClick={() => setPieceTab("playable")}>플레이어블 기물</button>
        <button type="button" aria-pressed={pieceTab === "monster"} className={pieceTab === "monster" ? "selected" : ""} onClick={() => setPieceTab("monster")}>몬스터 전용 기물</button>
      </nav><label htmlFor="compendium-piece-search">기물 찾기</label><input id="compendium-piece-search" type="search" value={pieceQuery} onChange={(event) => setPieceQuery(event.target.value)} placeholder="이름 검색" />
      <nav className="compendium-piece-list" aria-label="기물 선택">
        {visiblePieces.map((piece) => <button key={piece.kind} type="button" className={selectedPiece?.kind === piece.kind ? "selected" : ""}
          aria-pressed={selectedPiece?.kind === piece.kind} onClick={() => setPieceKind(piece.kind)}>
          <PieceImage pieceKey={piece.key} name={piece.name} /><span><strong>{piece.name}</strong><small>{piece.key === "corpse" ? "중립 기물" : piece.monster_only ? "몬스터 전용" : piece.starting ? "복수귀 시작 기물" : "기물 정의"}</small></span>
        </button>)}
      </nav>{!visiblePieces.length && <p className="muted">검색 결과가 없습니다.</p>}</div>
      {selectedPiece && restPiece && <article className="compendium-piece-detail" key={selectedPiece.kind}>
        <div className="compendium-piece-intro"><PieceImage pieceKey={selectedPiece.key} name={selectedPiece.name} /><div><p className="eyebrow">{selectedPiece.key === "corpse" ? "중립 기물" : selectedPiece.monster_only ? "몬스터 전용" : selectedPiece.starting ? "복수귀 시작 기물" : "기물 정의"}</p><h3>{selectedPiece.name}</h3><p>{selectedPiece.description}</p></div></div>
        <h4>이동과 포획</h4><MovementBoard piece={selectedPiece} />
        {selectedPiece.shots.length > 0 && <><h4>사격 범위</h4><ShootingBoard piece={selectedPiece} /></>}
        {allPaths.some((path) => path.from === selectedPiece.kind || path.to === selectedPiece.kind) ?
          <UpgradeTree piece={restPiece} paths={allPaths} acquired={[]} onSelect={() => {}} onConfirm={() => {}} completed readOnly /> :
          <p className="muted">정의된 영구 강화 경로가 없습니다.</p>}
      </article>}
    </div>}
    {category === "statuses" && <div className="compendium-statuses">
      <div className="compendium-status-index">
        <label htmlFor="compendium-status-search">상태이상 찾기</label>
        <input id="compendium-status-search" type="search" value={statusQuery} onChange={(event) => setStatusQuery(event.target.value)} placeholder="이름 검색" />
        <nav className="compendium-status-list" aria-label="상태이상 선택">
          {visibleStatuses.map((status) => <button key={status.kind} type="button" className={selectedStatus?.kind === status.kind ? "selected" : ""}
            aria-pressed={selectedStatus?.kind === status.kind} onClick={() => setStatusKind(status.kind)}>
            <span className={`compendium-status-icon${status.kind === "disarmed" ? " word" : ""}`} aria-hidden="true">{status.icon}</span><strong>{status.name}</strong>
          </button>)}
        </nav>
        {!visibleStatuses.length && <p className="muted">검색 결과가 없습니다.</p>}
      </div>
      {selectedStatus && <article className="compendium-status-detail" key={selectedStatus.kind}>
        <header><span className={`compendium-status-icon${selectedStatus.kind === "disarmed" ? " word" : ""}`} aria-hidden="true">{selectedStatus.icon}</span><h3>{selectedStatus.name}</h3></header>
        <dl>
          <div><dt>효과</dt><dd>{selectedStatus.effect}</dd></div>
          {selectedStatus.duration && <div><dt>지속 시간</dt><dd>{selectedStatus.duration}</dd></div>}
          {selectedStatus.stacking && <div><dt>중첩 규칙</dt><dd>{selectedStatus.stacking}</dd></div>}
          {selectedStatus.removal && <div><dt>해제 조건</dt><dd>{selectedStatus.removal}</dd></div>}
        </dl>
      </article>}
    </div>}
  </main>;
}

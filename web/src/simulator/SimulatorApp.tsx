import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import init, { WebGame, catalog, compendium, simulator_catalog } from "../wasm/chesslayer_web";
import { GameScreen } from "../screens/GameScreen";
import { PieceImage } from "../PieceImage";
import { UpgradeTree } from "../UpgradeTree";
import { emptyEndlessRecord } from "../endless";
import { useEnemyTurn } from "../game/useEnemyTurn";
import { useGamePresentation } from "../game/useGamePresentation";
import { selfDestructKeys } from "../game/combatPlan";
import type { CompendiumSquad } from "../Compendium";
import type { Character, Command, EnemyInspection, RestOption, Snapshot, UpgradePath } from "../types";
import { restoreSettings, saveSettings } from "./settings";

type Starter = { id: number; kind: number; key: string; name: string; cell: number };
type Squad = { id: string; deck: string[]; relics: string[]; pieces: Starter[] };
type Encounter = { index: number; id: string; name: string; description: string; floor: number;
  final: boolean;
  tier: "normal" | "elite" | "boss"; pieces: Starter[]; royal_id: number;
  additional_royal_ids: number[]; random_royal: boolean };
type Catalog = { squads: Squad[]; encounters: Encounter[] };
type Config = { squad: string; encounter: number; seed: number; deck: string[]; relics: string[];
  upgrades: { id: number; target: number }[] };
const tierNames = { normal: "일반", elite: "정예", boss: "보스" };
const rarityNames = { common: "일반", rare: "희귀", epic: "영웅", legendary: "전설" };
const emptyCatalog: Catalog = { squads: [], encounters: [] };

function descendants(paths: UpgradePath[], root: number) {
  const seen = new Set([root]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const path of paths) if (seen.has(path.from) && !seen.has(path.to)) {
      seen.add(path.to); changed = true;
    }
  }
  return seen;
}

export function SimulatorApp() {
  const engine = useRef<WebGame | null>(null);
  const [ready, setReady] = useState(false);
  const [characters, setCharacters] = useState<Character[]>([]);
  const [entries, setEntries] = useState<CompendiumSquad[]>([]);
  const [content, setContent] = useState<Catalog>(emptyCatalog);
  const [squadId, setSquadId] = useState("");
  const [deck, setDeck] = useState<string[]>([]);
  const [relics, setRelics] = useState<string[]>([]);
  const [upgrades, setUpgrades] = useState<Record<number, number>>({});
  const [selectedPiece, setSelectedPiece] = useState<number | null>(null);
  const [encounterId, setEncounterId] = useState<number | null>(null);
  const [cardQuery, setCardQuery] = useState("");
  const [cardRarity, setCardRarity] = useState("all");
  const [relicQuery, setRelicQuery] = useState("");
  const [encounterQuery, setEncounterQuery] = useState("");
  const [floorFilter, setFloorFilter] = useState("all");
  const [screen, setScreen] = useState<"setup" | "battle">("setup");
  const focusEnemies = useRef(false);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [cardMovement, setCardMovement] = useState<{ sequence: number; before: Snapshot; next: Snapshot } | null>(null);
  const sequence = useRef(0);
  const [battleId, setBattleId] = useState(0);
  const [error, setError] = useState("");
  const config = useRef<Config | null>(null);
  const presentation = useGamePresentation(useCallback(() => {}, []));
  const { present } = presentation;

  useEffect(() => {
    let active = true;
    init().then(() => {
      if (!active) return;
      const squads: Character[] = JSON.parse(catalog());
      const data: Catalog = JSON.parse(simulator_catalog());
      const compendiumEntries: CompendiumSquad[] = JSON.parse(compendium());
      setCharacters(squads); setEntries(compendiumEntries); setContent(data);
      const saved = restoreSettings(data, compendiumEntries);
      if (saved) {
        setSquadId(saved.squad); setDeck(saved.deck); setRelics(saved.relics);
        setUpgrades(saved.upgrades);
        setSelectedPiece(data.squads.find((item) => item.id === saved.squad)?.pieces[0]?.id ?? null);
        setEncounterId(saved.encounter);
      }
      setReady(true);
    }).catch((cause) => { if (active) setError(`엔진 로딩 실패: ${String(cause)}`); });
    return () => { active = false; };
  }, []);
  useEffect(() => {
    if (ready && encounterId !== null)
      saveSettings({ squad: squadId, encounter: encounterId, deck, relics, upgrades });
  }, [ready, squadId, encounterId, deck, relics, upgrades]);
  useEffect(() => {
    if (screen === "setup" && focusEnemies.current) {
      focusEnemies.current = false;
      document.getElementById("sim-encounters")?.scrollIntoView({ block: "start" });
    }
  }, [screen]);

  const squad = content.squads.find((item) => item.id === squadId);
  const character = characters.find((item) => item.id === squadId);
  const entry = entries.find((item) => item.id === squadId);
  const encounter = content.encounters.find((item) => item.index === encounterId);
  const activePiece = squad?.pieces.find((piece) => piece.id === selectedPiece) ?? squad?.pieces[0];
  const allPaths = useMemo<UpgradePath[]>(() => entry?.pieces.flatMap((piece) => piece.upgrades.map((edge) => ({
    ...edge, key: `${edge.from}-${edge.to}`, unlocked: true,
    required_pieces: edge.required_pieces.map((required) => ({ ...required, owned: true })),
  }))) ?? [], [entry]);
  const reachable = useMemo(() => activePiece ? descendants(allPaths, activePiece.kind) : new Set<number>(), [activePiece, allPaths]);
  const paths = useMemo(() => allPaths.filter((path) => reachable.has(path.from) && reachable.has(path.to)), [allPaths, reachable]);
  const chosenKind = activePiece && (upgrades[activePiece.id] ?? activePiece.kind);
  const chosenDefinition = entry?.pieces.find((piece) => piece.kind === chosenKind);
  const treePiece: RestOption | null = activePiece ? {
    id: activePiece.id, kind: activePiece.kind, key: activePiece.key, name: activePiece.name,
    description: entry?.pieces.find((piece) => piece.kind === activePiece.kind)?.description ?? "",
    cell: activePiece.cell, origin: activePiece.cell, square: "", upgrades: [], ward: false,
    can_upgrade: true, can_revive: false, dead: false, statuses: [],
  } : null;
  const cardCounts = useMemo(() => deck.reduce<Record<string, number>>((counts, kind) => {
    counts[kind] = (counts[kind] ?? 0) + 1; return counts;
  }, {}), [deck]);
  const availableCards = entry?.cards.filter((item) => item.available) ?? [];
  const availableRelics = entry?.relics ?? [];
  const shownCards = availableCards.filter(({ definition }) =>
    (cardRarity === "all" || (definition.rarity ?? "common") === cardRarity)
    && `${definition.name} ${definition.description} ${definition.kind}`.toLowerCase().includes(cardQuery.toLowerCase()));
  const shownRelics = availableRelics.filter((relic) =>
    `${relic.name} ${relic.description} ${relic.id}`.toLowerCase().includes(relicQuery.toLowerCase()));
  const shownEncounters = content.encounters.filter((item) =>
    (floorFilter === "all" || (floorFilter === "final" ? item.final : !item.final && String(item.floor) === floorFilter))
    && `${item.name} ${item.id} ${item.description}`.toLowerCase().includes(encounterQuery.toLowerCase()));

  const send = useCallback((command: Command): boolean => {
    if (!engine.current) return false;
    try {
      const before: Snapshot = JSON.parse(engine.current.snapshot());
      const context: { enemyTarget?: number; blast?: number[] } = {};
      if (command.type === "enemy_step_selected" && command.from !== null && command.index !== null) {
        context.enemyTarget = before.boss ? before.boss.plan?.targets[0]?.cell ?? before.boss.plan?.to
          : engine.current.enemy_action_target(command.from, command.index);
        if (context.enemyTarget === command.from && before.pieces.some((piece) => piece.cell === command.from && selfDestructKeys.has(piece.key)))
          context.blast = JSON.parse(engine.current.blast_cells(command.from));
      }
      if (command.type === "special_action" && command.from === command.to && before.pieces.some((piece) => piece.cell === command.from && selfDestructKeys.has(piece.key)))
        context.blast = JSON.parse(engine.current.blast_cells(command.from));
      const next: Snapshot = JSON.parse(engine.current.dispatch(JSON.stringify(command)));
      setSnapshot(next); setCardMovement({ sequence: ++sequence.current, before, next });
      present(next, before, command, context);
      setError(""); return true;
    } catch (cause) { setError(String(cause)); return false; }
  }, [present]);
  useEnemyTurn(screen === "battle" && !presentation.presentationBusy, snapshot, engine, send, setError);

  function start(nextConfig: Config) {
    try {
      const game = WebGame.start_simulator(JSON.stringify(nextConfig));
      engine.current?.free(); engine.current = game;
      config.current = nextConfig;
      const next: Snapshot = JSON.parse(game.snapshot());
      setSnapshot(next); setCardMovement(null); setBattleId((value) => value + 1);
      present(next, null, "resume"); setScreen("battle"); setError("");
    } catch (cause) { setError(String(cause)); }
  }
  function inspectEnemy(cell: number): EnemyInspection | null {
    try { return engine.current ? JSON.parse(engine.current.inspect_enemy(cell)) : null; }
    catch (cause) { setError(String(cause)); return null; }
  }
  function selectSquad(id: string) {
    const next = content.squads.find((item) => item.id === id);
    if (!next) return;
    setSquadId(id); setDeck([...next.deck]); setRelics([...next.relics]); setUpgrades({});
    setSelectedPiece(next.pieces[0]?.id ?? null);
  }
  const changeEnemy = () => { focusEnemies.current = true; setScreen("setup"); };

  return <div className="sim-app">
    <header className="sim-header"><div><p className="eyebrow">CHESSLAYER / LOCAL TOOLS</p><h1>전투 시뮬레이터</h1></div>
      {screen === "battle" && <div className="sim-actions"><button onClick={() => config.current && start(config.current)}>전투 재시작</button>
        <button onClick={() => setScreen("setup")}>설정으로 돌아가기</button><button onClick={changeEnemy}>전투 상대 변경</button></div>}</header>
    {error && <p className="sim-error" role="alert">{error}<button onClick={() => setError("")}>닫기</button></p>}
    {!ready && !error && <p>전투 엔진과 콘텐츠를 불러오는 중입니다.</p>}
    {screen === "setup" && ready && <>
      <div className="sim-grid">
        <section className="sim-panel"><h2>1 · 플레이어 부대</h2>
          <select aria-label="플레이어 부대" value={squadId} onChange={(event) => selectSquad(event.target.value)}>
            {characters.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select>
          <p>{character?.description}</p><h3>시작 기물 · {squad?.pieces.length ?? 0}기</h3>
          <div className="sim-piece-list">{squad?.pieces.map((piece) => {
            const applied = entry?.pieces.find((item) => item.kind === (upgrades[piece.id] ?? piece.kind));
            return <button key={piece.id} className={activePiece?.id === piece.id ? "selected" : ""} onClick={() => setSelectedPiece(piece.id)}>
              <PieceImage pieceKey={applied?.key ?? piece.key} name={applied?.name ?? piece.name} />
              <span>#{piece.id} · {piece.name}<small>{applied && applied.kind !== piece.kind ? `→ ${applied.name}` : "기본 단계"}</small></span></button>;
          })}</div>
          {treePiece && <><h3>#{treePiece.id} · 강화 단계</h3>
            <p>{chosenDefinition?.name ?? activePiece?.name} · {chosenDefinition?.description ?? treePiece.description}</p>
            {paths.length ? <UpgradeTree key={`${squadId}-${treePiece.id}`} piece={treePiece} paths={paths} acquired={[]}
              onSelect={() => {}} onConfirm={(target) => setUpgrades((current) => ({ ...current, [treePiece.id]: target }))}
              completed={false} instant selectedKind={chosenKind} /> : <p>등록된 강화 경로가 없습니다.</p>}
            <button onClick={() => setUpgrades((current) => ({ ...current, [treePiece.id]: treePiece.kind }))}>기본 단계로 되돌리기</button></>}
        </section>
        <section className="sim-panel"><h2>2 · 덱과 유물</h2><h3>현재 덱 · {deck.length}장</h3>
          <div className="sim-chips">{Object.entries(cardCounts).map(([kind, count]) => {
            const card = availableCards.find((item) => item.definition.kind === kind)?.definition;
            return <span key={kind}>{card?.name ?? kind} ×{count}<button aria-label={`${card?.name ?? kind} 한 장 제거`}
              onClick={() => setDeck((current) => { const copy = [...current]; copy.splice(copy.indexOf(kind), 1); return copy; })}>×</button></span>;
          })}{!deck.length && <em>빈 덱</em>}</div>
          <div className="sim-filters"><input aria-label="카드 검색" placeholder="카드 검색" value={cardQuery} onChange={(event) => setCardQuery(event.target.value)} />
            <select aria-label="카드 희귀도" value={cardRarity} onChange={(event) => setCardRarity(event.target.value)}>
              <option value="all">모든 희귀도</option>{Object.entries(rarityNames).map(([key, name]) => <option key={key} value={key}>{name}</option>)}</select></div>
          <div className="sim-catalog">{shownCards.map(({ definition }) => <article key={definition.kind}>
            <div><strong>{definition.name}</strong><small>{rarityNames[definition.rarity ?? "common"]} · 비용 {definition.cost_per_target ? "대상당 1" : definition.cost} · {definition.selection_mode}</small></div>
            <p>{definition.description}</p><button onClick={() => setDeck((current) => [...current, definition.kind])}>덱에 추가</button>
          </article>)}</div>
          <h3>유물 · {relics.length}개</h3><div className="sim-chips">{relics.map((id) => {
            const relic = availableRelics.find((item) => item.id === id);
            return <span key={id}>{relic?.name ?? id}<button aria-label={`${relic?.name ?? id} 제거`} onClick={() => setRelics((current) => current.filter((item) => item !== id))}>×</button></span>;
          })}{!relics.length && <em>장착 유물 없음</em>}</div>
          <div className="sim-filters"><input aria-label="유물 검색" placeholder="유물 검색" value={relicQuery}
            onChange={(event) => setRelicQuery(event.target.value)} /></div>
          <div className="sim-catalog sim-relics">{shownRelics.map((relic) => <article key={relic.id}>
            <strong>{relic.name}</strong><small>{rarityNames[relic.rarity ?? "common"]}</small><p>{relic.description}</p>
            <button disabled={relics.includes(relic.id)} onClick={() => setRelics((current) => [...current, relic.id])}>장착</button>
          </article>)}</div>
        </section>
        <section className="sim-panel" id="sim-encounters"><h2>3 · 전투 상대</h2>
          <div className="sim-filters"><input aria-label="전투 상대 검색" placeholder="이름 검색" value={encounterQuery} onChange={(event) => setEncounterQuery(event.target.value)} />
            <select aria-label="층 필터" value={floorFilter} onChange={(event) => setFloorFilter(event.target.value)}>
              <option value="all">모든 층</option>{[1, 2, 3, 4].map((floor) => <option key={floor} value={floor}>{floor}층</option>)}<option value="final">최종 결전</option></select></div>
          <div className="sim-encounter-list">{shownEncounters.map((item) => <button key={item.index} className={item.index === encounterId ? "selected" : ""} onClick={() => setEncounterId(item.index)}>
            <strong>{item.name}</strong><small>{item.final ? "최종 결전" : `${item.floor}층`} · {tierNames[item.tier]}</small></button>)}</div>
          {encounter && <div className="sim-enemy-detail"><h3>{encounter.name}</h3><p>{encounter.description}</p>
            <p>{encounter.final ? "최종 결전" : `${encounter.floor}층`} · {tierNames[encounter.tier]} · {encounter.id}</p>
            <h4>초기 기물 · {encounter.pieces.length}기</h4><div className="sim-lineup">{encounter.pieces.map((piece) => <span key={piece.id}><PieceImage pieceKey={piece.key} owner={1} />{piece.name}
              {(encounter.random_royal || piece.id === encounter.royal_id || encounter.additional_royal_ids.includes(piece.id)) && " ★"}</span>)}</div>
            <p className="muted">{encounter.random_royal ? "로얄: 전투 시작 시 배치된 적 중 무작위 지정" : `로얄: ${encounter.pieces.filter((piece) => piece.id === encounter.royal_id || encounter.additional_royal_ids.includes(piece.id)).map((piece) => piece.name).join(", ")}`}</p></div>}
        </section>
      </div>
      <footer className="sim-launch"><span>{character?.name} · 덱 {deck.length}장 · 유물 {relics.length}개 · {encounter?.name ?? "상대 미선택"}</span>
        <button className="primary large" disabled={!squad || !encounter} onClick={() => start({ squad: squadId, encounter: encounterId!,
          seed: crypto.getRandomValues(new Uint32Array(1))[0], deck: [...deck], relics: [...relics],
          upgrades: squad!.pieces.filter((piece) => upgrades[piece.id] !== undefined && upgrades[piece.id] !== piece.kind)
            .map((piece) => ({ id: piece.id, target: upgrades[piece.id] })),
        })}>전투 시작 →</button></footer>
    </>}
    {screen === "battle" && snapshot && presentation.displayState && <>
      <GameScreen key={battleId} simulator snapshot={snapshot} displayState={presentation.displayState} cardMovement={cardMovement}
        send={send} inspectEnemy={inspectEnemy} setError={setError} onRestart={() => config.current && start(config.current)}
        developerMode={false} upgradeHistory={[]} rewardUpgradeHistory={[]} presentationBusy={presentation.presentationBusy}
        lastEnemyMove={presentation.lastEnemyMove} boardRevision={presentation.boardRevision} animation={presentation.animation}
        statusEffects={presentation.statusEffects} endlessBest={emptyEndlessRecord} endlessRunBest={emptyEndlessRecord} />
      {snapshot.phase !== "combat" && <div className="sim-result" role="status"><h2>{snapshot.phase === "victory" ? "승리" : "패배"}</h2>
        <p>{snapshot.log.at(-1)}</p><button className="primary" onClick={() => config.current && start(config.current)}>같은 설정으로 재시작</button></div>}
    </>}
  </div>;
}

import { useMemo, useRef, useState } from "react";
import { WebSandbox } from "./wasm/chesslayer_web";
import { CardFace } from "./CardFace";
import { PieceImage } from "./PieceImage";
import { PromotionPicker } from "./PromotionPicker";
import { buildUpgradeTree } from "./upgradeTree";
import type { CompendiumPiece, CompendiumSquad } from "./Compendium";
import type { CardDefinition, Character, RestOption, UpgradePath } from "./types";

type BoardPiece = { id: number; kind: number; key: string; name: string; description: string; owner: number; cell: number; statuses: string[]; ward: boolean };
type BoardAction = { index: number; target: number; destination: number; capture: boolean; special: boolean; promotion: number | null; route: number[] | null; chance: number | null; blast: number[]; paths: number[][] };
type SandboxCard = { id: number; definition: CardDefinition };
type HandCard = SandboxCard & { targets: number[]; disabled_reason: string | null };
type BoardSnapshot = {
  pieces: BoardPiece[]; actions: BoardAction[]; choice: { name: string; targets: number[]; can_finish: boolean } | null;
  self_destruct: { name: string; blast: number[] } | null; fallout: { cell: number; remaining: number }[];
  events: string[]; deck: SandboxCard[]; hand: HandCard[]; draw_count: number; discard_count: number;
  exhausted_count: number; mana: number;
  relics: string[]; dead_pawns: { id: number; name: string }[];
};
type CardSelection = { target?: number; targets?: number[]; upgrades?: { id: number; target: number }[] };
const squareName = (cell: number) => `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;

function rootsFor(entries: CompendiumSquad[], kind: number, preferred: string) {
  for (const entry of [...entries].sort((a, b) => Number(b.id === preferred) - Number(a.id === preferred))) {
    for (const root of entry.pieces.filter((piece) => piece.starting)) {
      const seen = new Set([root.kind]);
      let size = -1;
      while (size !== seen.size) {
        size = seen.size;
        for (const piece of entry.pieces) if (seen.has(piece.kind)) piece.upgrades.forEach((edge) => seen.add(edge.to));
      }
      if (seen.has(kind)) return { squadId: entry.id, rootKind: root.kind };
    }
  }
  return null;
}

export function PieceSandbox({ choices, entries, onBack }: { choices: Character[]; entries: CompendiumSquad[]; onBack: () => void }) {
  const engine = useRef<WebSandbox | null>(null);
  if (!engine.current) engine.current = new WebSandbox();
  const origins = useRef(new Map<number, { squadId: string; rootKind: number }>());
  const [squadId, setSquadId] = useState(choices[0]?.id ?? "guardian");
  const [rootKind, setRootKind] = useState<number | null>(null);
  const [kind, setKind] = useState<number | null>(null);
  const [owner, setOwner] = useState(0);
  const [placing, setPlacing] = useState(true);
  const [selected, setSelected] = useState<number | null>(null);
  const [abilityMode, setAbilityMode] = useState(false);
  const [detonateMode, setDetonateMode] = useState(false);
  const [hoveredAction, setHoveredAction] = useState<number | null>(null);
  const [revision, setRevision] = useState(0);
  const [error, setError] = useState("");
  const [actionChoice, setActionChoice] = useState<{ from: number; actions: BoardAction[] } | null>(null);
  const [promotion, setPromotion] = useState<{ from: number; actions: BoardAction[] } | null>(null);
  const [cardSquadId, setCardSquadId] = useState(choices[0]?.id ?? "guardian");
  const [activeCardId, setActiveCardId] = useState<number | null>(null);
  const [stagedTargets, setStagedTargets] = useState<number[]>([]);
  const [stagedUpgrades, setStagedUpgrades] = useState<{ id: number; target: number }[]>([]);
  const [upgradeFocus, setUpgradeFocus] = useState<number | null>(null);
  const snapshot: BoardSnapshot = useMemo(() => JSON.parse(engine.current!.snapshot(selected ?? undefined)), [revision, selected]);
  const squad = entries.find((entry) => entry.id === squadId);
  const starters = squad?.pieces.filter((piece, index, all) => piece.starting && all.findIndex((candidate) => candidate.kind === piece.kind) === index) ?? [];
  const root = starters.find((piece) => piece.kind === rootKind) ?? starters[0];
  const chosen = squad?.pieces.find((piece) => piece.kind === kind) ?? root;
  const boardPiece = snapshot.pieces.find((piece) => piece.cell === selected);
  const activeCard = snapshot.hand.find((card) => card.id === activeCardId);
  const cardSquad = entries.find((entry) => entry.id === cardSquadId);
  const paths: UpgradePath[] = useMemo(() => squad?.pieces.flatMap((piece) => piece.upgrades.map((edge) => ({
    ...edge, key: `${edge.from}-${edge.to}`, unlocked: true,
    required_pieces: edge.required_pieces.map((required) => ({ ...required, owned: true })),
  }))) ?? [], [squad]);
  const restPiece: RestOption | null = root ? { id: 0, kind: root.kind, key: root.key, name: root.name,
    description: root.description, cell: null, origin: null, square: "", upgrades: [], ward: false,
    can_upgrade: false, can_revive: false, dead: false, statuses: [] } : null;
  const tree = useMemo(() => restPiece ? buildUpgradeTree(paths, restPiece) : null, [paths, root?.kind]);
  const nodes = tree?.nodes.slice().sort((a, b) => a.depth - b.depth || a.kind - b.kind) ?? [];
  const specialActions = snapshot.actions.filter((action) => action.special);
  const visibleActions = detonateMode ? [] : abilityMode ? specialActions : snapshot.actions;
  const actionTargets = new Map(visibleActions.map((action) => [action.target, action]));
  const focusPiece = snapshot.pieces.find((piece) => piece.id === upgradeFocus);
  const focusDefinition = entries.flatMap((entry) => entry.pieces).find((piece) => piece.kind === focusPiece?.kind);

  function run(operation: () => void, keepSelection = false): boolean {
    try {
      operation(); setRevision((value) => value + 1); setError("");
      if (!keepSelection) { setSelected(null); setAbilityMode(false); setDetonateMode(false); }
      return true;
    } catch (cause) { setError(String(cause)); return false; }
  }
  function cancelCard() { setActiveCardId(null); setStagedTargets([]); setStagedUpgrades([]); setUpgradeFocus(null); }
  function selectBoardPiece(piece: BoardPiece) {
    setSelected(piece.cell); setPlacing(false); setAbilityMode(false); setDetonateMode(false); setError("");
    const origin = origins.current.get(piece.id) ?? rootsFor(entries, piece.kind, squadId);
    if (origin) { setSquadId(origin.squadId); setRootKind(origin.rootKind); setKind(piece.kind); }
  }
  function chooseRoot(piece: CompendiumPiece) {
    setRootKind(piece.kind); setKind(piece.kind); setSelected(null); setPlacing(true); setAbilityMode(false); setDetonateMode(false);
  }
  function chooseVersion(next: number) {
    if (boardPiece) {
      if (!run(() => engine.current!.change_kind(boardPiece.cell, next), true)) return;
      setPlacing(false);
    } else setPlacing(true);
    setKind(next);
  }
  function playCard(card: HandCard, selection: CardSelection) {
    const encoded = { ...selection, upgrades: selection.upgrades?.map(({ id, target }) => [id, target]) };
    if (run(() => engine.current!.play_card(card.id, JSON.stringify(encoded)))) cancelCard();
  }
  function clickCell(cell: number) {
    const piece = snapshot.pieces.find((piece) => piece.cell === cell);
    const targetId = piece?.id ?? 10000 + cell;
    if (snapshot.choice) {
      if (snapshot.choice.targets.includes(targetId)) run(() => engine.current!.resolve_choice(targetId));
      else setError("강조된 효과 대상을 선택하세요.");
      return;
    }
    if (activeCard) {
      if (!activeCard.targets.includes(targetId)) { setError("이 카드는 해당 칸을 대상으로 사용할 수 없습니다."); return; }
      if (activeCard.definition.selection_mode === "single") { playCard(activeCard, { target: targetId }); return; }
      if (activeCard.definition.selection_mode === "multiple") {
        setStagedTargets((old) => old.includes(targetId) ? old.filter((id) => id !== targetId) : [...old, targetId]);
        return;
      }
      if (activeCard.definition.selection_mode === "upgrade") { setUpgradeFocus(targetId); return; }
    }
    if (selected !== null && !placing) {
      if (detonateMode && selected === cell) { run(() => engine.current!.detonate(selected)); return; }
      const actions = visibleActions.filter((action) => action.target === cell);
      if (actions.length) {
        if (actions.length > 1 && actions.some((action) => action.promotion !== null)) setPromotion({ from: selected, actions });
        else if (actions.length > 1) setActionChoice({ from: selected, actions });
        else run(() => engine.current!.act(selected, actions[0].index));
        return;
      }
    }
    if (piece) { selectBoardPiece(piece); return; }
    if (placing && chosen) {
      if (run(() => engine.current!.place(chosen.kind, owner, cell), true)) {
        const placed = (JSON.parse(engine.current!.snapshot(undefined)) as BoardSnapshot).pieces.find((piece) => piece.cell === cell);
        if (placed && root) origins.current.set(placed.id, { squadId, rootKind: root.kind });
        setSelected(cell); setPlacing(false);
      }
    } else setSelected(null);
  }
  function useStagedCard() {
    if (!activeCard) return;
    const mode = activeCard.definition.selection_mode;
    if (mode === "none") playCard(activeCard, {});
    else if (mode === "multiple") playCard(activeCard, { targets: stagedTargets });
    else if (mode === "upgrade") playCard(activeCard, { upgrades: stagedUpgrades });
    else setError("보드에서 대상 기물을 선택하세요.");
  }
  const cardTarget = (piece: BoardPiece | undefined, cell: number) => activeCard?.targets.includes(piece?.id ?? 10000 + cell);
  return <main className="sandbox-page sandbox-v2">
    <div className="sandbox-heading"><div><button className="text-button" onClick={onBack}>← 처음으로</button><h1>기물 테스트장</h1></div><span className="sandbox-note">원정 기록과 분리된 실험 보드</span></div>
    <div className="sandbox-v2-layout">
      <aside className="sandbox-v2-pieces">
        <h2>기물 · 강화</h2>
        <div className="sandbox-mini-tabs" role="group" aria-label="기물 부대">
          {choices.map((choice) => <button key={choice.id} className={squadId === choice.id ? "selected" : ""} onClick={() => {
            setSquadId(choice.id); setRootKind(null); setKind(null); setSelected(null); setPlacing(true);
          }}>{choice.name}</button>)}
        </div>
        <div className="sandbox-v2-starters" aria-label="시작 기물 목록">
          {starters.map((piece) => <button key={piece.kind} className={root?.kind === piece.kind ? "selected" : ""} onClick={() => chooseRoot(piece)}>
            <PieceImage pieceKey={piece.key} name={piece.name} /><span>{piece.name}</span></button>)}
        </div>
        {chosen && <div className="sandbox-v2-piece-info"><PieceImage pieceKey={chosen.key} name={chosen.name} /><div><strong>{chosen.name}</strong><p>{chosen.description}</p></div></div>}
        <div className="sandbox-v2-tree" aria-label="강화 트리">
          <strong>강화 경로</strong>
          {nodes.map((node) => <button key={node.kind} className={chosen?.kind === node.kind ? "selected" : ""}
            style={{ marginLeft: Math.min(node.depth * 12, 36) }} onClick={() => chooseVersion(node.kind)} aria-pressed={chosen?.kind === node.kind}>
            <PieceImage pieceKey={node.key} name={node.name} /><span>{node.name}<small>{node.depth === 0 ? "기본" : `${node.depth}단계`}</small></span></button>)}
          {!nodes.length && <small>강화 경로가 없습니다.</small>}
        </div>
      </aside>
      <section className="sandbox-v2-center">
        <div className="sandbox-v2-toolbar"><div role="group" aria-label="배치 진영"><button className={owner === 0 ? "selected" : ""} onClick={() => { setOwner(0); setPlacing(true); setSelected(null); }}>백 배치</button><button className={owner === 1 ? "selected" : ""} onClick={() => { setOwner(1); setPlacing(true); setSelected(null); }}>흑 배치</button></div>
          <button className={placing ? "selected" : ""} onClick={() => { setPlacing(true); setSelected(null); }}>기물 배치</button>
          <button onClick={() => run(() => engine.current!.progress_effects(), true)}>효과 진행</button>
          <button onClick={() => { if (run(() => engine.current!.clear())) origins.current.clear(); }}>보드 초기화</button></div>
        <div className="sandbox-v2-board board" role="group" aria-label="기물 테스트 체스판">
          {Array.from({ length: 64 }, (_, cell) => {
            const piece = snapshot.pieces.find((piece) => piece.cell === cell);
            const action = actionTargets.get(cell);
            const preview = detonateMode ? snapshot.self_destruct : visibleActions.find((candidate) => candidate.target === hoveredAction);
            const fallout = snapshot.fallout.find((effect) => effect.cell === cell);
            const pathIndex = "paths" in (preview ?? {}) ? (preview as BoardAction).paths.findIndex((path) => path.includes(cell)) : -1;
            const cardCandidate = cardTarget(piece, cell);
            const effectCandidate = snapshot.choice?.targets.includes(piece?.id ?? 10000 + cell);
            return <button key={cell} data-square={squareName(cell)} data-legal={!!action || undefined}
              className={`square ${(cell + Math.floor(cell / 8)) % 2 ? "dark" : "light"} ${selected === cell ? "selected" : ""} ${action || (detonateMode && selected === cell) ? "legal" : ""} ${cardCandidate || effectCandidate ? "targetable" : ""} ${fallout ? "fallout-cell" : ""} ${preview?.blast.includes(cell) ? "blast-preview" : ""} ${pathIndex >= 0 ? `pierce-path path-${pathIndex}` : ""}`}
              aria-label={`${squareName(cell)} ${piece ? `${piece.owner === 0 ? "백" : "흑"} ${piece.name}` : "빈 칸"}${fallout ? ` 방사성 낙진 ${fallout.remaining}라운드` : ""}${action ? action.capture ? " 포획 가능" : " 이동 가능" : ""}`}
              onMouseEnter={() => { if (action) setHoveredAction(cell); }}
              onMouseLeave={() => { if (hoveredAction === cell) setHoveredAction(null); }}
              onClick={() => clickCell(cell)}>
              {action && <span className={action.capture ? "capture-dot" : "move-dot"} />}
              {detonateMode && selected === cell && <span className="aim-target">자폭</span>}
              {piece && <PieceImage pieceKey={piece.key} owner={piece.owner} name={piece.name} />}
              {piece?.ward && <span className="sandbox-v2-ward" title="가호">✦</span>}
              {fallout && <span className="fallout-badge" title={`방사성 낙진 · 남은 ${fallout.remaining}라운드`}>{fallout.remaining}</span>}
            </button>;
          })}
        </div>
        <div className="sandbox-v2-selection">
          {snapshot.choice ? <><strong>{snapshot.choice.name}</strong>{snapshot.choice.can_finish && <button onClick={() => run(() => engine.current!.resolve_choice(0))}>능력 완료</button>}</> :
            boardPiece ? <><strong>{boardPiece.owner === 0 ? "백" : "흑"} {boardPiece.name} · {squareName(boardPiece.cell)}</strong>
              <span>{[boardPiece.ward ? "가호" : "", ...boardPiece.statuses].filter(Boolean).join(" · ") || (snapshot.actions.length ? "표시된 칸으로 이동할 수 있습니다." : "현재 배치에서 가능한 행동이 없습니다.")}</span>
              {specialActions.length > 0 && <button className={abilityMode ? "selected" : ""} onClick={() => { setAbilityMode(!abilityMode); setDetonateMode(false); }}>특수 능력 · {boardPiece.key === "machine_gun_drone" ? "기관총 사격" : boardPiece.key === "grenade_launcher_drone" ? "유탄 발사" : specialActions.some((action) => action.chance !== null) ? "사격" : "대상 선택"} · {specialActions.length}곳</button>}
              {snapshot.self_destruct && <button className={detonateMode ? "selected" : ""} onClick={() => { setDetonateMode(!detonateMode); setAbilityMode(false); }}>{snapshot.self_destruct.name} · 범위 보기</button>}
              <button onClick={() => run(() => engine.current!.remove(boardPiece.cell))}>기물 제거</button></> :
              <span>{placing ? `${chosen?.name ?? "기물"}을 배치할 빈 칸을 선택하세요.` : "기물을 선택하면 합법적인 행동이 표시됩니다."}</span>}
        </div>
        {error && <p className="sandbox-v2-error" role="alert">{error}</p>}
        {snapshot.events.length > 0 && <p className="sandbox-v2-event" role="status">{snapshot.events.at(-1)}</p>}
        <section className="sandbox-v2-hand"><div className="sandbox-v2-section-title"><h2>손패</h2><span>마나 <strong>{snapshot.mana}</strong> · 드로우 {snapshot.draw_count} · 버림 {snapshot.discard_count} · 소멸 {snapshot.exhausted_count}</span></div>
          <div className="sandbox-v2-hand-tools"><button onClick={() => run(() => engine.current!.draw_card(), true)}>1장 드로우</button><button onClick={() => { run(() => engine.current!.redraw_hand(), true); cancelCard(); }}>손패 다시 뽑기</button><button onClick={() => run(() => engine.current!.set_mana(snapshot.mana + 3), true)}>마나 +3</button><label>마나 <input type="number" min="0" max="9999" value={snapshot.mana} onChange={(event) => run(() => engine.current!.set_mana(Number(event.target.value) || 0), true)} /></label></div>
          <div className="sandbox-v2-hand-list">{snapshot.hand.map((card) => <button key={card.id} className={`sandbox-v2-hand-card ${activeCardId === card.id ? "selected" : ""}`} onClick={() => {
            if (card.disabled_reason) { setError(card.disabled_reason); return; }
            setActiveCardId(card.id); setStagedTargets([]); setStagedUpgrades([]); setUpgradeFocus(null); setAbilityMode(false);
          }}><CardFace card={card.definition} /></button>)}{!snapshot.hand.length && <span className="sandbox-v2-empty">손패가 비어 있습니다.</span>}</div>
          {activeCard && <div className="sandbox-v2-card-action"><strong>{activeCard.definition.name}</strong><span>{activeCard.definition.selection_mode === "none" ? "즉시 사용" : activeCard.definition.selection_mode === "single" ? "보드에서 대상 선택" : activeCard.definition.selection_mode === "upgrade" ? "보드에서 기물 선택 → 강화 단계 선택" : `대상 ${stagedTargets.length}개 선택`}</span>
            <button onClick={useStagedCard}>카드 사용</button><button onClick={cancelCard}>취소</button></div>}
          {activeCard?.definition.kind === "emergency_conscription" && <div className="sandbox-v2-card-action"><strong>부활할 폰 선택</strong>{snapshot.dead_pawns.filter((pawn) => activeCard.targets.includes(pawn.id)).map((pawn) => <button key={pawn.id} className={stagedTargets.includes(pawn.id) ? "selected" : ""} onClick={() => setStagedTargets((old) => old.filter((id) => !snapshot.dead_pawns.some((dead) => dead.id === id)).concat(pawn.id))}>{pawn.name} #{pawn.id}</button>)}</div>}
          {activeCard?.definition.selection_mode === "upgrade" && focusPiece && <div className="sandbox-v2-upgrade-choice"><strong>{focusPiece.name} 강화 선택</strong>{focusDefinition?.upgrades.map((edge) => <button key={edge.to} onClick={() => { setStagedUpgrades((old) => [...old.filter((choice) => choice.id !== focusPiece.id), { id: focusPiece.id, target: edge.to }]); setUpgradeFocus(null); }}>{edge.to_name}</button>)}</div>}
          {stagedUpgrades.length > 0 && <p className="sandbox-v2-staged">선택한 강화: {stagedUpgrades.map((choice) => `${snapshot.pieces.find((piece) => piece.id === choice.id)?.name ?? "기물"} → ${entries.flatMap((entry) => entry.pieces).find((piece) => piece.kind === choice.target)?.name ?? "강화"}`).join(", ")}</p>}
        </section>
      </section>
      <aside className="sandbox-v2-cards"><h2>카드 덱 편집</h2><div className="sandbox-mini-tabs" role="group" aria-label="카드 부대">{choices.map((choice) => <button key={choice.id} className={cardSquadId === choice.id ? "selected" : ""} onClick={() => setCardSquadId(choice.id)}>{choice.name}</button>)}</div>
        <h3>유물 시험</h3><div className="sandbox-v2-deck-list">{cardSquad?.relics.map((relic) => <label key={relic.id}><input type="checkbox" checked={snapshot.relics.includes(relic.id)} onChange={(event) => run(() => engine.current!.set_relic(relic.id, event.target.checked), true)} />{relic.name}<small>{relic.description}</small></label>)}</div>
        <div className="sandbox-v2-card-catalog">{cardSquad?.cards.map(({ definition }) => <div key={definition.kind} className="sandbox-v2-catalog-card"><button onClick={() => run(() => engine.current!.add_card(definition.kind), true)}><strong>{definition.name}</strong><small>비용 {definition.cost_per_target ? "N" : definition.cost}</small><span>{definition.description}</span></button><button title={`${definition.name} 손패에 직접 추가`} aria-label={`${definition.name} 손패에 직접 추가`} onClick={() => run(() => engine.current!.add_to_hand(definition.kind), true)}>손패 +</button></div>)}</div>
        <div className="sandbox-v2-deck-head"><h3>테스트 덱 · {snapshot.deck.length}장</h3><button onClick={() => { run(() => engine.current!.clear_deck(), true); cancelCard(); }}>덱 비우기</button><button onClick={() => { run(() => engine.current!.reset_cards(), true); cancelCard(); }}>덱 다시 뽑기</button></div>
        <div className="sandbox-v2-deck-list">{snapshot.deck.map((card) => <div key={card.id}><span>{card.definition.name}</span><button aria-label={`${card.definition.name} 덱에서 제거`} onClick={() => run(() => engine.current!.remove_card(card.id), true)}>×</button></div>)}{!snapshot.deck.length && <span className="sandbox-v2-empty">카드를 클릭해 덱에 추가하세요.</span>}</div>
      </aside>
    </div>
    {promotion && <div className="sandbox-promotion"><PromotionPicker square={squareName(promotion.actions[0].target)} options={promotion.actions.filter((action) => action.promotion !== null).map((action) => {
      const piece = entries.flatMap((entry) => entry.pieces).find((piece) => piece.kind === action.promotion);
      return { kind: action.promotion!, key: piece?.key ?? "pawn", name: piece?.name ?? "승격", description: piece?.description ?? "" };
    })} onChoose={(next) => { const action = promotion.actions.find((action) => action.promotion === next)!;
      run(() => engine.current!.act(promotion.from, action.index)); setPromotion(null); }} onCancel={() => setPromotion(null)} /></div>}
    {actionChoice && <div className="sandbox-promotion"><section className="sandbox-action-picker" role="dialog" aria-label="행동 선택"><div><strong>{squareName(actionChoice.actions[0].target)} · 행동 선택</strong><button onClick={() => setActionChoice(null)}>취소</button></div>
      {actionChoice.actions.map((action, order) => <button key={action.index} onClick={() => { run(() => engine.current!.act(actionChoice.from, action.index)); setActionChoice(null); }}>{order + 1}. {action.special ? "제자리 특수 행동" : action.capture ? "이동·포획" : "이동"}{action.chance !== null ? ` · 명중률 ${action.chance}%` : ""}{action.route?.length ? ` · ${action.route.map(squareName).join(" → ")}` : ""}</button>)}
    </section></div>}
  </main>;
}

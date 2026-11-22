import { useMemo, useRef, useState } from "react";
import { WebSandbox } from "./wasm/chesslayer_web";
import { ChessBoard, BoardSquare, squareName } from "./board/ChessBoard";
import { PieceImage } from "./PieceImage";
import { PromotionPicker } from "./PromotionPicker";
import { buildUpgradeTree } from "./upgradeTree";
import type { CompendiumPiece, CompendiumSquad } from "./Compendium";
import type { RestOption, UpgradePath } from "./types";

type BoardPiece = { id: number; kind: number; key: string; name: string; description: string; owner: number; cell: number; statuses: string[]; dice_roll: number | null; ward: boolean };
type BoardAction = { index: number; target: number; destination: number; capture: boolean; special: boolean; promotion: number | null; route: number[] | null; chance: number | null; blast: number[]; paths: number[][] };
type BoardSnapshot = {
  pieces: BoardPiece[]; actions: BoardAction[]; choice: { name: string; targets: number[]; can_finish: boolean } | null;
  self_destruct: { name: string; blast: number[] } | null; fallout: { cell: number; remaining: number }[];
  events: string[]; relics: string[]; mana: number; ammo: number;
};

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
  return { squadId: "avenger", rootKind: kind };
}

export function PieceSandbox({ entries, onBack }: { entries: CompendiumSquad[]; onBack: () => void }) {
  const engine = useRef<WebSandbox | null>(null);
  if (!engine.current) engine.current = new WebSandbox();
  const origins = useRef(new Map<number, { squadId: string; rootKind: number }>());
  const squadId = "avenger";
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
  const snapshot: BoardSnapshot = useMemo(() => JSON.parse(engine.current!.snapshot(selected ?? undefined)), [revision, selected]);
  const squad = entries.find((entry) => entry.id === squadId);
  const starters = squad?.pieces ?? [];
  const root = starters.find((piece) => piece.kind === rootKind) ?? starters[0];
  const chosen = squad?.pieces.find((piece) => piece.kind === kind) ?? root;
  const boardPiece = snapshot.pieces.find((piece) => piece.cell === selected);
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


  function run(operation: () => void, keepSelection = false): boolean {
    try {
      operation(); setRevision((value) => value + 1); setError("");
      if (!keepSelection) { setSelected(null); setAbilityMode(false); setDetonateMode(false); }
      return true;
    } catch (cause) { setError(String(cause)); return false; }
  }
  function selectBoardPiece(piece: BoardPiece) {
    setSelected(piece.cell); setPlacing(false); setAbilityMode(false); setDetonateMode(false); setError("");
    const origin = origins.current.get(piece.id) ?? rootsFor(entries, piece.kind, squadId);
    if (origin) { setRootKind(origin.rootKind); setKind(piece.kind); }
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
  function clickCell(cell: number) {
    const piece = snapshot.pieces.find((piece) => piece.cell === cell);
    const targetId = piece?.id ?? 10000 + cell;
    if (snapshot.choice) {
      if (snapshot.choice.targets.includes(targetId)) run(() => engine.current!.resolve_choice(targetId));
      else setError("강조된 효과 대상을 선택하세요.");
      return;
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
  return <main className="sandbox-page sandbox-v2">
    <div className="sandbox-heading"><div><button className="text-button" onClick={onBack}>← 처음으로</button><h1>기물 테스트장</h1></div><span className="sandbox-note">원정 기록과 분리된 실험 보드</span></div>
    <div className="sandbox-v2-layout">
      <aside className="sandbox-v2-pieces">
        <h2>기물 · 강화</h2>
        <div className="sandbox-v2-starters" aria-label="모든 기물 목록">
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
          <span className="sandbox-resources">마나 {snapshot.mana} · 포탄량 {snapshot.ammo} / 10</span>
          <button className={placing ? "selected" : ""} onClick={() => { setPlacing(true); setSelected(null); }}>기물 배치</button>
          <button onClick={() => run(() => engine.current!.progress_effects(), true)}>효과 진행</button>
          <button onClick={() => { if (run(() => engine.current!.clear())) origins.current.clear(); }}>보드 초기화</button></div>
        <ChessBoard className="sandbox-v2-board" role="group" aria-label="기물 테스트 체스판"
          renderSquare={(cell) => {
            const piece = snapshot.pieces.find((piece) => piece.cell === cell);
            const action = actionTargets.get(cell);
            const preview = detonateMode ? snapshot.self_destruct : visibleActions.find((candidate) => candidate.target === hoveredAction);
            const fallout = snapshot.fallout.find((effect) => effect.cell === cell);
            const pathIndex = "paths" in (preview ?? {}) ? (preview as BoardAction).paths.findIndex((path) => path.includes(cell)) : -1;
            const effectCandidate = snapshot.choice?.targets.includes(piece?.id ?? 10000 + cell);
            return <BoardSquare cell={cell} key={cell} data-legal={!!action || undefined}
              className={`${selected === cell ? "selected" : ""} ${action || (detonateMode && selected === cell) ? "legal" : ""} ${effectCandidate ? "targetable" : ""} ${fallout ? "fallout-cell" : ""} ${preview?.blast.includes(cell) ? "blast-preview" : ""} ${pathIndex >= 0 ? `pierce-path path-${pathIndex}` : ""}`}
              aria-label={`${squareName(cell)} ${piece ? `${piece.owner === 0 ? "백" : "흑"} ${piece.name}${piece.dice_roll ? ` 주사위 ${piece.dice_roll}` : ""}` : "빈 칸"}${fallout ? ` 방사성 낙진 ${fallout.remaining}라운드` : ""}${action ? action.capture ? " 포획 가능" : " 이동 가능" : ""}`}
              onMouseEnter={() => { if (action) setHoveredAction(cell); }}
              onMouseLeave={() => { if (hoveredAction === cell) setHoveredAction(null); }}
              onClick={() => clickCell(cell)}>
              {action && <span className={action.capture ? "capture-dot" : "move-dot"} />}
              {detonateMode && selected === cell && <span className="aim-target">자폭</span>}
              {piece && <PieceImage pieceKey={piece.key} owner={piece.owner} name={piece.name} />}
              {piece?.dice_roll && <span className="dice-mark" title={`주사위 ${piece.dice_roll} · 현재 행마`}>🎲{piece.dice_roll}</span>}
              {piece?.ward && <span className="sandbox-v2-ward" title="가호">✦</span>}
              {fallout && <span className="fallout-badge" title={`방사성 낙진 · 남은 ${fallout.remaining}라운드`}>{fallout.remaining}</span>}
            </BoardSquare>;
          }}
        />
        <div className="sandbox-v2-selection">
          {snapshot.choice ? <><strong>{snapshot.choice.name}</strong>{snapshot.choice.can_finish && <button onClick={() => run(() => engine.current!.resolve_choice(0))}>능력 완료</button>}</> :
            boardPiece ? <><strong>{boardPiece.owner === 0 ? "백" : "흑"} {boardPiece.name} · {squareName(boardPiece.cell)}</strong>
              <span>{[boardPiece.dice_roll ? `🎲 ${boardPiece.dice_roll} · ${["폰", "나이트", "비숍", "룩", "퀸", "킹"][boardPiece.dice_roll - 1]} 행마` : "", boardPiece.ward ? "가호" : "", ...boardPiece.statuses].filter(Boolean).join(" · ") || (snapshot.actions.length ? "표시된 칸으로 이동할 수 있습니다." : "현재 배치에서 가능한 행동이 없습니다.")}</span>
              {specialActions.length > 0 && <button className={abilityMode ? "selected" : ""} onClick={() => { setAbilityMode(!abilityMode); setDetonateMode(false); }}>특수 능력 · {boardPiece.key === "machine_gun_drone" ? "기관총 사격" : boardPiece.key === "grenade_launcher_drone" ? "유탄 발사" : specialActions.some((action) => action.chance !== null) ? "사격" : "대상 선택"} · {specialActions.length}곳</button>}
              {snapshot.self_destruct && <button className={detonateMode ? "selected" : ""} onClick={() => { setDetonateMode(!detonateMode); setAbilityMode(false); }}>{snapshot.self_destruct.name} · 범위 보기</button>}
              <button onClick={() => run(() => engine.current!.remove(boardPiece.cell))}>기물 제거</button></> :
              <span>{placing ? `${chosen?.name ?? "기물"}을 배치할 빈 칸을 선택하세요.` : "기물을 선택하면 합법적인 행동이 표시됩니다."}</span>}
        </div>
        {error && <p className="sandbox-v2-error" role="alert">{error}</p>}
        {snapshot.events.length > 0 && <p className="sandbox-v2-event" role="status">{snapshot.events.at(-1)}</p>}
      </section>
      <aside className="sandbox-v2-cards"><h2>유물 시험</h2><div className="sandbox-v2-deck-list">{squad?.relics.map((relic) => <label key={relic.id}><input type="checkbox" checked={snapshot.relics.includes(relic.id)} onChange={(event) => run(() => engine.current!.set_relic(relic.id, event.target.checked), true)} />{relic.name}<small>{relic.description}</small></label>)}</div></aside>
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

import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { createPortal } from "react-dom";
import init, { WebGame, catalog } from "./wasm/chesslayer_web";
import type {
  Card,
  Squad,
  Command,
  EnemyInspection,
  Piece,
  Promotion,
  Snapshot,
} from "./types";
import "./style.css";
import { ActMap, nodeTypes } from "./ActMap";
import { PieceImage } from "./PieceImage";
import { RestPanel, type RestChoice } from "./RestPanel";
import { PromotionPicker } from "./PromotionPicker";
import { CardUpgradePanel } from "./CardUpgradePanel";
import { RelicIcon } from "./ContentArt";
import { CardFace } from "./CardFace";
import { BattleRewards } from "./BattleRewards";
import { ExpeditionStatus } from "./ExpeditionStatus";
import { version } from "../package.json";

// Keep the original storage slot so legacy saves receive the WASM version notice.
const META_KEY = "chesslayer.progress.v1";
const SAVE_KEY = "chesslayer.run.v1";
const starterRelicNames: Record<Squad["relic"], string> = {
  agility: "민첩성",
  reinforcements: "충원",
  carnival_flare: "카니발의 신호탄",
  silver_cutlery: "은제 식기",
  empty_goblet: "빈 술잔",
  torn_menu: "찢어진 메뉴판",
  last_plate: "마지막 접시",
  golden_tray: "금박 쟁반",
  laughing_mask: "웃는 가면",
  eternal_bell: "영원한 향연의 종",
  supply_marker: "보급 표식",
  special_insignia: "특수부대 표식",
  haste: "가속", whirlwind: "소용돌이", self_defense: "호신용품",
  kings_indian: "인디언 킹의 전략책", queens_indian: "인디언 퀸의 전략책",
  hedgehog: "고슴도치의 지혜", initiative: "선공권", ice_cream: "아이스크림",
};

function statusText(piece:Piece) {
  const s=piece.status;
  return [s.agile && `민첩 ${s.agile}`,s.weak && `약화 ${s.weak}`,s.hunger && ["정상","공복","굶주림","기아"][s.hunger],s.madness && `광기 ${s.madness}`,s.guilt && `죄책감 ${s.guilt}`,s.blessed && "황충의 축복",s.ingredient && "식재료",s.disabled && "영구 전투불능",s.etiquette && "식사의 예법",piece.bell_target && "종소리 대상 · 다음 수 허기 악화"].filter(Boolean).join(" · ");
}

function BoardPiece({
  piece,
  focused = false,
  previewKey,
}: {
  piece: Piece;
  focused?: boolean;
  previewKey?: string;
}) {
  return (
    <span
      className={`piece owner-${piece.owner} ${piece.ward ? "warded" : ""} ${focused ? "rest-focused" : ""}`}
      style={{
        left: `${(piece.cell % 8) * 12.5}%`,
        top: `${Math.floor(piece.cell / 8) * 12.5}%`,
      }}
      data-piece-id={piece.id}
      data-cell={piece.cell}
      aria-hidden="true"
    >
      <PieceImage pieceKey={previewKey || piece.key} owner={piece.owner} />
      {piece.status.disabled ? <span className="status-badge danger">불능</span> : piece.status.agile>0 ? <span className="status-badge">+{piece.status.agile}</span> : piece.status.hunger>0 ? <span className="status-badge hunger">{["","공복","굶주림","기아"][piece.status.hunger]}</span> : null}
      {piece.bell_target && <span className="bell-mark">♧</span>}
      {piece.royal && <span className="royal-mark">★</span>}
      {piece.ward && <span className="ward-mark" title="가호">✦</span>}
      {piece.temporary_upgrade && <span className="temporary-mark" title="일시 강화 · 이번 전투 한정">↑</span>}
      {piece.self_defense && <span className="self-defense-mark">◆</span>}
      {piece.limiter && <span className="limiter-mark" title={`리미터 해제: ${piece.limiter.active ? `남은 이동 ${piece.limiter.moves_left}회` : "다음 소속 턴에 3회"} · 턴 종료 후 사망`}>{piece.limiter.active ? piece.limiter.moves_left : "!"}</span>}
    </span>
  );
}
function App() {
  const engine = useRef<WebGame | null>(null);
  const [ready, setReady] = useState(false);
  const [choices, setChoices] = useState<Squad[]>([]);
  const [screen, setScreen] = useState<"home" | "choose" | "game">("home");
  const [cleared,setCleared]=useState(0);
  const [floor,setFloor]=useState(1);
  const [abilityId,setAbilityId]=useState<number|null>(null);
  const [placementPick,setPlacementPick]=useState<number|null>(null);
  const [placement,setPlacement]=useState<{id:number;cell:number}[]>([]);
  const [squad, setSquad] = useState("guardian");
  const [state, setState] = useState<Snapshot | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [enemyInspection, setEnemyInspection] = useState<EnemyInspection | null>(null);
  const [restChoice, setRestChoice] = useState<RestChoice | null>(null);
  const [rewardUpgradeOpen, setRewardUpgradeOpen] = useState(false);
  const [upgradeHistory, setUpgradeHistory] = useState<Extract<Command, { type: "rest_upgrade" }>[]>([]);
  const [rewardUpgradeHistory, setRewardUpgradeHistory] = useState<{ id: number; target: number }[]>([]);
  const [promotion, setPromotion] = useState<{
    from: number;
    to: number;
    options: Promotion[];
  } | null>(null);
  const [hasSave, setHasSave] = useState(false);
  const [saveNotice, setSaveNotice] = useState("");
  const [targetCard, setTargetCard] = useState<Card | null>(null);
  const [cardTargets, setCardTargets] = useState<number[]>([]);
  const [cardUpgrades, setCardUpgrades] = useState<{ id: number; target: number }[]>([]);
  const [upgradeFocus, setUpgradeFocus] = useState<number | null>(null);
  const [error, setError] = useState("");
  const [help, setHelp] = useState(false);
  const [statusOpen, setStatusOpen] = useState(false);
  const closeStatus = useCallback(() => setStatusOpen(false), []);
  const [floorTransition, setFloorTransition] = useState<{ next: Snapshot; stage: "out" | "in" } | null>(null);
  const logRef = useRef<HTMLDivElement>(null);
  const boardRef = useRef<HTMLDivElement>(null);
  const outcomeRef = useRef<HTMLElement>(null);
  const previousPhase = useRef<string | undefined>(undefined);
  useEffect(() => {
    let active = true;
    init()
      .then(() => {
        if (active) {
          setChoices(JSON.parse(catalog()));
          setReady(true);
          try {
            setHasSave(!!localStorage.getItem(SAVE_KEY));
            const meta=JSON.parse(localStorage.getItem(META_KEY)||"{}");
            setCleared(Number.isInteger(meta.cleared)?meta.cleared & 15:0);
          } catch {
            setSaveNotice(
              "브라우저 저장소를 사용할 수 없어 이번 원정은 저장되지 않습니다.",
            );
          }
        }
      })
      .catch((e) => setError(`게임 엔진을 불러오지 못했습니다. ${String(e)}`));
    return () => {
      active = false;
    };
  }, []);
  // A new run frees the previous engine in start(); avoid freeing live state during Vite hot reload.
  // The browser releases the WASM instance when this document closes.
  useEffect(() => {
    if (state?.phase && state.phase !== previousPhase.current) {
      if (state.phase !== "combat" && state.phase !== "rest")
        outcomeRef.current?.scrollIntoView({
          behavior: "smooth",
          block: "start",
        });
      else if (previousPhase.current)
        boardRef.current?.scrollIntoView({
          behavior: "smooth",
          block: "center",
        });
    }
    previousPhase.current = state?.phase;
  }, [state?.phase]);
  useEffect(() => {
    if (targetCard)
      boardRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, [targetCard]);
  const send = useCallback((command: Command) => {
    if (!engine.current) return;
    try {
      const before: Snapshot = JSON.parse(engine.current.snapshot());
      const next: Snapshot = JSON.parse(engine.current.dispatch(JSON.stringify(command)));
      if ((command.type === "choose_reward" || command.type === "continue_reward" || command.type === "advance_floor") && before.floor === 1 && next.floor === 2 && next.phase === "map" && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
        setFloorTransition({ next, stage: "out" });
      } else {
        setState(next);
      }
      persist(engine.current);
      setRestChoice(null);
      if (command.type === "claim_upgrade") setRewardUpgradeOpen(false);
      setAbilityId(null);setPlacementPick(null);setPlacement([]);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setTargetCard(null);
      setCardTargets([]);
      setCardUpgrades([]);
      setUpgradeFocus(null);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  }, []);
  useEffect(() => {
    if (!floorTransition) return;
    const timer = window.setTimeout(() => {
      if (floorTransition.stage === "out") {
        setState(floorTransition.next);
        setScreen("game");
        setFloorTransition({ ...floorTransition, stage: "in" });
      } else setFloorTransition(null);
    }, floorTransition.stage === "out" ? 600 : 1100);
    return () => window.clearTimeout(timer);
  }, [floorTransition]);
  const skipFloorTransition = () => {
    if (!floorTransition) return;
    setState(floorTransition.next);
    setScreen("game");
    setFloorTransition(null);
  };
  useEffect(() => {
    if (!floorTransition) return;
    const escape = (event: KeyboardEvent) => { if (event.key === "Escape") skipFloorTransition(); };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, [floorTransition]);
  useEffect(() => {
    if (screen !== "game" || state?.phase !== "combat" || state.turn !== 1)
      return;
    const timer = setTimeout(() => send({ type: "enemy_step" }), 650);
    return () => clearTimeout(timer);
  }, [screen, state, send]);
  useEffect(() => {
    logRef.current?.scrollTo({
      top: logRef.current.scrollHeight,
      behavior: "smooth",
    });
  }, [state?.log]);
  useEffect(() => {
    const cancel = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (statusOpen) return;
        if (rewardUpgradeOpen) {
          setRewardUpgradeOpen(false);
          setRestChoice(null);
          return;
        }
        setSelected(null);
        setEnemyInspection(null);
        setRestChoice(null);
        setPromotion(null);
        setTargetCard(null);
        setCardTargets([]);
        setCardUpgrades([]);
        setUpgradeFocus(null);
        setError("");
      }
    };
    window.addEventListener("keydown", cancel);
    return () => window.removeEventListener("keydown", cancel);
  }, [statusOpen, rewardUpgradeOpen]);
  function persist(game: WebGame) {
    const saved = game.save();
    // Read the existing command history for presentation only. Promotions are
    // not rest upgrades and must not illuminate an unrelated upgrade edge.
    const commands: Command[] = JSON.parse(saved).commands;
    setUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" }> => c.type === "rest_upgrade"));
    setRewardUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" | "claim_upgrade" }> => c.type === "rest_upgrade" || c.type === "claim_upgrade").map(({ id, target }) => ({ id, target })));
    try {
      localStorage.setItem(SAVE_KEY, saved);
      const snapshot:Snapshot=JSON.parse(game.snapshot());
      if(snapshot.phase==="victory") {
        const old=JSON.parse(localStorage.getItem(META_KEY)||"{}");
        const progress=((Number.isInteger(old.cleared)?old.cleared:0)|(1<<(snapshot.floor-1))) & 15;
        localStorage.setItem(META_KEY,JSON.stringify({version:1,cleared:progress}));setCleared(progress);
      }
      setHasSave(true);
      setSaveNotice("");
    } catch {
      setSaveNotice(
        "진행 상태를 저장하지 못했습니다. 이 탭을 닫으면 최근 진행을 잃을 수 있습니다.",
      );
    }
  }
  function resume() {
    try {
      const saved = localStorage.getItem(SAVE_KEY);
      if (!saved) return;
      const next = WebGame.restore(saved);
      engine.current?.free();
      engine.current = next;
      setState(JSON.parse(next.snapshot()));
      setFloorTransition(null);
      persist(next);
      setRestChoice(null);
      setRewardUpgradeOpen(false);
      setAbilityId(null);setPlacementPick(null);setPlacement([]);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setTargetCard(null);
      setCardTargets([]);
      setCardUpgrades([]);
      setUpgradeFocus(null);
      setStatusOpen(false);
      setError("");
      setScreen("game");
    } catch (e) {
      setError(String(e));
    }
  }
  function unlockAll() {
    setCleared(15);
    try {
      localStorage.setItem(META_KEY, JSON.stringify({ version: 1, cleared: 15 }));
      setSaveNotice("");
    } catch {
      setSaveNotice("모든 부대와 층을 열었습니다. 브라우저 저장소를 사용할 수 없어 새로고침 후에는 다시 눌러 주세요.");
    }
  }
  function start() {
    try {
      // Entropy only comes from the browser; the seeded Rust engine generates the entire Act.
      const seed = crypto.getRandomValues(new Uint32Array(1))[0];
      const next = WebGame.start(squad, seed, floor, cleared);
      engine.current?.free();
      engine.current = next;
      const snapshot: Snapshot = JSON.parse(next.snapshot());
      if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
        setState(snapshot);
        setFloorTransition(null);
        setScreen("game");
      } else {
        setFloorTransition({ next: snapshot, stage: "out" });
      }
      persist(next);
      setRestChoice(null);
      setRewardUpgradeOpen(false);
      setAbilityId(null);setPlacementPick(null);setPlacement([]);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setTargetCard(null);
      setCardTargets([]);
      setCardUpgrades([]);
      setUpgradeFocus(null);
      setStatusOpen(false);
      setError("");
    } catch (e) {
      setError(String(e));
    }
  }
  function chooseCard(card: Card) {
    if (promotion || state?.effect_choice) return;
    setAbilityId(null);setPlacementPick(null);setPlacement([]);
    if (card.disabled_reason) {
      setError(card.disabled_reason);
      return;
    }
    setError("");
    setSelected(null);
    setEnemyInspection(null);
    setCardTargets([]);
    setCardUpgrades([]);
    setUpgradeFocus(null);
    if (card.definition.targeted || card.definition.selection_mode==="square") setTargetCard(card);
    else send({ type: "play_card", id: card.id });
  }
  function clickSquare(cell: number) {
    if (!state || promotion) return;
    if (rewardUpgradeOpen) {
      const boardPiece = boardPieces.find((p) => p.cell === cell);
      const ally = state.rest_options.find((p) => p.can_upgrade && p.id === boardPiece?.id);
      if (ally) setRestChoice({ cell, id: ally.id });
      return;
    }
    if (state.phase === "rest") {
      const ally = state.rest_options.find((p) => !p.dead && p.cell === cell);
      const occupant = state.pieces.find((p) => p.cell === cell);
      const candidate =
        !occupant &&
        state.rest_options.find((p) => p.can_revive && p.origin === cell);
      const option = ally || candidate;
      if (option) setRestChoice({ cell, id: option.id });
      return;
    }
    if (state.phase !== "combat") return;
    const piece = state.pieces.find((p) => p.cell === cell);
    if (state.effect_choice) {
      if(state.effect_choice.kind==="move_corpse" && state.effect_choice.cells.includes(cell)) send({type:"resolve_effect",cell});
      else if(piece && state.effect_choice.targets.includes(piece.id))send({type:"resolve_effect",target:piece.id});
      return;
    }
    if(abilityId!==null){
      if(piece && state.abilities.find(a=>a.id===abilityId)?.targets.includes(piece.id))send({type:"ability",id:abilityId,target:piece.id});
      return;
    }
    if (targetCard) {
      if(targetCard.definition.selection_mode==="square"){
        if(!piece && cell>=48)send({type:"tactical_card",id:targetCard.id,targets:[],cells:[cell]});
        return;
      }
      if(targetCard.definition.selection_mode==="placement"){
        if(placementPick!==null){
          if(piece?.owner===1)return;
          setPlacement(old=>[...old.filter(p=>p.id!==placementPick),{id:placementPick,cell}]);setPlacementPick(null);
        }else if(piece && targetCard.targets.includes(piece.id)){setPlacementPick(piece.id);}
        return;
      }
      if (piece && targetCard.targets.includes(piece.id)) {
        if (targetCard.definition.selection_mode === "upgrade") {
          if (!cardUpgrades.some((u) => u.id === piece.id) && cardUpgrades.length >= state.mana) {
            setError("남은 마나보다 많은 기물을 선택할 수 없습니다.");
            return;
          }
          setUpgradeFocus(piece.id);
        } else if (targetCard.definition.selection_mode === "multiple") {
          setCardTargets((ids) => ids.includes(piece.id) ? ids.filter((id) => id !== piece.id) : [...ids, piece.id]);
        } else if (["limiter_release", "sacred_domain"].includes(targetCard.definition.kind)) {
          setCardTargets([piece.id]);
        } else {
          send({ type: "play_card", id: targetCard.id, target: piece.id });
        }
      }
      return;
    }
    if (
      state.turn === 0 &&
      selected !== null &&
      state.moves.some((m) => m.from === selected && m.to === cell)
    ) {
      const options = state.moves
        .filter((m) => m.from === selected && m.to === cell)
        .flatMap((m) => (m.promotion ? [m.promotion] : []));
      if (options.length) setPromotion({ from: selected, to: cell, options });
      else send({ type: "move", from: selected, to: cell });
      return;
    }
    if (piece?.owner === 1) {
      if (enemyInspection?.piece_id === piece.id) {
        setEnemyInspection(null);
      } else {
        try {
          const inspected: EnemyInspection = JSON.parse(engine.current!.inspect_enemy(cell));
          setEnemyInspection(inspected);
          setError("");
        } catch (e) {
          setEnemyInspection(null);
          setError(String(e));
        }
      }
      setSelected(null);
      return;
    }
    setEnemyInspection(null);
    setSelected(state.turn === 0 && piece?.owner === 0 && selected !== cell ? cell : null);
  }
  const turn = state?.turn === 0;
  const rewardRevisit = state?.phase === "reward" && state.map.nodes.some((node) => node.current && node.visited);
  const showMap = (state?.phase === "map" || rewardRevisit) && !rewardUpgradeOpen;
  const restPiece = state?.rest_options.find((p) => p.id === restChoice?.id);
  const restView = state?.phase === "rest" || rewardUpgradeOpen;
  const upgradeOpen = restView && !!restPiece && !restPiece.dead;
  const boardPieces = rewardUpgradeOpen ? state?.next_battle_pieces ?? [] : state?.pieces ?? [];
  function closeRest() {
    setRestChoice(null);
  }
  function closeRewardUpgrade() {
    setRewardUpgradeOpen(false);
    setRestChoice(null);
  }
  const restPath = state?.upgrade_paths.find(
    (p) => p.from === restPiece?.kind && p.to === restChoice?.target,
  );
  const previewKey = restPath?.to_key;
  const inspectedEnemy = state?.phase === "combat" ? state.pieces.find((p) => p.id === enemyInspection?.piece_id && p.cell === enemyInspection?.cell) : undefined;
  const selectedPiece = inspectedEnemy || state?.pieces.find((p) => p.cell === selected);
  const initiativeRemaining = selectedPiece?.owner === 1 ? state?.enemy_initiative_turns_left ?? 0 : state?.initiative_turns_left ?? 0;
  const enemyCaptures = new Set(enemyInspection?.moves.flatMap((m) => m.captures) ?? []);
  const enemyMoves = new Set(enemyInspection?.moves.map((m) => m.to) ?? []);
  const enemyRoutes = [...new Map((enemyInspection?.moves ?? []).filter((m) => m.route).map((m) => [m.route!.join(","), m.route!])).values()];
  const activeEffects = state?.pieces.filter((p) => p.limiter || p.temporary_upgrade || p.ward) ?? [];
  const focusedUpgrade = state?.pieces.find((p) => p.id === upgradeFocus);
  const stagedCount = targetCard?.definition.selection_mode === "upgrade" ? cardUpgrades.length : cardTargets.length;
  const stagedCost = targetCard?.definition.cost_per_target ? stagedCount : targetCard?.definition.cost ?? 0;
  function cancelCard() {
    setAbilityId(null);setPlacementPick(null);setPlacement([]);
    setTargetCard(null);
    setCardTargets([]);
    setCardUpgrades([]);
    setUpgradeFocus(null);
    setError("");
  }
  function confirmCard() {
    if(targetCard?.definition.selection_mode==="placement"){
      if(placement.length)send({type:"tactical_card",id:targetCard.id,targets:placement.map(p=>p.id),cells:placement.map(p=>p.cell)});
      return;
    }
    if (!targetCard || stagedCount < 1 || !state || stagedCost > state.mana) return;
    if (targetCard.definition.selection_mode === "multiple" || targetCard.definition.selection_mode === "upgrade") {
      send({ type: "play_card_selection", id: targetCard.id,
        targets: targetCard.definition.selection_mode === "multiple" ? cardTargets : [],
        upgrades: targetCard.definition.selection_mode === "upgrade" ? cardUpgrades : [] });
    } else {
      send({ type: "play_card", id: targetCard.id, target: cardTargets[0] });
    }
  }
  function returnToBoardOnNarrowScreen() {
    if (window.matchMedia("(max-width: 1000px)").matches)
      boardRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
  }
  function confirmCardUpgrade(target: number) {
    if (!focusedUpgrade) return;
    setCardUpgrades((old) => [...old.filter((choice) => choice.id !== focusedUpgrade.id), { id: focusedUpgrade.id, target }]);
    setUpgradeFocus(null);
    returnToBoardOnNarrowScreen();
  }
  function closeCardUpgrade() {
    setUpgradeFocus(null);
    returnToBoardOnNarrowScreen();
  }
  return (
    <>
      {floorTransition && <div className={`floor-transition stage-${floorTransition.stage} act-${floorTransition.next.floor}`} role="status" aria-label={`제${floorTransition.next.floor}층 진입`}>
        <div className="floor-transition-title"><small>CHESSLAYER · {floorTransition.next.floor === 1 ? "EXPEDITION BEGINS" : "NEXT ACT"}</small><strong>ACT {["I","II","III","IV"][floorTransition.next.floor-1]}</strong><span>{`제${floorTransition.next.floor}층 원정`}</span><small>{"새로운 원정 시작"}</small></div>
        <button onClick={skipFloorTransition} aria-label="층 전환 연출 건너뛰기">건너뛰기 ↗</button>
      </div>}
      <header className="topbar">
        <a
          className="brand"
          href="#"
          onClick={(e) => {
            e.preventDefault();
            if (screen !== "game") setScreen("home");
          }}
        >
          <PieceImage pieceKey="knight" owner={1} /> CHESSLAYER
        </a>
        <div className="top-right">
          <span className="edition">FIRST EXPEDITION / {`ACT ${["I","II","III","IV"][(state?.floor ?? 1)-1]}`}</span>
          <button
            className="text-button"
            onClick={() => setHelp(!help)}
            aria-expanded={help}
          >
            플레이 안내 {help ? "−" : "+"}
          </button>
        </div>
      </header>
      {screen === "game" && state && !floorTransition && createPortal(<button className="primary expedition-status-trigger" onClick={() => setStatusOpen(true)} aria-haspopup="dialog" aria-expanded={statusOpen}>원정 현황</button>, document.body)}
      {help && (
        <section className="guide">
          <strong>체크메이트 대신, 로얄을 포획하세요.</strong>
          <p>
            백 기물을 클릭하면 이동 가능한 칸이 표시됩니다. 카드 사용은 이동
            횟수를 쓰지 않습니다. 이동을 마치거나 턴 종료를 누르면 적이
            행동합니다. ★는 로얄, ◇는 가호입니다.
          </p>
          <p>
            맵에서 이어진 다음 노드를 선택하세요. 승리한 진형은 다음 전투로
            이어집니다. 적의 시작 칸과 겹친 아군은 역습으로 사라집니다. 킹을
            잃으면 원정이 끝납니다. 진행은 이 브라우저에 자동 저장됩니다.
          </p>
          <p>
            소멸: 사용 후 이번 전투에서 제외 · 보존: 턴 종료에도 손패 유지 ·
            선천성: 첫 손패 우선 · 휘발성: 쓰지 않고 턴 종료 시 소멸. 체크
            제한·캐슬링·앙파상은 없습니다. 폰 계열은 최종 랭크로 이동할 때
            승격할 기물을 선택합니다.
          </p>
        </section>
      )}
      {error && (
        <div className="error" role="alert">
          {error}
          <button className="text-button" onClick={() => setError("")}>
            닫기
          </button>
        </div>
      )}
      {saveNotice && (
        <p className="save-notice" role="status">
          {saveNotice}
        </p>
      )}
      {screen === "home" && (
        <main className="home">
          <section className="home-copy">
            <p className="eyebrow">A CHESS ROGUELIKE</p>
            <h1>Chesslayer</h1>
            <p className="home-version">v{version}</p>
            <p className="intro">
              길을 선택하고, 카드로 판을 바꾸세요.
              <br />
              당신의 기물로 완성하는 첫 번째 원정.
            </p>
            <button
              className="primary large"
              disabled={!ready}
              onClick={() => setScreen("choose")}
            >
              {ready ? "새 게임 시작" : "엔진 불러오는 중…"} <span>↗</span>
            </button>
            {ready && hasSave && (
              <button className="resume-button" onClick={resume}>
                저장된 원정 이어하기 →
              </button>
            )}
            <div className="home-facts">
              <span>갈라지는 경로</span>
              <span>선택하는 보상</span>
              <span>마지막 로얄</span>
            </div>
          </section>
          <div className="hero-art" aria-hidden="true">
            <div className="hero-grid">
              {Array.from({ length: 64 }, (_, i) => (
                <i
                  key={i}
                  className={(i + Math.floor(i / 8)) % 2 ? "dark" : ""}
                />
              ))}
            </div>
            <span className="hero-king">
              <PieceImage pieceKey="king" />
            </span>
            <span className="hero-knight">
              <PieceImage pieceKey="knight" owner={1} />
            </span>
            <span className="hero-caption">EVERY POSITION TELLS A STORY.</span>
          </div>
          <div className="home-foot">
            01 / FIRST EXPEDITION <span>RUST ENGINE · WASM · CHESSEMBLY</span>
          </div>
        </main>
      )}
      {screen === "choose" && (
        <main className="selection-page">
          <button className="text-button" onClick={() => setScreen("home")}>
            ← 처음으로
          </button>
          <p className="eyebrow">CHOOSE YOUR SQUAD</p>
          <h1>부대를 선택하세요</h1>
          <p className="muted">
            각 부대의 편성과 카드, 능력을 살펴보고 원정을 시작하세요.
          </p>
          <div className="developer-access">
            <button className="developer-unlock" onClick={unlockAll} disabled={!ready || cleared === 15}>
              개발자 모드 · 전체 해금
            </button>
            <span role="status">{cleared === 15 ? "모든 부대와 층이 개방되어 있습니다." : "플레이 확인용 · 모든 부대와 층을 바로 개방합니다."}</span>
          </div>
          <div className="squads">
            {choices.map((c) => (
              <button
                className={`squad ${squad === c.id ? "chosen" : ""}`}
                key={c.id}
                onClick={() => setSquad(c.id)}
                disabled={c.unlock_floor>0 && !(cleared & (1 << (c.unlock_floor-1)))}
                aria-pressed={squad === c.id}
              >
                <RelicIcon id={c.relic} name={starterRelicNames[c.relic]} className="squad-icon" />
                <span className="eyebrow">{c.title}</span>
                <h2>{c.name}</h2>
                <p>{c.description}</p>
                <span className="squad-radio">
                  {c.unlock_floor>0 && !(cleared & (1 << (c.unlock_floor-1))) ? `잠김 · ${c.unlock_floor}층 보스 격파` : squad === c.id ? "● 선택됨" : "○ 선택"}
                </span>
              </button>
            ))}
          </div>
          <div className="floor-select" role="group" aria-label="시작 층 선택">
            {[1,2,3,4].map(f=><button key={f} aria-pressed={floor===f} disabled={f>1 && !(cleared & (1<<(f-2)))} onClick={()=>setFloor(f)}>{f}층 {f>1 && !(cleared & (1<<(f-2))) ? `· ${f-1}층 격파 필요` : ""}</button>)}
          </div>
          <div className="launch">
            <span className="muted">
              선택한 부대의 시작 편성과 유물로 원정을 시작합니다.
            </span>
            <button className="primary large" onClick={start} disabled={!ready}>
              런 시작 <span>→</span>
            </button>
          </div>
        </main>
      )}
      {screen === "game" && state && showMap && (
        <ActMap state={state} send={send} coveredByReward={rewardRevisit} />
      )}
      {screen === "game" && state && !showMap && (
        <main className="game-layout" data-phase={state.phase} data-upgrade-open={upgradeOpen || undefined} data-card-upgrade-open={!!focusedUpgrade || undefined}>
          <nav className="node-navigation" aria-label="원정 위치">
            <span>
              {state.floor}층 / {state.map.nodes.find((n) => n.current)!.row + 1}번째 지점
              · {state.node_kind && nodeTypes[state.node_kind].label}
            </span>
          </nav>
          {state.phase !== "treasure" && state.phase !== "restaurant" && (
            <>
              <div className="battle-column">
              <section className="battle-area">
                <div className="battle-heading">
                  <div>
                    <p className="eyebrow">
                      FIRST EXPEDITION / {state.squad}
                    </p>
                    <h1>{state.encounter}</h1>
                  </div>
                  <div
                    className={`turn-pill ${turn ? "your-turn" : ""}`}
                    data-testid="turn"
                  >
                    {state.phase === "combat" ? (
                      <>
                        <i />
                        {turn ? "나의 턴" : "적의 턴"}
                        <small>{state.actions_left}회 행동</small>
                      </>
                    ) : restView ? (
                      rewardUpgradeOpen ? "전투 보상 · 기물 강화" : "쉼터 · 행동 1회"
                    ) : (
                      "진형 계승"
                    )}
                  </div>
                </div>
                <div className="board-frame" ref={boardRef}>
                  <div className="rank-labels">
                    {"87654321".split("").map((r) => (
                      <span key={r}>{r}</span>
                    ))}
                  </div>
                  <div className="board" role="group" aria-label="체스 보드">
                    {Array.from({ length: 64 }, (_, i) => {
                      const piece = boardPieces.find((p) => p.cell === i);
                      const legal =
                        selected !== null &&
                        state.moves.some(
                          (m) => m.from === selected && m.to === i,
                        );
                      const enemyCapture = !!inspectedEnemy && enemyCaptures.has(i);
                      const enemyMove = !!inspectedEnemy && !piece && enemyMoves.has(i);
                      const activeAbility=state.abilities.find(a=>a.id===abilityId);
                      const target = (piece && (targetCard?.targets.includes(piece.id) || activeAbility?.targets.includes(piece.id) || state.effect_choice?.targets.includes(piece.id))) || state.effect_choice?.cells.includes(i) || (targetCard?.definition.selection_mode==="square" && !piece && i>=48);

                      const staged = piece && (cardTargets.includes(piece.id) || cardUpgrades.some((u) => u.id === piece.id));
                      const square = `${"abcdefgh"[i % 8]}${8 - Math.floor(i / 8)}`;
                      return (
                        <button
                          key={i}
                          data-square={square}
                          data-legal={legal || undefined}
                          data-enemy-move={enemyMove || undefined}
                          data-enemy-capture={enemyCapture || undefined}
                          className={`square ${(i + Math.floor(i / 8)) % 2 ? "dark" : "light"} ${(restView ? restChoice?.cell : selected) === i ? "selected" : ""} ${inspectedEnemy?.cell === i ? "enemy-selected" : ""} ${restView && restChoice?.cell === i ? "rest-selected" : ""} ${legal ? "legal" : ""} ${target ? "targetable" : ""} ${staged ? "card-staged" : ""}`}
                          title={piece ? `${piece.name} · ${statusText(piece)}${piece.self_defense ? " · 호신용품: 직접 포획되면 포획한 적도 사망" : ""}` : undefined}
                          onClick={() => clickSquare(i)}
                          disabled={
                            !restView &&
                            (state.phase !== "combat" || !!promotion || (!turn && piece?.owner !== 1 && !(!piece && inspectedEnemy)))
                          }
                          aria-label={`${square}${piece ? ` ${piece.owner === 0 ? "백" : "흑"} ${piece.name}${statusText(piece) ? ` ${statusText(piece)}` : ""}${piece.royal ? " 로얄" : ""}${piece.ward ? " 가호" : ""}${piece.self_defense ? " 호신용품 보호 대상" : ""}${piece.limiter ? ` 리미터 해제 남은 이동 ${piece.limiter.moves_left}회 효과 종료 후 사망` : ""}` : " 빈 칸"}${legal ? " 이동 가능" : ""}${enemyMove ? " 적 이동 가능" : ""}${enemyCapture ? " 적 포획 가능" : ""}${staged ? " 카드 선택됨" : ""}`}
                          aria-pressed={
                            (restView
                              ? restChoice?.cell
                              : inspectedEnemy?.cell ?? selected) === i
                          }
                        >
                          {legal && (
                            <span
                              className={piece ? "capture-dot" : "move-dot"}
                            />
                          )}
                          {enemyMove && <span className="enemy-move-dot" />}
                          {enemyCapture && <span className="enemy-capture-ring" />}
                        </button>
                      );
                    })}
                    {inspectedEnemy && enemyRoutes.length > 0 && <svg className="enemy-routes" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
                      {enemyRoutes.map((route) => <polyline key={route.join(",")} points={route.map((cell) => `${cell % 8 + 0.5},${Math.floor(cell / 8) + 0.5}`).join(" ")} />)}
                    </svg>}
                    <div className="piece-layer">
                      {boardPieces.map((p) => (
                        <BoardPiece
                          key={p.id}
                          piece={p}
                          focused={
                            restView && restChoice?.id === p.id
                          }
                          previewKey={
                            restChoice?.id === p.id ? previewKey : undefined
                          }
                        />
                      ))}
                      {state.phase === "rest" &&
                        restPiece?.dead &&
                        restChoice && (
                          <span
                            className="piece rest-ghost rest-focused"
                            style={{
                              left: `${(restChoice.cell % 8) * 12.5}%`,
                              top: `${Math.floor(restChoice.cell / 8) * 12.5}%`,
                            }}
                          >
                            <PieceImage pieceKey={restPiece.key} />
                            <small>미리보기</small>
                          </span>
                        )}
                    </div>
                  </div>
                  {promotion && (
                    <PromotionPicker
                      options={promotion.options}
                      square={`${"abcdefgh"[promotion.to % 8]}${8 - Math.floor(promotion.to / 8)}`}
                      onChoose={(kind) =>
                        send({
                          type: "move",
                          from: promotion.from,
                          to: promotion.to,
                          promotion: kind,
                        })
                      }
                      onCancel={() => setPromotion(null)}
                    />
                  )}
                  <div className="file-labels">
                    {"abcdefgh".split("").map((f) => (
                      <span key={f}>{f}</span>
                    ))}
                  </div>
                </div>
                {state.phase === "combat" && <div className="board-caption">
                  <span>★ 적 로얄: {state.royal_name}{state.enemy_initiative_turns_left > 0 && ` · 선공권 ${state.enemy_initiative_turns_left}턴`}</span>
                </div>}
                {state.phase==="combat" && state.effect_choice && <div className="action-hint target-hint" role="status"><strong>{state.effect_choice.prompt}</strong><span>강조한 기물 또는 칸을 선택하세요.</span></div>}
                {state.phase==="combat" && abilityId!==null && <div className="action-hint target-hint"><strong>{state.abilities.find(a=>a.id===abilityId)?.name} 대상 선택</strong><span>강조된 기물을 선택하세요.</span><button onClick={()=>setAbilityId(null)}>취소</button></div>}
                {state.phase === "combat" && (targetCard || inspectedEnemy || !turn) && (
                  <div
                    className={`action-hint ${targetCard ? "target-hint" : ""}`}
                    aria-live="polite"
                  >
                    {targetCard ? (
                      <>
                        <strong>{targetCard.definition.name}</strong>
                        {targetCard.definition.selection_mode==="square" && <span>1~2랭크의 빈 칸을 선택하세요.</span>}
                        {targetCard.definition.selection_mode==="placement" && <><span>{placementPick!==null ? "새 배치 칸 선택" : "재배치할 아군 선택"} · {placement.length}기 지정</span><button className="primary" disabled={!placement.length} onClick={confirmCard}>배치 완료</button></>}

                        {targetCard.definition.kind === "limiter_release" && <span>효과 종료 후 대상 사망</span>}
                        <span>선택 {stagedCount}기 · 현재 코스트 {stagedCost} / 마나 {state.mana}</span>
                        {targetCard.definition.selection_mode === "upgrade" && cardUpgrades.length > 0 && <span className="staged-upgrades">{cardUpgrades.map((choice) => <button key={choice.id} onClick={() => setCardUpgrades((old) => old.filter((u) => u.id !== choice.id))}>{state.pieces.find((p) => p.id === choice.id)?.name} ×</button>)}</span>}
                        {stagedCount > 0 && (["multiple", "upgrade"].includes(targetCard.definition.selection_mode) || ["limiter_release", "sacred_domain"].includes(targetCard.definition.kind)) ? <button className="primary" onClick={confirmCard} disabled={stagedCost > state.mana}>확인</button> : null}
                        <button
                          className="text-button"
                          onClick={cancelCard}
                        >
                          취소 Esc
                        </button>
                      </>
                    ) : inspectedEnemy ? (
                      <span>적 행마 조회</span>
                    ) : (
                      "상대가 다음 수를 고르고 있습니다…"
                    )}
                  </div>
                )}
              </section>
          {state.phase === "combat" && (
            <section className="hand-area">
              <div className="hand-heading">
                <h2>손패 <span>{state.hand.length}/10</span></h2>
                <div className="piles">
                  <span>
                    드로우 <b data-testid="draw-count">{state.draw_count}</b>
                  </span>
                  <span>
                    버림 <b>{state.discard_count}</b>
                  </span>
                  <span>
                    소멸 <b>{state.exhausted_count}</b>
                  </span>
                  <span>
                    전체 덱 <b data-testid="deck-count">{state.deck_count}</b>
                  </span>
                </div>
              </div>
              <div className="hand">
                {state.hand.length === 0 && <p className="empty-hand">손패 없음</p>}
                {state.hand.map((card) => (
                  <button
                    data-card-id={card.id}
                    className={`playing-card ${targetCard?.id === card.id ? "active-card" : ""}`}
                    key={card.id}
                    onClick={() => chooseCard(card)}
                    disabled={!!card.disabled_reason || !!promotion || !!state.effect_choice}
                    title={card.disabled_reason || card.definition.description}
                  >
                    <CardFace card={card.definition} selectedCount={targetCard?.id === card.id ? stagedCount : 0} />
                    {card.disabled_reason && (
                      <span className="card-unavailable">
                        {card.disabled_reason}
                      </span>
                    )}
                  </button>
                ))}
              </div>
            </section>
          )}
              </div>
              <aside className="sidebar">
                {restView ? (
                  <RestPanel
                    state={state}
                    choice={restChoice}
                    onChoose={setRestChoice}
                    onCancel={rewardUpgradeOpen ? closeRewardUpgrade : closeRest}
                    send={send}
                    history={rewardUpgradeOpen ? rewardUpgradeHistory : upgradeHistory}
                    onContinue={rewardUpgradeOpen ? closeRewardUpgrade : closeRest}
                    reward={rewardUpgradeOpen}
                  />
                ) : (
                  <>
                    {targetCard?.definition.selection_mode === "upgrade" && focusedUpgrade && <CardUpgradePanel
                      key={focusedUpgrade.id}
                      card={targetCard}
                      piece={focusedUpgrade}
                      paths={state.upgrade_paths}
                      onConfirm={confirmCardUpgrade}
                      onCancel={closeCardUpgrade}
                    />}
                    {selectedPiece && (
                      <section className={`selected-piece-info ${inspectedEnemy ? "enemy-piece-info" : ""}`} aria-label={`${inspectedEnemy ? "적" : "아군"} 기물 정보`}>
                        <PieceImage
                          pieceKey={selectedPiece.key}
                          owner={selectedPiece.owner}
                        />
                        <div>
                          <small className="piece-side-label">{inspectedEnemy ? "적 기물 · 행마 조회" : "아군 기물"}</small>
                          <strong>
                            {selectedPiece.name} · {selectedPiece.square}
                          </strong>
                          {selectedPiece.royal && <span className="royal-info">★ {selectedPiece.key==="warehouse" ? "창고 · 모든 창고가 파괴되면 패배합니다." : <>로얄 · {inspectedEnemy ? "이 기물은 승리 목표입니다." : "이 기물을 잃으면 원정이 끝납니다."}</>}</span>}
                          <small className="piece-info-heading">행마법 · 특수 능력</small>
                          <p>{selectedPiece.description}</p>
                          {statusText(selectedPiece) && <p className="status-list">{statusText(selectedPiece)}</p>}
                          {state.abilities.filter(a=>a.id===selectedPiece.id && a.targets.length>0).map(a=><div className="ability-control" key={a.name}><button className="primary" disabled={!!state.effect_choice} onClick={()=>{setTargetCard(null);setAbilityId(a.id);}}>{a.name} · 행동 1회{state.ammo!==null && a.name==="사격"?" · 탄약 1":""}</button><small>{a.targets.map((id,i)=>`${state.pieces.find(p=>p.id===id)?.square} ${a.chances[i]}%`).join(" / ")}</small></div>)}
                          <small className="piece-info-heading">현재 적용 효과</small>
                          {!selectedPiece.temporary_upgrade && !selectedPiece.ward && !selectedPiece.limiter && !selectedPiece.self_defense && selectedPiece.absorbed_moves.length === 0 && initiativeRemaining === 0 && <small className="effect-note">없음</small>}
                          {initiativeRemaining > 0 && <small className="effect-note">선공권 보호 · 상대의 포획 제한 · 남은 상대 턴 {initiativeRemaining}회</small>}
                          {selectedPiece.temporary_upgrade && <small className="effect-note">일시 강화 · 현재 전투에서만 유지</small>}
                          {selectedPiece.ward && <small className="effect-note">가호 · 다음 포획 시 공격 무효</small>}
                          {selectedPiece.self_defense && <small className="effect-note">호신용품 · 직접 포획되면 포획한 적도 사망</small>}
                          {selectedPiece.absorbed_moves.length > 0 && <small className="effect-note">흡수한 행마 · {selectedPiece.absorbed_moves.join(", ")}</small>}
                          {selectedPiece.limiter && <small className="effect-note danger">리미터 해제 · {selectedPiece.limiter.active ? `남은 이동 ${selectedPiece.limiter.moves_left}회` : "다음 소속 턴에 최대 3회 이동"} · 턴 종료 후 사망</small>}
                        </div>
                      </section>
                    )}
                    <section className="turn-panel">
                      <div className="section-label">
                        TURN {state.round.toString().padStart(2, "0")}
                        <span>{state.ammo!==null ? `한계 ${state.ammo_capacity}` : "기본 마나 3"}</span>
                      </div>
                      <div className="mana">
                        <span className="mana-orb">{state.mana}</span>
                        <div>
                          <strong>{state.ammo!==null ? `탄약 / ${state.ammo_capacity}` : "사용 가능한 마나"}</strong>
                          <small>{state.mana_rule}</small>
                        </div>
                      </div>
                      <button
                        className="primary end-turn"
                        onClick={() => send({ type: "end_turn" })}
                        disabled={!turn || state.phase !== "combat" || !!promotion || !!state.effect_choice}
                      >
                        턴 종료 <span>→</span>
                      </button>
                    </section>
                    {state.phase === "combat" && activeEffects.length > 0 && <section className="effect-panel" aria-label="전투 효과">
                      <div className="section-label">전투 효과 <span>{activeEffects.length}기</span></div>
                      {activeEffects.map((p) => <div className="effect-row" key={p.id}>
                        <PieceImage pieceKey={p.key} owner={p.owner} name={p.name} />
                        <div><strong>{p.owner === 0 ? "아군" : "적"} {p.name} · {p.square}</strong>
                          {p.limiter && <small>리미터 · {p.limiter.active ? `남은 이동 ${p.limiter.moves_left}회` : "다음 소속 턴 최대 3회"} · 종료 후 사망</small>}
                          {p.temporary_upgrade && <small>일시 강화 · 전투 한정</small>}
                          {p.ward && <small>가호 · 다음 포획 무효</small>}
                        </div>
                      </div>)}
                    </section>}
                    <section className="relic-panel">
                      <div className="section-label">
                        보유 유물 <span>{state.relics.length}</span>
                      </div>
                      {state.relics.map((r) => (
                        <div className="relic" key={r.id}>
                          <RelicIcon id={r.id} name={r.name} />
                          <div>
                            <strong>{r.name}</strong>
                            <p>{r.description}</p>
                            {r.status && <small className="relic-status" role="status">{r.status}</small>}
                          </div>
                        </div>
                      ))}
                    </section>
                    <section className="log-panel">
                      <div className="section-label">
                        전투 기록<span>최근 기록</span>
                      </div>
                      <div
                        className="battle-log"
                        ref={logRef}
                        role="log"
                        aria-label="전투 기록"
                      >
                        {state.log.map((line, i) => (
                          <p
                            key={`${i}-${line}`}
                            className={
                              line.includes("역습") || line.includes("승리")
                                ? "important"
                                : ""
                            }
                          >
                            <span>·</span>
                            {line}
                          </p>
                        ))}
                      </div>
                    </section>
                  </>
                )}
              </aside>
            </>
          )}
          {state.phase !== "combat" && state.phase !== "rest" && state.phase !== "reward" && (
            <section className="outcome" ref={outcomeRef} aria-live="polite">
              {state.phase === "restaurant" && <><h2>식당</h2><p>주인은 말없이 따뜻한 음식을 내놓았습니다.</p><button className="primary" onClick={()=>send({type:"dine",eat:true})}>식사하기 · 전체 허기 해소</button><button onClick={()=>send({type:"dine",eat:false})}>떠나기</button></>}
              {state.phase === "treasure" && (
                <>
                  <p className="eyebrow">A GIFT ALONG THE WAY</p>
                  <h2>남겨진 보물</h2>
                  {state.node_reward && <div className="treasure-relic"><RelicIcon id={state.node_reward.id} name={state.node_reward.name} /></div>}
                  <h3>{state.node_reward?.name || "부대 카드 1장"}</h3>
                  <p>{state.node_reward?.description}</p>
                  <button
                    className="primary large"
                    onClick={() => send({ type: "claim_treasure" })}
                  >
                    보물 받고 맵으로 →
                  </button>
                </>
              )}
              {(state.phase === "victory" || state.phase === "defeat") && (
                <>
                  <span className="end-symbol">
                    <PieceImage
                      pieceKey="king"
                      owner={state.phase === "victory" ? 0 : 1}
                    />
                  </span>
                  <p className="eyebrow">
                    EXPEDITION{" "}
                    {state.phase === "victory" ? "COMPLETE" : "ENDED"}
                  </p>
                  <h2>
                    {state.phase === "victory"
                      ? "마지막 왕관까지."
                      : "왕이 쓰러졌습니다."}
                  </h2>
                  <p>
                    {state.phase === "victory"
                      ? `${state.encounter} 보스를 쓰러뜨렸습니다. 2층 원정을 완수했습니다.`
                      : "진형은 여기에서 멈추지만, 다음 원정의 한 수는 달라질 수 있습니다."}
                  </p>
                  {state.phase==="victory" && <p className="unlock-notice">{state.unlocked_squad} 해금{state.floor<4 ? ` · ${state.floor+1}층 개방` : " · 네 부대의 영토를 수복했습니다"}</p>}
                  <button
                    className="primary large"
                    onClick={() => {
                      setScreen("choose");
                      setError("");
                    }}
                  >
                    새 원정 시작 →
                  </button>
                </>
              )}
            </section>
          )}
        </main>
      )}
      {screen === "game" && state && statusOpen && <ExpeditionStatus state={state} onClose={closeStatus} />}
      {screen==="game" && state?.aftermath && createPortal(<div className="reward-overlay"><section className="battle-rewards" role="dialog" aria-modal="true" aria-label={state.aftermath.prompt}><h2>{state.aftermath.prompt}</h2><div className="aftermath-options">{state.aftermath.targets.map(id=>{const p=state.rest_options.find(p=>p.id===id);return <button key={id} onClick={()=>send({type:"aftermath",target:id})}>{p?.name} · {p?.square}</button>;})}</div></section></div>,document.body)}
      {screen === "game" && state?.phase === "reward" && !state.aftermath && !floorTransition && !rewardUpgradeOpen && (
        <BattleRewards state={state} send={send} revisit={!!rewardRevisit} onUpgrade={() => { setRestChoice(null); setRewardUpgradeOpen(true); }} />
      )}
      <footer>
        CHESSLAYER <span>첫 번째 원정 · 진행은 이 브라우저에 저장됩니다.</span>
      </footer>
    </>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

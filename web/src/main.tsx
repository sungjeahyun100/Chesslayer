import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { createPortal } from "react-dom";
import init, { WebGame, catalog, compendium } from "./wasm/chesslayer_web";
import type {
  Card,
  Ability,
  Character,
  Command,
  EnemyInspection,
  Piece,
  Promotion,
  Snapshot,
  Progress,
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
import { SquadSelection } from "./SquadSelection";
import { PieceStatuses } from "./PieceStatuses";
import { LocationPanel } from "./LocationPanel";
import { Compendium, type CompendiumSquad } from "./Compendium";
import { PieceSandbox } from "./PieceSandbox";
import { PROGRESS_KEY, DEVELOPER_MODE_KEY, DEVELOPER_RUN_KEY, initialProgress, readProgress, mergeProgress, developerProgress, floorLabel, floorNames } from "./progress";

// Keep the original storage slot so legacy saves receive the WASM version notice.
const SAVE_KEY = "chesslayer.run.v1";
type BoardAnnotation = { from: number; to: number };

function boardCellAt(board: HTMLDivElement, clientX: number, clientY: number): number | null {
  const bounds = board.getBoundingClientRect();
  const x = (clientX - bounds.left - board.clientLeft) / board.clientWidth;
  const y = (clientY - bounds.top - board.clientTop) / board.clientHeight;
  if (x < 0 || x >= 1 || y < 0 || y >= 1) return null;
  return Math.floor(y * 8) * 8 + Math.floor(x * 8);
}

const simpleMovement: Record<string, string> = {
  amazon: "퀸 + 나이트", centaur: "킹 + 나이트", mounted_officer: "킹 + 나이트",
  knight: "나이트", scout: "나이트", bishop: "비숍", rook: "룩", queen: "퀸",
  king: "킹", tempest_king: "킹", smiling_king: "킹", toast_lord: "킹", ecstatic_lord: "킹",
  pawn: "폰", bouncing_pawn: "폰", hungry_follower: "폰", procession_follower: "폰",
  unicorn: "나이트 + 빈칸 순간이동", emperor: "킹", hook: "룩 + 직각 꺾기",
  movement_god: "직교·대각 꺾기", tempest_pawn: "폰 + 좌우 이동·2칸 포획",
  drone: "나이트 + 직교 1칸", c4_drone: "나이트 + 직교 1칸",
  machine_gun_drone: "나이트 + 직교 1칸", grenade_launcher_drone: "나이트 + 직교 1칸",
  tactical_nuke_drone: "나이트 + 직교 1칸", tank: "킹 + 직선 2칸",
  fragment_tank: "킹 + 직선 2칸", ap_tank: "킹 + 직선 2칸",
  he_tank: "킹 + 직선 2칸", rifleman: "직교 1칸", mortar: "직교 1칸",
  sniper: "대각 1칸", marksman: "킹", special_commander: "킹", shadow_commander: "킹 + 나이트",
  tempest_rook: "대각 1칸 → 직교 직선", tempest_bishop: "직교 1칸 → 전방 대각",
  tempest_queen: "템페스트 룩 + 비숍", tempest_knight: "대각 경유 도약 + 직교 3칸",
  typhoon_rook: "템페스트 룩 + 룩", typhoon_bishop: "템페스트 비숍 + 비숍",
  typhoon_queen: "템페스트 퀸 + 퀸", typhoon_knight: "템페스트 나이트 + 나이트",
  bouncing_rook: "반사 룩", bouncing_bishop: "반사 비숍", bouncing_queen: "반사 룩 + 비숍",
  bouncing_tempest_rook: "반사 템페스트 룩", bouncing_tempest_bishop: "반사 템페스트 비숍",
  bouncing_tempest_queen: "반사 템페스트 퀸", bouncing_knightrider: "반사 나이트라이더",
  knightrider: "나이트 연속 도약", cardinal: "연속 반사 비숍", tengu: "대각 꺾기",
  frenzied_dancer: "나이트", masked_dancer: "나이트", tempting_dancer: "나이트",
  vanguard_officer: "킹", vanguard_commander: "킹 + 연속 1칸", mounted_commander: "2칸 이내 도약",
  phantom_scout: "나이트 + 대각 2칸 도약", butcher_cart: "룩", slaughter_cart: "룩",
  funeral_cart: "룩", feast_cart: "룩", warehouse: "이동 불가", corpse: "이동 불가",
  tribal_dictator: "킹", lobster: "직교 1칸", sword_infantry: "장창 + 전방 대각 1칸",
  spear_infantry: "직교 1칸 + 전방 나이트", axe_infantry: "직교 1칸 + 전방 대각 2칸",
  halberd_infantry: "장창 + 전방 대각 2칸", pistol_infantry: "검 보병",
  elite_commando: "킹 + 전방 2칸 도약", commando: "킹 + 전방 2칸 도약",
  masked_lady: "퀸", red_lady: "퀸", banquet_lady: "퀸", inviting_lady: "퀸",
  intoxicated_priest: "비숍", feast_priest: "비숍", hunger_priest: "비숍", locust_high_priest: "비숍",
  ecstatic_dancer: "나이트", glutton_follower: "폰", inviting_follower: "폰",
  eternal_guest: "킹",
};
const specialMovement: Record<string, string> = {
  unicorn: "빈칸 순간이동은 포획 불가", emperor: "직접 포획한 기물의 행마 흡수",
  drone: "십자포화: 직교 인접 칸과 자신 제거", c4_drone: "광역 자폭: 주변 8칸과 자신 제거",
  machine_gun_drone: "기관총 사격 또는 십자포화", grenade_launcher_drone: "유탄 사격 또는 십자포화",
  tactical_nuke_drone: "광역 자폭 후 방사성 낙진", tank: "직선 사격 · 탄약 2",
  fragment_tank: "명중 시 주변 직교 칸 폭발", ap_tank: "관통 사격",
  he_tank: "명중 시 주변 직교 칸 폭발", rifleman: "전방 3방향 사격 · 탄약 1",
  mortar: "장애물 너머 2~5칸 포격 · 탄약 2", marksman: "8방향 7칸 사격",
  sniper: "8방향 7칸 사격",
  special_commander: "8방향 5칸 사격 · 전투 시작 시 가호", shadow_commander: "8방향 5칸 사격 · 전투 시작 시 가호·민첩 1",
  frenzied_dancer: "인접 시체 포획", masked_dancer: "주변 8칸 시체 포획",
  tempting_dancer: "시체 포획 시 인접 적 허기 악화", scout: "전투 시작 시 민첩 1",
  phantom_scout: "전투 시작 시 민첩 2", butcher_cart: "첫 시체 포획 시 카드 1장",
  slaughter_cart: "첫 시체 포획 시 민첩 1", funeral_cart: "직선 경로의 시체 통과",
  feast_cart: "시체 포획 후 다른 시체 1칸 이동", warehouse: "탄약 한계 +200 · 모두 파괴되면 패배",
  corpse: "양 진영 모두 포획 가능", tribal_dictator: "주변 사망 시 HP·행마 흡수 · 갈고리 반격 · 샷건",
  smiling_king: "근처 아군의 첫 시체 포획 시 카드 1장", toast_lord: "근처 아군의 첫 시체 포획 시 카드 1장·민첩 1",
  ecstatic_lord: "아군의 첫 시체 포획 시 카드 1장·민첩 1·마나 1",
  hungry_follower: "전방 시체 포획", procession_follower: "승격 불가",
  bouncing_pawn: "아군 바운싱 기물 반사", pistol_infantry: "전방 사격",
  elite_commando: "돌격 사격 · 기본 명중률 100%",
  commando: "전방 3방향 4칸 사격",
  masked_lady: "인접 아군 폰을 시체로 전환 · 행동 1회",
  red_lady: "인접 아군 비로얄을 시체로 전환 · 행동 1회",
  banquet_lady: "제물 능력 사용 시 카드 1장",
  inviting_lady: "제물 능력 사용 시 근처 적 허기 악화",
  intoxicated_priest: "첫 시체 포획 시 인접 아군 민첩 1",
  feast_priest: "첫 시체 포획 시 2칸 이내 아군 민첩 1",
  hunger_priest: "시체 포획 시 2칸 이내 적 허기 악화",
  locust_high_priest: "시체 포획 시 아군 민첩 2",
  ecstatic_dancer: "주변 8칸 시체 포획 · 첫 포획 시 민첩 1",
  glutton_follower: "전방 시체 포획 · 첫 포획 시 민첩 1",
  inviting_follower: "전방 시체 포획 · 포획 시 근처 적 허기 악화",
  eternal_guest: "양 진영에 식재료 · 아군 시체 포획 강화 · 3·6수에 허기 악화",
};
function movementInfo(piece: Piece): { movement: string; special?: string } {
  const movement = simpleMovement[piece.key] ?? (piece.description.split(".")[0] || piece.name);
  const special = specialMovement[piece.key] ?? (piece.description.split(/(?<=\.)\s+/).slice(1)
    .filter((sentence) => /전투당|사격|자폭|탄약|허기|민첩|카드|가호|낙진|포격|폭발|흡수|축복|제물|폭사|패배/.test(sentence))
    .join(" ") || undefined);
  return { movement, special };
}

function cardSelectionHint(kind: string): string {
  const hints: Record<string, string> = {
    body_slam: "아군과 적을 같은 수만큼 선택하세요. 로얄은 대상이 아닙니다.",
    masquerade: "위치를 교환할 아군 비로얄 2기를 선택하세요.",
    intermission_waltz: "서로 인접한 아군 비로얄 2기를 선택하세요.",
    clear_tablecloth: "시체를 먼저, 이동할 인접한 빈 칸을 다음으로 선택하세요.",
    delivered_supper: "시체를 먹을 아군과 2칸 이내의 시체를 선택하세요.",
    last_supper: "소비할 시체 최대 3개와 민첩을 받을 아군을 선택하세요.",
    frontline: "재배치할 아군과 목적지 빈 칸을 차례로 선택하세요.",
    infiltration: "이동할 아군과 목적지 빈 칸을 선택하세요.",
    extraction: "구출할 아군과 목적지 빈 칸을 선택하세요.",
    emergency_conscription: "사망한 폰을 아래에서 고르고 아군 진영의 빈칸을 선택하세요.",
    assault_order: "전방 빈칸으로 이동할 아군 1~3기를 순서대로 선택하세요.",
    rapid_reload: "사수와 사격할 적을 선택하세요.",
    focus_fire: "사수 두 기와 같은 적 하나를 선택하세요.",
  };
  return hints[kind] ?? "대상을 선택한 뒤 확인을 누르세요. 다시 누르면 선택이 해제됩니다.";
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
      {piece.royal && <span className="royal-mark">★</span>}
      {piece.ward && <span className="ward-mark" title="가호">✦</span>}
      {piece.temporary_upgrade && <span className="temporary-mark" title="일시 강화 · 이번 전투 한정">↑</span>}
      {piece.self_defense && <span className="self-defense-mark">◆</span>}
      {piece.limiter && <span className="limiter-mark" title={`리미터 해제: 남은 이동 ${piece.limiter.moves_left}회 · 0회가 되면 사망하고 턴 종료`}>{piece.limiter.moves_left}</span>}
      {piece.statuses?.some((status) => /전투불능/.test(status)) && <span className="status-mark disabled-mark" title="영구 전투불능 · 휴식에서 고해성사">×</span>}
      {piece.statuses?.some((status) => /종소리/.test(status)) && <span className="status-mark bell-mark" title="만찬의 종소리 대상">!</span>}
      {piece.statuses?.some((status) => /기아|굶주림|공복/.test(status)) && <span className="hunger-mark" title={piece.statuses.filter((status) => /기아|굶주림|공복/.test(status)).join(" ")}>◌</span>}
    </span>
  );
}
function App() {
  const engine = useRef<WebGame | null>(null);
  const enemyWorker = useRef<Worker | null>(null);
  const [ready, setReady] = useState(false);
  const [choices, setChoices] = useState<Character[]>([]);
  const [screen, setScreen] = useState<"home" | "choose" | "game" | "compendium" | "sandbox">("home");
  const [compendiumEntries, setCompendiumEntries] = useState<CompendiumSquad[]>([]);
  const [character, setCharacter] = useState("guardian");
  const [startFloor, setStartFloor] = useState(1);
  const [progress, setProgress] = useState<Progress>(initialProgress);
  const [developerMode, setDeveloperMode] = useState(false);
  const developerRun = useRef(false);
  const [state, setState] = useState<Snapshot | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [enemyInspection, setEnemyInspection] = useState<EnemyInspection | null>(null);
  const [activeAbility, setActiveAbility] = useState<Ability | null>(null);
  const [hoveredAbilityTarget, setHoveredAbilityTarget] = useState<number | null>(null);
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
  const [lastEnemyMove, setLastEnemyMove] = useState<{ from: number; to: number } | null>(null);
  const [boardAnnotations, setBoardAnnotations] = useState<BoardAnnotation[]>([]);
  const [annotationPreview, setAnnotationPreview] = useState<BoardAnnotation | null>(null);
  const annotationDrag = useRef<{ pointerId: number; from: number } | null>(null);
  const boardRef = useRef<HTMLDivElement>(null);
  const outcomeRef = useRef<HTMLElement>(null);
  const previousPhase = useRef<string | undefined>(undefined);
  useEffect(() => {
    let active = true;
    init()
      .then(() => {
        if (active) {
          const availableChoices: Character[] = JSON.parse(catalog());
          setChoices(availableChoices);
          setCompendiumEntries(JSON.parse(compendium()));
          setReady(true);
          try {
            setHasSave(!!localStorage.getItem(SAVE_KEY));
            setProgress(readProgress());
            setDeveloperMode(localStorage.getItem(DEVELOPER_MODE_KEY) === "true");
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
      if (state.phase !== "combat") {
        setBoardAnnotations([]);
        setAnnotationPreview(null);
        annotationDrag.current = null;
      }
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
  function startBoardAnnotation(event: React.PointerEvent<HTMLDivElement>) {
    if (state?.phase !== "combat" || event.button !== 2) return;
    const from = boardCellAt(event.currentTarget, event.clientX, event.clientY);
    if (from === null) return;
    event.preventDefault();
    annotationDrag.current = { pointerId: event.pointerId, from };
    setAnnotationPreview({ from, to: from });
    event.currentTarget.setPointerCapture(event.pointerId);
  }
  function moveBoardAnnotation(event: React.PointerEvent<HTMLDivElement>) {
    const drag = annotationDrag.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const to = boardCellAt(event.currentTarget, event.clientX, event.clientY);
    setAnnotationPreview(to === null ? null : { from: drag.from, to });
  }
  function endBoardAnnotation(event: React.PointerEvent<HTMLDivElement>) {
    const drag = annotationDrag.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    const to = boardCellAt(event.currentTarget, event.clientX, event.clientY);
    if (to !== null) {
      setBoardAnnotations((current) => current.some((mark) => mark.from === drag.from && mark.to === to)
        ? current.filter((mark) => mark.from !== drag.from || mark.to !== to)
        : [...current, { from: drag.from, to }]);
    }
    annotationDrag.current = null;
    setAnnotationPreview(null);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) event.currentTarget.releasePointerCapture(event.pointerId);
  }
  function cancelBoardAnnotation(event: React.PointerEvent<HTMLDivElement>) {
    if (annotationDrag.current?.pointerId !== event.pointerId) return;
    annotationDrag.current = null;
    setAnnotationPreview(null);
  }
  useEffect(() => {
    if (targetCard)
      boardRef.current?.scrollIntoView({ behavior: "smooth", block: "center" });
  }, [targetCard]);
  const send = useCallback((command: Command) => {
    if (!engine.current) return;
    try {
      const before: Snapshot = JSON.parse(engine.current.snapshot());
      const next: Snapshot = JSON.parse(engine.current.dispatch(JSON.stringify(command)));
      if (before.phase !== "combat" && next.phase === "combat") {
        setLastEnemyMove(null);
        setBoardAnnotations([]);
      } else if (command.type === "enemy_step_selected" && next.phase === "combat") {
        const moved = before.pieces.find((piece) => piece.owner === 1 &&
          next.pieces.some((after) => after.id === piece.id && after.cell !== piece.cell));
        const destination = next.pieces.find((piece) => piece.id === moved?.id)?.cell;
        if (moved && destination !== undefined) setLastEnemyMove({ from: moved.cell, to: destination });
      }
      if ((command.type === "choose_reward" || command.type === "continue_reward" || command.type === "advance_floor") &&
        (before.floor !== next.floor || before.final_battle !== next.final_battle) && next.phase === "map" && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
        setFloorTransition({ next, stage: "out" });
      } else {
        setState(next);
      }
      persist(engine.current);
      setRestChoice(null);
      if (command.type === "claim_upgrade") setRewardUpgradeOpen(false);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setActiveAbility(null);
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
    if (screen !== "game" || state?.phase !== "combat" || state.turn !== 1 || state.effect_choice)
      return;
    const timer = setTimeout(() => {
      const saved = engine.current?.save();
      if (!saved) return;
      const worker = new Worker(new URL("./enemyWorker.ts", import.meta.url), { type: "module" });
      enemyWorker.current = worker;
      worker.onmessage = (event: MessageEvent<{ choice?: [number, number] | null; error?: string }>) => {
        if (enemyWorker.current !== worker || engine.current?.save() !== saved) return;
        enemyWorker.current = null;
        worker.terminate();
        if (event.data.error) { setError(`적 AI 오류: ${event.data.error}`); return; }
        const [from, index] = event.data.choice ?? [null, null];
        send({ type: "enemy_step_selected", from, index });
      };
      worker.onerror = (event) => {
        if (enemyWorker.current === worker) { enemyWorker.current = null; setError(`적 AI 오류: ${event.message}`); }
        worker.terminate();
      };
      worker.postMessage(saved);
    }, 650);
    return () => {
      clearTimeout(timer);
      enemyWorker.current?.terminate();
      enemyWorker.current = null;
    };
  }, [screen, state, send]);
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
        setActiveAbility(null);
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
    const snapshot: Snapshot = JSON.parse(game.snapshot());
    if (!developerRun.current)
      setProgress((previous) => mergeProgress(previous, snapshot.progress ?? initialProgress));
    // Read the existing command history for presentation only. Promotions are
    // not rest upgrades and must not illuminate an unrelated upgrade edge.
    const commands: Command[] = JSON.parse(saved).commands;
    setUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" }> => c.type === "rest_upgrade"));
    setRewardUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" | "claim_upgrade" }> => c.type === "rest_upgrade" || c.type === "claim_upgrade").map(({ id, target }) => ({ id, target })));
    try {
      localStorage.setItem(SAVE_KEY, saved);
      localStorage.setItem(DEVELOPER_RUN_KEY, String(developerRun.current));
      if (!developerRun.current) {
        const nextProgress = mergeProgress(readProgress(), snapshot.progress ?? initialProgress);
        localStorage.setItem(PROGRESS_KEY, JSON.stringify(nextProgress));
        setProgress(nextProgress);
      }
      setHasSave(true);
      setSaveNotice("");
    } catch {
      setSaveNotice(
        "진행 상태를 저장하지 못했습니다. 이 탭을 닫으면 최근 진행을 잃을 수 있습니다.",
      );
    }
  }
  function toggleDeveloperMode() {
    const nextMode = !developerMode;
    setDeveloperMode(nextMode);
    if (!nextMode) {
      if (!progress.unlocked_squads.includes(character)) setCharacter("guardian");
      setStartFloor(1);
    }
    try {
      localStorage.setItem(DEVELOPER_MODE_KEY, String(nextMode));
      setSaveNotice("");
    } catch {
      setSaveNotice("개발자 모드 상태를 저장하지 못했습니다. 새로고침 후에는 다시 설정해 주세요.");
    }
  }
  function resume() {
    try {
      const saved = localStorage.getItem(SAVE_KEY);
      if (!saved) return;
      developerRun.current = localStorage.getItem(DEVELOPER_RUN_KEY) === "true";
      const next = WebGame.restore(saved);
      engine.current?.free();
      engine.current = next;
      setState(JSON.parse(next.snapshot()));
      setLastEnemyMove(null);
      setBoardAnnotations([]);
      setFloorTransition(null);
      persist(next);
      setRestChoice(null);
      setRewardUpgradeOpen(false);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setActiveAbility(null);
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
  function start() {
    try {
      const effectiveProgress = developerMode ? developerProgress(progress, choices.map((choice) => choice.id)) : progress;
      // Entropy only comes from the browser; the seeded Rust engine generates the entire Act.
      const seed = crypto.getRandomValues(new Uint32Array(1))[0];
      const next = (WebGame as typeof WebGame & { start_run(character: string, seed: number, floor: number, progress: string): WebGame })
        .start_run(character, seed, startFloor, JSON.stringify(effectiveProgress));
      developerRun.current = developerMode;
      engine.current?.free();
      engine.current = next;
      const snapshot: Snapshot = JSON.parse(next.snapshot());
      setLastEnemyMove(null);
      setBoardAnnotations([]);
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
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setActiveAbility(null);
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
    if (promotion) return;
    if (card.disabled_reason) {
      setError(card.disabled_reason);
      return;
    }
    setError("");
    setSelected(null);
    setEnemyInspection(null);
    setActiveAbility(null);
    setCardTargets([]);
    setCardUpgrades([]);
    setUpgradeFocus(null);
    if (card.definition.targeted) setTargetCard(card);
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
      const target = piece?.id ?? 10000 + cell;
      if (state.effect_choice.targets.includes(target)) send({ type: "resolve_effect", target });
      return;
    }
    if (targetCard) {
      const targetId = piece?.id ?? 10000 + cell;
      if (targetCard.targets.includes(targetId)) {
        if (targetCard.definition.selection_mode === "upgrade") {
          if (!piece) return;
          if (!cardUpgrades.some((u) => u.id === piece.id) && cardUpgrades.length >= state.mana) {
            setError("남은 마나보다 많은 기물을 선택할 수 없습니다.");
            return;
          }
          setUpgradeFocus(piece.id);
        } else if (targetCard.definition.selection_mode === "multiple") {
          setCardTargets((ids) => ids.includes(targetId) ? ids.filter((id) => id !== targetId) : [...ids, targetId]);
        } else if (["limiter_release", "sacred_domain"].includes(targetCard.definition.kind)) {
          setCardTargets([targetId]);
        } else {
          send({ type: "play_card", id: targetCard.id, target: targetId });
        }
      }
      return;
    }
    if (activeAbility) {
      const actor = state.pieces.find((candidate) => candidate.id === activeAbility.piece_id);
      if (actor && activeAbility.targets.some((target) => target.cell === cell)) {
        send({ type: "special_action", from: actor.cell, to: cell });
        return;
      }
      setActiveAbility(null);
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
  const bossPiece = state?.boss ? state.pieces.find((p) => p.key === "tribal_dictator") : undefined;
  const bossRisk = new Set(state?.boss?.plan?.area ?? []);
  const bossTargets = new Set(state?.boss?.plan?.targets.map((target) => target.cell) ?? []);
  const initiativeRemaining = selectedPiece?.owner === 1 ? state?.enemy_initiative_turns_left ?? 0 : state?.initiative_turns_left ?? 0;
  const enemyCaptures = new Set(enemyInspection?.moves.flatMap((m) => m.captures) ?? []);
  const enemyMoves = new Set(enemyInspection?.moves.map((m) => m.to) ?? []);
  const enemyRoutes = [...new Map((enemyInspection?.moves ?? []).filter((m) => m.route).map((m) => [m.route!.join(","), m.route!])).values()];
  const latestBattleStart = state?.log.map((line, index) => line.includes("전투 시작") ? index : -1).filter((index) => index >= 0).pop() ?? -1;
  const recentBossEvent = state?.boss ? state.log.slice(latestBattleStart + 1).reverse().find((line) => /부족의 독재자 HP|찢고 죽여라|제왕학|갈고리 반격|샷건/.test(line)) : undefined;
  const selectedMovement = selectedPiece ? movementInfo(selectedPiece) : null;
  const selectedAbilities = state?.abilities?.filter((ability) => ability.piece_id === selectedPiece?.id) ?? [];
  const focusedUpgrade = state?.pieces.find((p) => p.id === upgradeFocus);
  const stagedCount = targetCard?.definition.selection_mode === "upgrade" ? cardUpgrades.length : cardTargets.length;
  const stagedCost = targetCard?.definition.cost_per_target ? stagedCount : targetCard?.definition.cost ?? 0;
  function cancelCard() {
    setTargetCard(null);
    setCardTargets([]);
    setCardUpgrades([]);
    setUpgradeFocus(null);
    setError("");
  }
  function confirmCard() {
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
      {floorTransition && <div className={`floor-transition stage-${floorTransition.stage} act-${floorTransition.next.floor}`} role="status" aria-label={`${floorLabel(floorTransition.next.floor, floorTransition.next.final_battle)} 진입`}>
        <div className="floor-transition-title"><small>CHESSLAYER · EXPEDITION BEGINS</small><strong>{floorTransition.next.final_battle ? "FINALE" : `ACT ${["", "I", "II", "III", "IV"][floorTransition.next.floor]}`}</strong><span>{floorTransition.next.final_battle ? "부족의 독재자" : floorNames[floorTransition.next.floor]}</span><small>{floorTransition.next.character}</small></div>
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
          <span className="edition">v{version} / {screen === "sandbox" ? "SANDBOX" : screen === "game" && state ? floorLabel(state.floor, state.final_battle) : "EXPEDITION"}</span>
          {screen !== "sandbox" && <button
            className="text-button"
            onClick={() => setHelp(!help)}
            aria-expanded={help}
          >
            플레이 안내 {help ? "−" : "+"}
          </button>}
        </div>
      </header>
      {screen === "game" && state && !floorTransition && createPortal(<button className="primary expedition-status-trigger" onClick={() => setStatusOpen(true)} aria-haspopup="dialog" aria-expanded={statusOpen}>원정 현황</button>, document.body)}
      {help && screen !== "sandbox" && (
        <section className="guide">
          <strong>체크메이트 대신, 로얄을 포획하세요.</strong>
          <p>
            백 기물을 클릭하면 이동 가능한 칸이 표시됩니다. 카드 사용은 이동
            횟수를 쓰지 않습니다. 이동 행동을 모두 마치면 적이 행동합니다.
            ★는 로얄, ◇는 가호입니다.
          </p>
          <p>전투 보드에서 칸을 우클릭하면 표식을 남기고, 우클릭한 채 다른 칸으로 드래그하면 화살표를 그립니다. 같은 표식이나 화살표를 반복하면 지워집니다.</p>
          <p>
            맵에서 이어진 다음 노드를 선택하세요. 승리한 진형은 다음 전투로
            이어집니다. 적의 시작 칸과 겹친 아군은 역습으로 사라집니다. 로얄을
            잃으면 원정이 끝납니다. 포병대대는 모든 창고를 잃으면 패배합니다. 진행은 이 브라우저에 자동 저장됩니다.
          </p>
          <p>
            소멸: 사용 후 이번 전투에서 제외 · 보존: 턴 종료에도 손패 유지 ·
            선천성: 첫 손패 우선 · 휘발성: 쓰지 않고 턴 종료 시 소멸. 체크
            제한·캐슬링·앙파상은 없습니다. 폰 계열은 최종 랭크로 이동할 때
            승격할 기물을 선택합니다.
          </p>
          <p>기물을 선택하면 행마와 상태를 확인할 수 있습니다. 사격·제물 능력은 기물 정보에서 능력을 선택한 뒤 표시된 대상 칸을 누르세요. 식재료를 포획하면 공격자는 제자리에 남고 대상은 시체가 됩니다. 시체는 양 진영이 포획할 수 있습니다.</p>
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
              다섯 부대가 펼치는 네 층의 원정.
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
            <button className="resume-button" disabled={!ready} onClick={() => setScreen("compendium")}>도감 보기 →</button>
            <button className="resume-button" disabled={!ready} onClick={() => setScreen("sandbox")}>기물 테스트장 →</button>
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
            CHESSLAYER v{version} <span>길을 선택하고, 왕관을 되찾으세요.</span>
          </div>
        </main>
      )}
      {screen === "choose" && <SquadSelection choices={choices} character={character} onCharacter={setCharacter}
        floor={startFloor} onFloor={setStartFloor} progress={developerMode ? developerProgress(progress, choices.map((choice) => choice.id)) : progress} developerMode={developerMode}
        onBack={() => setScreen("home")} onStart={start} onToggleDeveloperMode={toggleDeveloperMode} ready={ready} />}
      {screen === "compendium" && <Compendium choices={choices} entries={compendiumEntries} onBack={() => setScreen("home")} />}
      {screen === "sandbox" && <PieceSandbox choices={choices} entries={compendiumEntries} onBack={() => setScreen("home")} />}
      {screen === "game" && state && showMap && (
        <ActMap state={state} send={send} coveredByReward={rewardRevisit} />
      )}
      {screen === "game" && state && !showMap && (
        <main className="game-layout" data-phase={state.phase} data-upgrade-open={upgradeOpen || undefined} data-card-upgrade-open={!!focusedUpgrade || undefined}>
          <nav className="node-navigation" aria-label="원정 위치">
            <span>
              {floorLabel(state.floor, state.final_battle)} / {(state.map.nodes.find((n) => n.current)?.row ?? 0) + 1}번째 지점
              · {state.node_kind && nodeTypes[state.node_kind].label}
            </span>
          </nav>
          {!["treasure", "event", "shop", "victory", "defeat"].includes(state.phase) && (
            <>
              <div className="battle-column">
              <section className="battle-area">
                <div className="battle-heading">
                  <div>
                    <p className="eyebrow">
                      EXPEDITION / {state.character}
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
                  <div className="board" role="group" aria-label="체스 보드"
                    onContextMenu={(event) => { if (state.phase === "combat") event.preventDefault(); }}
                    onPointerDown={startBoardAnnotation}
                    onPointerMove={moveBoardAnnotation}
                    onPointerUp={endBoardAnnotation}
                    onPointerCancel={cancelBoardAnnotation}>
                    {Array.from({ length: 64 }, (_, i) => {
                      const piece = boardPieces.find((p) => p.cell === i);
                      const legal =
                        !activeAbility &&
                        selected !== null &&
                        state.moves.some(
                          (m) => m.from === selected && m.to === i,
                        );
                      const enemyCapture = !!inspectedEnemy && enemyCaptures.has(i);
                      const enemyMove = !!inspectedEnemy && !piece && enemyMoves.has(i);
                      const abilityTarget = activeAbility?.targets.find((target) => target.cell === i);
                      const preview = activeAbility?.targets.find((target) => target.cell === hoveredAbilityTarget) ?? (activeAbility?.targets.length === 1 ? activeAbility.targets[0] : undefined);
                      const fallout = state.fallout.find((effect) => effect.cell === i);
                      const pathIndex = preview?.paths.findIndex((path) => path.includes(i)) ?? -1;
                      const targetId = piece?.id ?? 10000 + i;
                      const effectTarget = state.effect_choice?.targets.includes(targetId);
                      const target = targetCard?.targets.includes(targetId) || !!abilityTarget || effectTarget;
                      const staged = cardTargets.includes(targetId) || (piece && cardUpgrades.some((u) => u.id === piece.id));
                      const square = `${"abcdefgh"[i % 8]}${8 - Math.floor(i / 8)}`;
                      return (
                        <button
                          key={i}
                          data-square={square}
                          data-legal={legal || undefined}
                          data-enemy-move={enemyMove || undefined}
                          data-enemy-capture={enemyCapture || undefined}
                          data-last-enemy-move={state.phase === "combat" && (lastEnemyMove?.from === i || lastEnemyMove?.to === i) || undefined}
                          className={`square ${(i + Math.floor(i / 8)) % 2 ? "dark" : "light"} ${state.phase === "combat" && (lastEnemyMove?.from === i || lastEnemyMove?.to === i) ? "last-enemy-move" : ""} ${(restView ? restChoice?.cell : selected) === i ? "selected" : ""} ${inspectedEnemy?.cell === i ? "enemy-selected" : ""} ${restView && restChoice?.cell === i ? "rest-selected" : ""} ${legal ? "legal" : ""} ${target ? "targetable" : ""} ${staged ? "card-staged" : ""} ${fallout ? "fallout-cell" : ""} ${preview?.blast.includes(i) ? "blast-preview" : ""} ${pathIndex >= 0 ? `pierce-path path-${pathIndex}` : ""} ${bossRisk.has(i) ? "boss-risk" : ""} ${bossTargets.has(i) ? "boss-target" : ""} ${state.boss?.hook_target === i ? "boss-hook-target" : ""}`}
                          title={abilityTarget ? `${activeAbility?.name}${abilityTarget.chance !== null ? ` · 명중률 ${abilityTarget.chance}%` : ""}` : fallout ? `방사성 낙진 · 남은 ${fallout.remaining}라운드 · 턴 종료 시 제거` : piece?.statuses?.join(" · ") || (piece?.self_defense ? "호신용품: 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다." : undefined)}
                          onMouseEnter={() => { if (abilityTarget) setHoveredAbilityTarget(i); }}
                          onMouseLeave={() => { if (hoveredAbilityTarget === i) setHoveredAbilityTarget(null); }}
                          onClick={() => clickSquare(i)}
                          disabled={
                            !restView &&
                            (state.phase !== "combat" || !!promotion || (!!state.effect_choice && !effectTarget) || (!state.effect_choice && !turn && piece?.owner !== 1 && !(!piece && inspectedEnemy)))
                          }
                          aria-label={`${square}${piece ? ` ${piece.owner === 0 ? "백" : piece.owner === 1 ? "흑" : "중립"} ${piece.name}${piece.royal ? " 로얄" : ""}${piece.ward ? " 가호" : ""}${piece.self_defense ? " 호신용품 보호 대상" : ""}${piece.limiter ? ` 리미터 해제 남은 이동 ${piece.limiter.moves_left}회 0회가 되면 사망하고 턴 종료` : ""}` : " 빈 칸"}${fallout ? ` 방사성 낙진 ${fallout.remaining}라운드` : ""}${bossRisk.has(i) ? " 보스 예고 위험 범위" : ""}${state.boss?.hook_target === i ? " 갈고리 반격 대상" : ""}${legal ? " 이동 가능" : ""}${enemyMove ? " 적 이동 가능" : ""}${enemyCapture ? " 적 포획 가능" : ""}${staged ? " 카드 선택됨" : ""}`}
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
                          {bossTargets.has(i) && <span className="boss-target-mark" title="예고된 행동의 현재 대상">!</span>}
                          {state.boss?.hook_target === i && <span className="boss-hook-mark" title="갈고리 반격 대상">⚓</span>}
                          {abilityTarget && <span className="aim-target">{abilityTarget.chance === null ? "✦" : `${abilityTarget.chance}%`}</span>}
                          {fallout && <span className="fallout-badge" title={`방사성 낙진 · 남은 ${fallout.remaining}라운드`}>{fallout.remaining}</span>}
                          {staged && <span className="selection-number">{cardTargets.indexOf(targetId) + 1 || "✓"}</span>}
                        </button>
                      );
                    })}
                    {inspectedEnemy && enemyRoutes.length > 0 && <svg className="enemy-routes" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
                      {enemyRoutes.map((route) => <polyline key={route.join(",")} points={route.map((cell) => `${cell % 8 + 0.5},${Math.floor(cell / 8) + 0.5}`).join(" ")} />)}
                    </svg>}
                    {bossPiece && state.boss?.hook_target !== null && state.boss?.hook_target !== undefined && <svg className="boss-hook-line" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
                      <line x1={bossPiece.cell % 8 + .5} y1={Math.floor(bossPiece.cell / 8) + .5} x2={state.boss.hook_target % 8 + .5} y2={Math.floor(state.boss.hook_target / 8) + .5} />
                      <circle cx={state.boss.hook_target % 8 + .5} cy={Math.floor(state.boss.hook_target / 8) + .5} r=".17" />
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
                    {state.phase === "combat" && (boardAnnotations.length > 0 || annotationPreview) &&
                      <svg className="board-annotations" viewBox="0 0 8 8" preserveAspectRatio="none" aria-hidden="true">
                        <defs><marker id="board-annotation-head" markerWidth=".42" markerHeight=".42" refX=".36" refY=".21" viewBox="0 0 .42 .42" orient="auto" markerUnits="userSpaceOnUse"><path d="M 0 0 L .42 .21 L 0 .42 Z" /></marker></defs>
                        {[...boardAnnotations, ...(annotationPreview ? [annotationPreview] : [])].map(({ from, to }, index) =>
                          from === to
                            ? <circle key={`${from}-${to}-${index}`} className={`board-annotation-mark ${index === boardAnnotations.length ? "preview" : ""}`} data-cell={from} cx={from % 8 + .5} cy={Math.floor(from / 8) + .5} r=".38" />
                            : <line key={`${from}-${to}-${index}`} className={`board-annotation-arrow ${index === boardAnnotations.length ? "preview" : ""}`} data-from={from} data-to={to}
                                x1={from % 8 + .5} y1={Math.floor(from / 8) + .5} x2={to % 8 + .5} y2={Math.floor(to / 8) + .5} markerEnd="url(#board-annotation-head)" />)}
                      </svg>}
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
                {state.phase === "combat" && state.boss && <section className="boss-intent-panel" aria-label="부족의 독재자 상태와 행동 예고" aria-live="polite">
                  <strong key={state.boss.hp} className="boss-hp-value">부족의 독재자 · HP {state.boss.hp} · 샷건 {state.boss.ammo}/{state.boss.max_ammo}</strong>
                  <span>독재자 행동 예고: {state.boss.plan?.name ?? "행동 없음"}</span>
                  {state.boss.plan?.targets.length ? <span>독재자 현재 대상: {state.boss.plan.targets.map((target) => `${target.name} ${"abcdefgh"[target.cell % 8]}${8 - Math.floor(target.cell / 8)} (${target.chance}%)`).join(" · ")}</span> : null}
                  <span>갈고리 반격: {state.boss.hook_name ? `${state.boss.hook_name} 지정 · 다음 적 턴에 끌어옴` : "피해를 준 공격자를 다음 턴에 끌어옴"}</span>
                  <small>제왕학 · 주변 8칸 사망 시 행마 습득 / 찢고 죽여라 · 주변 사망마다 HP +1</small>
                  {state.boss.learned.length > 0 && <small>습득한 행마: {state.boss.learned.join(", ")}</small>}
                  {recentBossEvent && <small key={state.log.length} className="boss-event-flash">최근 효과: {recentBossEvent}</small>}
                </section>}
                {state.phase === "combat" && (state.effect_choice || targetCard || activeAbility) && (
                  <div
                    className={`action-hint ${targetCard ? "target-hint" : ""}`}
                    aria-live="polite"
                  >
                    {state.effect_choice ? <><strong>{state.effect_choice.name}</strong><span>표시된 대상을 선택하세요.</span>{state.effect_choice.can_finish && <button className="primary" onClick={() => send({ type: "resolve_effect", target: 0 })}>배치 완료</button>}</> : targetCard ? (
                      <>
                        <strong>{targetCard.definition.name}</strong>
                        {targetCard.definition.kind === "limiter_release" && <span>세 번째 이동 직후 대상 사망 · 턴 종료</span>}
                        <span>선택 {stagedCount}개 · 비용 {stagedCost} / {state.ammo !== null ? `탄약 ${state.ammo}` : `마나 ${state.mana}`}</span>
                        {targetCard.definition.selection_mode === "multiple" && <span className="card-target-instruction">{cardSelectionHint(targetCard.definition.kind)}</span>}
                        {targetCard.definition.kind === "emergency_conscription" && <span className="staged-upgrades">{state.rest_options.filter((piece) => piece.dead && targetCard.targets.includes(piece.id)).map((piece) => <button key={piece.id} onClick={() => setCardTargets((ids) => ids.includes(piece.id) ? ids.filter((id) => id !== piece.id) : [...ids.filter((id) => !state.rest_options.some((p) => p.dead && p.id === id)), piece.id])}>{cardTargets.includes(piece.id) ? "✓ " : ""}{piece.name} #{piece.id}</button>)}</span>}
                        {targetCard.definition.selection_mode === "upgrade" && cardUpgrades.length > 0 && <span className="staged-upgrades">{cardUpgrades.map((choice) => <button key={choice.id} onClick={() => setCardUpgrades((old) => old.filter((u) => u.id !== choice.id))}>{state.pieces.find((p) => p.id === choice.id)?.name} ×</button>)}</span>}
                        {stagedCount > 0 && (["multiple", "upgrade"].includes(targetCard.definition.selection_mode) || ["limiter_release", "sacred_domain"].includes(targetCard.definition.kind)) ? <button className="primary" onClick={confirmCard} disabled={stagedCost > state.mana}>확인</button> : null}
                        <button
                          className="text-button"
                          onClick={cancelCard}
                        >
                          취소 Esc
                        </button>
                      </>
                    ) : activeAbility ? <><strong>{activeAbility.name}</strong><span>표시된 대상 칸을 선택하세요.</span><button className="text-button" onClick={() => setActiveAbility(null)}>취소 Esc</button></> : null}
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
                          <strong>
                            {selectedPiece.name} {selectedPiece.royal && <span className="royal-info">★ 로얄</span>}
                          </strong>
                          {selectedMovement && <p><b>행마:</b> {selectedMovement.movement}</p>}
                          {selectedMovement?.special && <p>{selectedMovement.special}</p>}
                          {selectedPiece.key === "tribal_dictator" && state.boss?.learned.length ? <small className="effect-note">습득 행마: {state.boss.learned.join(", ")}</small> : null}
                          {selectedPiece.key === "tribal_dictator" && state.boss?.learned_abilities.map((ability) => <small className="effect-note" key={ability.name}><b>제왕학 · {ability.name}</b> — {ability.description}</small>)}
                          {turn && !inspectedEnemy && selectedAbilities.map((ability, index) => <button key={index} className={`ability-button ${activeAbility === ability ? "selected" : ""}`}
                            title={ability.description} onClick={() => { setTargetCard(null); setActiveAbility(activeAbility === ability ? null : ability); }}
                            disabled={ability.targets.length === 0 || !!state.effect_choice}>{ability.name} <span>{ability.targets.length ? "대상 선택 →" : "대상 없음"}</span></button>)}
                          <PieceStatuses statuses={selectedPiece.statuses} compact />
                          {initiativeRemaining > 0 && <small className="effect-note">선공권 보호 · 상대의 포획 제한 · 남은 상대 턴 {initiativeRemaining}회</small>}
                          {selectedPiece.temporary_upgrade && <small className="effect-note">일시 강화 · 현재 전투에서만 유지</small>}
                          {selectedPiece.ward && <small className="effect-note">가호 · 다음 포획 시 공격 무효</small>}
                          {selectedPiece.self_defense && <small className="effect-note">호신용품 · 직접 포획되면 포획한 적도 사망</small>}
                          {selectedPiece.absorbed_moves.length > 0 && <small className="effect-note">흡수한 행마 · {selectedPiece.absorbed_moves.join(", ")}</small>}
                          {selectedPiece.limiter && <small className="effect-note danger">리미터 해제 · 남은 이동 {selectedPiece.limiter.moves_left}회 · 0회가 되면 사망하고 턴 종료</small>}
                        </div>
                      </section>
                    )}
                    <div className="combat-resource" aria-label={state.ammo !== null ? `탄약 ${state.ammo}` : `마나 ${state.mana}`}>
                      <span className="mana-orb">{state.ammo ?? state.mana}</span>
                      <span>{state.ammo !== null ? `탄약 / ${state.ammo_capacity}` : "마나"}</span>
                    </div>
                    <details className="relic-panel">
                      <summary className="section-label">
                        보유 유물 <span>{state.relics.length}</span>
                      </summary>
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
                    </details>
                  </>
                )}
              </aside>
            </>
          )}
          {state.phase !== "combat" && state.phase !== "rest" && state.phase !== "reward" && (
            <section className="outcome" ref={outcomeRef} aria-live="polite">
              <LocationPanel state={state} send={send} />
              {state.phase === "treasure" && (
                <>
                  <p className="eyebrow">A GIFT ALONG THE WAY</p>
                  <h2>남겨진 보물</h2>
                  {state.node_reward && <div className="treasure-relic"><RelicIcon id={state.node_reward.id} name={state.node_reward.name} /></div>}
                  <h3>{state.node_reward?.name}</h3>
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
                      ? state.final_battle ? "마지막 왕관까지." : `${state.floor}층 수복 완료`
                      : "왕이 쓰러졌습니다."}
                  </h2>
                  <p>
                    {state.phase === "victory"
                      ? state.final_battle ? "부족의 독재자를 쓰러뜨리고 모든 영토를 수복했습니다." : `${state.encounter} 보스를 쓰러뜨렸습니다. 해금된 부대와 새로운 원정을 시작하세요.`
                      : "진형은 여기에서 멈추지만, 다음 원정의 한 수는 달라질 수 있습니다."}
                  </p>
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
      {screen === "game" && state?.phase === "reward" && !floorTransition && !rewardUpgradeOpen && (
        <BattleRewards state={state} send={send} revisit={!!rewardRevisit} onUpgrade={() => { setRestChoice(null); setRewardUpgradeOpen(true); }} />
      )}
      <footer>
        CHESSLAYER <span>원정과 부대 해금은 이 브라우저에 저장됩니다.</span>
      </footer>
    </>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

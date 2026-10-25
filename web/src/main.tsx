import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import init, { WebGame, catalog } from "./wasm/chesslayer_web";
import type {
  Card,
  Character,
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
import { UpgradeTree } from "./UpgradeTree";
import { RelicIcon } from "./ContentArt";
import { CardFace } from "./CardFace";
import { BattleRewards } from "./BattleRewards";
import { version } from "../package.json";

// Keep the original storage slot so legacy saves receive the WASM version notice.
const SAVE_KEY = "chesslayer.run.v1";
const starterRelicNames: Record<Character["relic"], string> = {
  haste: "가속", whirlwind: "소용돌이", self_defense: "호신용품",
};

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
      {piece.limiter && <span className="limiter-mark" title={`리미터 해제: ${piece.limiter.active ? `남은 이동 ${piece.limiter.moves_left}회` : "다음 소속 턴에 3회"} · 턴 종료 후 사망`}>{piece.limiter.active ? piece.limiter.moves_left : "!"}</span>}
    </span>
  );
}
function App() {
  const engine = useRef<WebGame | null>(null);
  const [ready, setReady] = useState(false);
  const [choices, setChoices] = useState<Character[]>([]);
  const [screen, setScreen] = useState<"home" | "choose" | "game">("home");
  const [character, setCharacter] = useState("vanguard");
  const [state, setState] = useState<Snapshot | null>(null);
  const [selected, setSelected] = useState<number | null>(null);
  const [enemyInspection, setEnemyInspection] = useState<EnemyInspection | null>(null);
  const [restChoice, setRestChoice] = useState<RestChoice | null>(null);
  const [restCompleted, setRestCompleted] = useState(false);
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
  const [mapOpen, setMapOpen] = useState(false);
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
    if (state?.phase && state.phase !== previousPhase.current && !restCompleted) {
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
  }, [state?.phase, restCompleted]);
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
      if (command.type === "rest_upgrade") {
        setRestCompleted(true);
        setRestChoice((choice) => choice && { ...choice, target: command.target });
      } else {
        setRestChoice(null);
        setRestCompleted(false);
      }
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setTargetCard(null);
      setCardTargets([]);
      setCardUpgrades([]);
      setUpgradeFocus(null);
      setMapOpen(false);
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
        setSelected(null);
        setEnemyInspection(null);
        setRestChoice(null);
        setRestCompleted(false);
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
  }, []);
  function persist(game: WebGame) {
    const saved = game.save();
    // Read the existing command history for presentation only. Promotions are
    // not rest upgrades and must not illuminate an unrelated upgrade edge.
    const commands: Command[] = JSON.parse(saved).commands;
    setUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" }> => c.type === "rest_upgrade"));
    setRewardUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" | "claim_upgrade" }> => c.type === "rest_upgrade" || c.type === "claim_upgrade").map(({ id, target }) => ({ id, target })));
    try {
      localStorage.setItem(SAVE_KEY, saved);
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
      setRestCompleted(false);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setTargetCard(null);
      setCardTargets([]);
      setCardUpgrades([]);
      setUpgradeFocus(null);
      setMapOpen(false);
      setError("");
      setScreen("game");
    } catch (e) {
      setError(String(e));
    }
  }
  function start() {
    try {
      // Entropy only comes from the browser; the seeded Rust engine generates the entire Act.
      const seed = crypto.getRandomValues(new Uint32Array(1))[0];
      const next = new WebGame(character, seed);
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
      setRestCompleted(false);
      setPromotion(null);
      setSelected(null);
      setEnemyInspection(null);
      setTargetCard(null);
      setCardTargets([]);
      setCardUpgrades([]);
      setUpgradeFocus(null);
      setMapOpen(false);
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
    setCardTargets([]);
    setCardUpgrades([]);
    setUpgradeFocus(null);
    if (card.definition.targeted) setTargetCard(card);
    else send({ type: "play_card", id: card.id });
  }
  function clickSquare(cell: number) {
    if (!state || promotion) return;
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
    if (targetCard) {
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
  const showMap = (state?.phase === "map" || mapOpen || rewardRevisit) && (!restCompleted || mapOpen);
  const restPiece = state?.rest_options.find((p) => p.id === restChoice?.id);
  const restView = state?.phase === "rest" || restCompleted;
  const upgradeOpen = restView && !!restPiece && !restPiece.dead;
  function closeRest() {
    setRestChoice(null);
    setRestCompleted(false);
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
  return (
    <>
      {floorTransition && <div className={`floor-transition stage-${floorTransition.stage} act-${floorTransition.next.floor}`} role="status" aria-label={`제${floorTransition.next.floor}층 진입`}>
        <div className="floor-transition-title"><small>CHESSLAYER · {floorTransition.next.floor === 1 ? "EXPEDITION BEGINS" : "NEXT ACT"}</small><strong>ACT {floorTransition.next.floor === 1 ? "I" : "II"}</strong><span>{floorTransition.next.floor === 1 ? "제1막 · 갈림길의 숲" : "제2층에 도착했습니다"}</span><small>{floorTransition.next.floor === 1 ? "원정 시작" : "아군 전원 회복"}</small></div>
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
          <span className="edition">FIRST EXPEDITION / {screen === "game" && state?.floor === 2 ? "ACT II" : "ACT I"}</span>
          <button
            className="text-button"
            onClick={() => setHelp(!help)}
            aria-expanded={help}
          >
            플레이 안내 {help ? "−" : "+"}
          </button>
        </div>
      </header>
      {screen === "game" && state && <button className="primary expedition-status-trigger" onClick={() => setMapOpen(true)}>원정 현황</button>}
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
          <p className="eyebrow">CHOOSE YOUR EXPEDITION</p>
          <h1>어떤 수를 두시겠어요?</h1>
          <p className="muted">
            모두 같은 16기의 백 진형으로 시작합니다. 시작 유물이 플레이를
            바꿉니다.
          </p>
          <div className="characters">
            {choices.map((c) => (
              <button
                className={`character ${character === c.id ? "chosen" : ""}`}
                key={c.id}
                onClick={() => setCharacter(c.id)}
                aria-pressed={character === c.id}
              >
                <RelicIcon id={c.relic} name={starterRelicNames[c.relic]} className="character-icon" />
                <span className="eyebrow">{c.title}</span>
                <h2>{c.name}</h2>
                <p>{c.description}</p>
                <span className="character-radio">
                  {character === c.id ? "● 선택됨" : "○ 선택"}
                </span>
              </button>
            ))}
          </div>
          <div className="launch">
            <span className="muted">
              빈 시작 덱 · 턴마다 1장 드로우 · 기본 마나 3
            </span>
            <button className="primary large" onClick={start} disabled={!ready}>
              런 시작 <span>→</span>
            </button>
          </div>
        </main>
      )}
      {screen === "game" && state && showMap && (
        <ActMap state={state} send={send} onClose={() => setMapOpen(false)} coveredByReward={rewardRevisit} overlay={mapOpen} />
      )}
      {screen === "game" && state && (!showMap || (mapOpen && state.phase !== "map")) && (
        <main className="game-layout" data-phase={state.phase} data-upgrade-open={upgradeOpen || undefined}>
          <nav className="node-navigation" aria-label="원정 위치">
            <span>
              {state.floor}층 / {state.map.nodes.find((n) => n.current)!.row + 1}번째 지점
              · {state.node_kind && nodeTypes[state.node_kind].label}
            </span>
          </nav>
          {state.phase !== "treasure" && (
            <>
              <section className="battle-area">
                <div className="battle-heading">
                  <div>
                    <p className="eyebrow">
                      FIRST EXPEDITION / {state.character}
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
                      restCompleted ? "쉼터 · 강화 완료" : "쉼터 · 행동 1회"
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
                      const piece = state.pieces.find((p) => p.cell === i);
                      const legal =
                        selected !== null &&
                        state.moves.some(
                          (m) => m.from === selected && m.to === i,
                        );
                      const enemyCapture = !!inspectedEnemy && enemyCaptures.has(i);
                      const enemyMove = !!inspectedEnemy && !piece && enemyMoves.has(i);
                      const target =
                        piece && targetCard?.targets.includes(piece.id);
                      const staged = piece && (cardTargets.includes(piece.id) || cardUpgrades.some((u) => u.id === piece.id));
                      const square = `${"abcdefgh"[i % 8]}${8 - Math.floor(i / 8)}`;
                      return (
                        <button
                          key={i}
                          data-square={square}
                          data-legal={legal || undefined}
                          data-enemy-move={enemyMove || undefined}
                          data-enemy-capture={enemyCapture || undefined}
                          className={`square ${(i + Math.floor(i / 8)) % 2 ? "dark" : "light"} ${(state.phase === "rest" ? restChoice?.cell : selected) === i ? "selected" : ""} ${inspectedEnemy?.cell === i ? "enemy-selected" : ""} ${state.phase === "rest" && restChoice?.cell === i ? "rest-selected" : ""} ${legal ? "legal" : ""} ${target ? "targetable" : ""} ${staged ? "card-staged" : ""}`}
                          title={piece?.self_defense ? "호신용품: 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다." : undefined}
                          onClick={() => clickSquare(i)}
                          disabled={
                            state.phase !== "rest" &&
                            (state.phase !== "combat" || !!promotion || (!turn && piece?.owner !== 1 && !(!piece && inspectedEnemy)))
                          }
                          aria-label={`${square}${piece ? ` ${piece.owner === 0 ? "백" : "흑"} ${piece.name}${piece.royal ? " 로얄" : ""}${piece.ward ? " 가호" : ""}${piece.self_defense ? " 호신용품 보호 대상" : ""}${piece.limiter ? ` 리미터 해제 남은 이동 ${piece.limiter.moves_left}회 효과 종료 후 사망` : ""}` : " 빈 칸"}${legal ? " 이동 가능" : ""}${enemyMove ? " 적 이동 가능" : ""}${enemyCapture ? " 적 포획 가능" : ""}${staged ? " 카드 선택됨" : ""}`}
                          aria-pressed={
                            (state.phase === "rest"
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
                      {state.pieces.map((p) => (
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
                <div className="board-caption">
                  <span>백 · 나의 진형</span>
                  <span>
                    {restView
                      ? "강화 · 부활은 확정 후 적용"
                      : state.phase === "combat" && state.hard_disk_deployed
                        ? "하드디스크 · 직전 전투 종료 배치 적용"
                        : state.phase === "combat" && state.enemy_initiative_turns_left > 0
                          ? `★ 적 로얄: ${state.royal_name} · 적 선공권 ${state.enemy_initiative_turns_left}턴`
                        : `★ 적 로얄: ${state.royal_name}`}
                  </span>
                </div>
                {state.phase === "combat" && (
                  <div
                    className={`action-hint ${targetCard ? "target-hint" : ""}`}
                    aria-live="polite"
                  >
                    {targetCard ? (
                      <>
                        <strong>{targetCard.definition.name}</strong> · 표시된 기물을 선택하세요.
                        {targetCard.definition.kind === "limiter_release" && <span>효과 종료 후 대상 사망</span>}
                        <span>선택 {stagedCount}기 · 현재 코스트 {stagedCost} / 마나 {state.mana}</span>
                        {targetCard.definition.selection_mode === "upgrade" && cardUpgrades.length > 0 && <span className="staged-upgrades">{cardUpgrades.map((choice) => <button key={choice.id} onClick={() => setCardUpgrades((old) => old.filter((u) => u.id !== choice.id))}>{state.pieces.find((p) => p.id === choice.id)?.name} ×</button>)}</span>}
                        {["multiple", "upgrade"].includes(targetCard.definition.selection_mode) || ["limiter_release", "sacred_domain"].includes(targetCard.definition.kind) ? <button className="primary" onClick={confirmCard} disabled={stagedCount < 1 || stagedCost > state.mana}>확인</button> : null}
                        <button
                          className="text-button"
                          onClick={cancelCard}
                        >
                          취소 Esc
                        </button>
                      </>
                    ) : selected !== null ? (
                      <span>표시된 칸을 클릭해 이동하세요.{state.pieces.find((p) => p.cell === selected)?.self_defense && <strong className="self-defense-info"> ◆ 호신용품: 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다.</strong>}</span>
                    ) : inspectedEnemy ? (
                      <span>적 행마 조회 · 파란 원은 빈 도착 칸, 빨간 테두리는 포획 대상입니다. 점선은 확인된 경유 경로이며, 적의 다음 행동을 예측하지 않습니다.</span>
                    ) : turn ? (
                      "카드를 사용하거나 기물을 선택하세요. 적 기물을 누르면 행마를 확인할 수 있습니다."
                    ) : (
                      "상대가 다음 수를 고르고 있습니다…"
                    )}
                  </div>
                )}
              </section>
              <aside className="sidebar">
                {restView ? (
                  <RestPanel
                    state={state}
                    choice={restChoice}
                    onChoose={setRestChoice}
                    onCancel={closeRest}
                    send={send}
                    history={upgradeHistory}
                    completed={restCompleted}
                    onContinue={closeRest}
                  />
                ) : (
                  <>
                    {targetCard?.definition.selection_mode === "upgrade" && focusedUpgrade && <section className="card-upgrade-picker">
                      <p>일시 강화 경로 선택 · {focusedUpgrade.name}</p>
                      <UpgradeTree key={focusedUpgrade.id} piece={{ ...focusedUpgrade, origin: focusedUpgrade.cell, upgrades: [], can_upgrade: true, can_revive: false, dead: false }} paths={state.upgrade_paths} acquired={[]}
                        onSelect={() => {}}
                        onConfirm={(target) => { setCardUpgrades((old) => [...old.filter((u) => u.id !== focusedUpgrade.id), { id: focusedUpgrade.id, target }]); setUpgradeFocus(null); }}
                        completed={false} selectionOnly />
                    </section>}
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
                          {selectedPiece.royal && <span className="royal-info">★ 로얄 · {inspectedEnemy ? "이 기물은 승리 목표입니다." : "이 기물을 잃으면 원정이 끝납니다."}</span>}
                          <small className="piece-info-heading">행마법 · 특수 능력</small>
                          <p>{selectedPiece.description}</p>
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
                        <span>기본 마나 3</span>
                      </div>
                      <div className="mana">
                        <span className="mana-orb">{state.mana}</span>
                        <div>
                          <strong>사용 가능한 마나</strong>
                          <small>{state.mana_rule}</small>
                        </div>
                      </div>
                      <button
                        className="primary end-turn"
                        onClick={() => send({ type: "end_turn" })}
                        disabled={!turn || state.phase !== "combat" || !!promotion}
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
          {state.phase === "combat" && (
            <section className="hand-area">
              <div className="hand-heading">
                <div>
                  <h2>
                    손패 <span>{state.hand.length}/10</span>
                  </h2>
                  <small>턴마다 1장 드로우 · 종료 시 손패 5장 이상이면 일반 카드를 버립니다.</small>
                </div>
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
                {state.hand.length === 0 && <p className="empty-hand">손패가 비어 있습니다. 다음 턴에 덱에서 1장을 뽑습니다.</p>}
                {state.hand.map((card) => (
                  <button
                    data-card-id={card.id}
                    className={`playing-card ${targetCard?.id === card.id ? "active-card" : ""}`}
                    key={card.id}
                    onClick={() => chooseCard(card)}
                    disabled={!!card.disabled_reason || !!promotion}
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
          {state.phase !== "combat" && state.phase !== "rest" && state.phase !== "reward" && !restCompleted && (
            <section className="outcome" ref={outcomeRef} aria-live="polite">
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
                      ? "마지막 왕관까지."
                      : "왕이 쓰러졌습니다."}
                  </h2>
                  <p>
                    {state.phase === "victory"
                      ? `${state.encounter} 보스를 쓰러뜨렸습니다. 2층 원정을 완수했습니다.`
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
      {screen === "game" && state?.phase === "reward" && !floorTransition && (
        <BattleRewards state={state} send={send} revisit={!!rewardRevisit} history={rewardUpgradeHistory} />
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

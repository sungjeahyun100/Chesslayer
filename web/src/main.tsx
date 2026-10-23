import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import init, { WebGame, catalog } from "./wasm/chesslayer_web";
import type {
  Card,
  CardDefinition,
  Character,
  Command,
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
import { CardArt, RelicIcon } from "./ContentArt";
import { version } from "../package.json";

// Keep the original storage slot so legacy saves receive the WASM version notice.
const SAVE_KEY = "chesslayer.run.v1";
const starterRelicNames: Record<Character["relic"], string> = {
  haste: "가속", whirlwind: "소용돌이", self_defense: "호신용품",
};

function Tags({ card }: { card: CardDefinition }) {
  return (
    <div className="tags">
      {card.innate && <span>선천성</span>}
      {card.exhaust && <span>소멸</span>}
      {card.retain && <span>보존</span>}
      {card.ethereal && <span>휘발성</span>}
    </div>
  );
}
function CardFace({ card, selectedCount = 0 }: { card: CardDefinition; selectedCount?: number }) {
  return (
    <>
      <div className="card-top">
        <span className="cost" aria-label={card.cost_per_target ? `선택 비용 ${selectedCount}` : `비용 ${card.cost}`}>
          {card.cost_per_target ? (selectedCount || "N") : card.cost}
        </span>
        <span>전술 카드</span>
      </div>
      <div className="card-symbol"><CardArt kind={card.kind} name={card.name} /></div>
      <h3>{card.name}</h3>
      <p>{card.description}</p>
      <Tags card={card} />
    </>
  );
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
  const [restChoice, setRestChoice] = useState<RestChoice | null>(null);
  const [restCompleted, setRestCompleted] = useState(false);
  const [upgradeHistory, setUpgradeHistory] = useState<Extract<Command, { type: "rest_upgrade" }>[]>([]);
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
  const [upgradePreview, setUpgradePreview] = useState<number | undefined>();
  const [error, setError] = useState("");
  const [help, setHelp] = useState(false);
  const [mapOpen, setMapOpen] = useState(false);
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
      setState(JSON.parse(engine.current.dispatch(JSON.stringify(command))));
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
      persist(next);
      setRestChoice(null);
      setRestCompleted(false);
      setPromotion(null);
      setSelected(null);
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
      setState(JSON.parse(next.snapshot()));
      persist(next);
      setRestChoice(null);
      setRestCompleted(false);
      setPromotion(null);
      setSelected(null);
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
  function chooseCard(card: Card) {
    if (promotion) return;
    if (card.disabled_reason) {
      setError(card.disabled_reason);
      return;
    }
    setError("");
    setSelected(null);
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
    if (state.phase !== "combat" || state.turn !== 0) return;
    const piece = state.pieces.find((p) => p.cell === cell);
    if (targetCard) {
      if (piece && targetCard.targets.includes(piece.id)) {
        if (targetCard.definition.selection_mode === "upgrade") {
          if (!cardUpgrades.some((u) => u.id === piece.id) && cardUpgrades.length >= state.mana) {
            setError("남은 마나보다 많은 기물을 선택할 수 없습니다.");
            return;
          }
          setUpgradeFocus(piece.id);
          setUpgradePreview(cardUpgrades.find((u) => u.id === piece.id)?.target);
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
    setSelected(piece?.owner === 0 && selected !== cell ? cell : null);
  }
  const turn = state?.turn === 0;
  const showMap = (state?.phase === "map" || mapOpen) && !restCompleted;
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
  const selectedPiece = state?.pieces.find((p) => p.cell === selected);
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
          <span className="edition">FIRST EXPEDITION / 01</span>
          <button
            className="text-button"
            onClick={() => setHelp(!help)}
            aria-expanded={help}
          >
            플레이 안내 {help ? "−" : "+"}
          </button>
        </div>
      </header>
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
            제한·캐슬링·앙파상은 없습니다. 강화된 폰은 최종 랭크로 이동할 때
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
        <ActMap state={state} send={send} onClose={() => setMapOpen(false)} />
      )}
      {screen === "game" && state && !showMap && (
        <main className="game-layout" data-phase={state.phase} data-upgrade-open={upgradeOpen || undefined}>
          <nav className="node-navigation" aria-label="원정 위치">
            <span>
              ACT I / {state.map.nodes.find((n) => n.current)!.row + 1}번째 지점
              · {state.node_kind && nodeTypes[state.node_kind].label}
            </span>
            <button className="text-button" onClick={() => { setRestCompleted(false); setMapOpen(true); }}>
              전체 지도 보기 ↗
            </button>
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
                      const target =
                        piece && targetCard?.targets.includes(piece.id);
                      const staged = piece && (cardTargets.includes(piece.id) || cardUpgrades.some((u) => u.id === piece.id));
                      const square = `${"abcdefgh"[i % 8]}${8 - Math.floor(i / 8)}`;
                      return (
                        <button
                          key={i}
                          data-square={square}
                          data-legal={legal || undefined}
                          className={`square ${(i + Math.floor(i / 8)) % 2 ? "dark" : "light"} ${(state.phase === "rest" ? restChoice?.cell : selected) === i ? "selected" : ""} ${state.phase === "rest" && restChoice?.cell === i ? "rest-selected" : ""} ${legal ? "legal" : ""} ${target ? "targetable" : ""} ${staged ? "card-staged" : ""}`}
                          title={piece?.self_defense ? "호신용품: 적에게 직접 포획되어 사망하면 포획한 적도 사망합니다." : undefined}
                          onClick={() => clickSquare(i)}
                          disabled={
                            state.phase !== "rest" &&
                            (state.phase !== "combat" || !turn || !!promotion)
                          }
                          aria-label={`${square}${piece ? ` ${piece.owner === 0 ? "백" : "흑"} ${piece.name}${piece.royal ? " 로얄" : ""}${piece.ward ? " 가호" : ""}${piece.self_defense ? " 호신용품 보호 대상" : ""}${piece.limiter ? ` 리미터 해제 남은 이동 ${piece.limiter.moves_left}회 효과 종료 후 사망` : ""}` : " 빈 칸"}${legal ? " 이동 가능" : ""}${staged ? " 카드 선택됨" : ""}`}
                          aria-pressed={
                            (state.phase === "rest"
                              ? restChoice?.cell
                              : selected) === i
                          }
                        >
                          {legal && (
                            <span
                              className={piece ? "capture-dot" : "move-dot"}
                            />
                          )}
                        </button>
                      );
                    })}
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
                    ) : turn ? (
                      "카드를 사용하거나, 백 기물을 선택하세요."
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
                      <UpgradeTree key={focusedUpgrade.id} piece={{ ...focusedUpgrade, origin: focusedUpgrade.cell, upgrades: [], can_upgrade: true, can_revive: false, dead: false }} paths={state.upgrade_paths} acquired={[]} selected={upgradePreview}
                        onSelect={setUpgradePreview}
                        onConfirm={(target) => { setCardUpgrades((old) => [...old.filter((u) => u.id !== focusedUpgrade.id), { id: focusedUpgrade.id, target }]); setUpgradeFocus(null); }}
                        onCancel={() => setUpgradeFocus(null)} completed={false} selectionOnly />
                    </section>}
                    {selectedPiece && (
                      <section className="selected-piece-info">
                        <PieceImage
                          pieceKey={selectedPiece.key}
                          owner={selectedPiece.owner}
                        />
                        <div>
                          <strong>
                            {selectedPiece.name} · {selectedPiece.square}
                          </strong>
                          <p>{selectedPiece.description}</p>
                          {selectedPiece.temporary_upgrade && <small className="effect-note">일시 강화 · 현재 전투에서만 유지</small>}
                          {selectedPiece.ward && <small className="effect-note">가호 · 다음 포획 시 공격 무효</small>}
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
          {state.phase !== "combat" && state.phase !== "rest" && !restCompleted && (
            <section className="outcome" ref={outcomeRef} aria-live="polite">
              {state.phase === "reward" && (
                <>
                  <p className="eyebrow">ROYAL DEFEATED</p>
                  <h2>전투 승리</h2>
                  <p>
                    생존 기물은 시작 칸으로 돌아가며(하드디스크 보유 시 종료 위치 유지) 영구 강화는 유지됩니다. 카드 또는 유물 보상
                    3개 중 하나를 선택하세요.
                  </p>
                  {state.node_kind === "elite" && state.node_reward && (
                    <p className="elite-reward">
                      추가 보상: {state.node_reward.name} ·{" "}
                      {state.node_reward.description}
                      {" "}선택 보상과 유물이 중복되면 가호 카드로 대체됩니다.
                    </p>
                  )}
                  <div className="reward-cards">
                    {state.rewards.map((reward, i) => (
                      <button
                        className={`playing-card ${reward.type === "relic" ? "relic-reward" : ""}`}
                        key={i}
                        data-reward-type={reward.type}
                        onClick={() =>
                          send({ type: "choose_reward", index: i })
                        }
                        aria-label={`${reward.type === "card" ? "카드" : "유물"} 보상 ${reward.definition.name}`}
                      >
                        {reward.type === "card" ? (
                          <CardFace card={reward.definition} />
                        ) : (
                          <>
                            <div className="card-top"><span className="relic-type">유물</span><span>영구 효과</span></div>
                            <div className="card-symbol"><RelicIcon id={reward.definition.id} name={reward.definition.name} /></div>
                            <h3>{reward.definition.name}</h3>
                            <p>{reward.definition.description}</p>
                          </>
                        )}
                        <span className="reward-select">
                          {reward.type === "card" ? "덱에 추가 +" : "유물 획득 +"}
                        </span>
                      </button>
                    ))}
                  </div>
                </>
              )}
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
                      ? `${state.encounter} 보스를 쓰러뜨렸습니다. 첫 번째 원정을 완수했습니다.`
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

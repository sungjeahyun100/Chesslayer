import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import init, { WebGame, catalog } from "./wasm/chesslayer_web";
import type {
  Card,
  CardDefinition,
  Character,
  Command,
  Piece,
  Snapshot,
} from "./types";
import "./style.css";
import { ActMap, nodeTypes } from "./ActMap";

const icons = ["", "♙", "♘", "♗", "♖", "♕", "♔", "♕", "♙"];
const blackIcons = ["", "♟", "♞", "♝", "♜", "♛", "♚", "♛", "♟"];
const relicIcons = { haste: "»", whirlwind: "◎", self_defense: "◇" };
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
function CardFace({ card }: { card: CardDefinition }) {
  return (
    <>
      <div className="card-top">
        <span className="cost">{card.cost}</span>
        <span>전술 카드</span>
      </div>
      <div className="card-symbol" aria-hidden="true">
        {card.kind === "jeans" ? "♖ × ♕" : card.kind === "gun" ? "♖ ♖ ♕" : "◇"}
      </div>
      <h3>{card.name}</h3>
      <p>{card.description}</p>
      <Tags card={card} />
    </>
  );
}
function PieceGlyph({ piece }: { piece: Piece }) {
  return (
    <span
      className={`piece owner-${piece.owner} ${piece.ward ? "warded" : ""}`}
      style={{
        left: `${(piece.cell % 8) * 12.5}%`,
        top: `${Math.floor(piece.cell / 8) * 12.5}%`,
      }}
      data-piece-id={piece.id}
      data-cell={piece.cell}
      aria-hidden="true"
    >
      <span className="glyph">
        {(piece.owner === 0 ? icons : blackIcons)[piece.kind]}
      </span>
      {piece.royal && <span className="royal-mark">★</span>}
      {piece.kind === 7 && <span className="piece-mod">♞</span>}
      {piece.kind === 8 && <span className="piece-mod">ϟ</span>}
      {piece.ward && <span className="ward-mark">◇</span>}
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
  const [targetCard, setTargetCard] = useState<Card | null>(null);
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
      if (state.phase !== "combat")
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
      setState(JSON.parse(engine.current.dispatch(JSON.stringify(command))));
      setSelected(null);
      setTargetCard(null);
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
        setTargetCard(null);
        setError("");
      }
    };
    window.addEventListener("keydown", cancel);
    return () => window.removeEventListener("keydown", cancel);
  }, []);
  function start() {
    try {
      // Entropy only comes from the browser; the seeded Rust engine generates the entire Act.
      const seed = crypto.getRandomValues(new Uint32Array(1))[0];
      const next = new WebGame(character, seed);
      engine.current?.free();
      engine.current = next;
      setState(JSON.parse(next.snapshot()));
      setSelected(null);
      setTargetCard(null);
      setMapOpen(false);
      setError("");
      setScreen("game");
    } catch (e) {
      setError(String(e));
    }
  }
  function chooseCard(card: Card) {
    if (card.disabled_reason) {
      setError(card.disabled_reason);
      return;
    }
    setError("");
    setSelected(null);
    if (card.definition.targeted) setTargetCard(card);
    else send({ type: "play_card", id: card.id });
  }
  function clickSquare(cell: number) {
    if (!state || state.phase !== "combat" || state.turn !== 0) return;
    const piece = state.pieces.find((p) => p.cell === cell);
    if (targetCard) {
      if (piece && targetCard.targets.includes(piece.id))
        send({ type: "play_card", id: targetCard.id, target: piece.id });
      return;
    }
    if (
      selected !== null &&
      state.moves.some((m) => m.from === selected && m.to === cell)
    ) {
      send({ type: "move", from: selected, to: cell });
      return;
    }
    setSelected(piece?.owner === 0 && selected !== cell ? cell : null);
  }
  const turn = state?.turn === 0;
  const showMap = state?.phase === "map" || mapOpen;
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
          <span>♞</span> CHESSLAYER
        </a>
        <div className="top-right">
          <span className="edition">PLAYABLE PROTOTYPE / 01</span>
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
            잃으면 원정이 끝납니다. 새로고침하면 진행은 초기화됩니다.
          </p>
          <p>
            소멸: 사용 후 이번 전투에서 제외 · 보존: 턴 종료에도 손패 유지 ·
            선천성: 첫 손패 우선 · 휘발성: 쓰지 않고 턴 종료 시 소멸. 체크
            제한·캐슬링·앙파상·자동 승격은 없습니다.
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
      {screen === "home" && (
        <main className="home">
          <section className="home-copy">
            <p className="eyebrow">A CHESS ROGUELIKE</p>
            <h1>
              한 수의 끝이,
              <br />
              <em>다음 전투의 시작.</em>
            </h1>
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
            <span className="hero-king">♔</span>
            <span className="hero-knight">♞</span>
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
                <span className="character-icon">{relicIcons[c.relic]}</span>
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
              카드 10장 · 기본 마나 3 · 영구적인 기물 손실
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
        <main className="game-layout" data-phase={state.phase}>
          <nav className="node-navigation" aria-label="원정 위치">
            <span>
              ACT I / {state.map.nodes.find((n) => n.current)!.row + 1}번째 지점
              · {state.node_kind && nodeTypes[state.node_kind].label}
            </span>
            <button className="text-button" onClick={() => setMapOpen(true)}>
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
                      const square = `${"abcdefgh"[i % 8]}${8 - Math.floor(i / 8)}`;
                      return (
                        <button
                          key={i}
                          data-square={square}
                          data-legal={legal || undefined}
                          className={`square ${(i + Math.floor(i / 8)) % 2 ? "dark" : "light"} ${selected === i ? "selected" : ""} ${legal ? "legal" : ""} ${target ? "targetable" : ""}`}
                          onClick={() => clickSquare(i)}
                          disabled={state.phase !== "combat" || !turn}
                          aria-label={`${square}${piece ? ` ${piece.owner === 0 ? "백" : "흑"} ${piece.name}${piece.royal ? " 로얄" : ""}${piece.ward ? " 가호" : ""}` : " 빈 칸"}${legal ? " 이동 가능" : ""}`}
                          aria-pressed={selected === i}
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
                        <PieceGlyph key={p.id} piece={p} />
                      ))}
                    </div>
                  </div>
                  <div className="file-labels">
                    {"abcdefgh".split("").map((f) => (
                      <span key={f}>{f}</span>
                    ))}
                  </div>
                </div>
                <div className="board-caption">
                  <span>백 · 나의 진형</span>
                  <span>★ 적 로얄: {state.royal_name}</span>
                </div>
                {state.phase === "combat" && (
                  <div
                    className={`action-hint ${targetCard ? "target-hint" : ""}`}
                    aria-live="polite"
                  >
                    {targetCard ? (
                      <>
                        <strong>{targetCard.definition.name}</strong> · 표시된
                        기물을 선택하세요.
                        <button
                          className="text-button"
                          onClick={() => setTargetCard(null)}
                        >
                          취소 Esc
                        </button>
                      </>
                    ) : selected !== null ? (
                      "표시된 칸을 클릭해 이동하세요."
                    ) : turn ? (
                      "카드를 사용하거나, 백 기물을 선택하세요."
                    ) : (
                      "상대가 다음 수를 고르고 있습니다…"
                    )}
                  </div>
                )}
              </section>
              <aside className="sidebar">
                <section className="turn-panel">
                  <div className="section-label">
                    TURN {state.round.toString().padStart(2, "0")}
                    <span>기본 마나 3</span>
                  </div>
                  <div className="mana">
                    <span className="mana-orb">{state.mana}</span>
                    <div>
                      <strong>사용 가능한 마나</strong>
                      <small>내 턴 시작 시 회복됩니다.</small>
                    </div>
                  </div>
                  <button
                    className="primary end-turn"
                    onClick={() => send({ type: "end_turn" })}
                    disabled={!turn || state.phase !== "combat"}
                  >
                    턴 종료 <span>→</span>
                  </button>
                </section>
                <section className="relic-panel">
                  <div className="section-label">
                    보유 유물 <span>{state.relics.length}</span>
                  </div>
                  {state.relics.map((r) => (
                    <div className="relic" key={r.name}>
                      <span>◇</span>
                      <div>
                        <strong>{r.name}</strong>
                        <p>{r.description}</p>
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
                  <small>카드 사용은 기물 이동 횟수를 소모하지 않습니다.</small>
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
                {state.hand.map((card) => (
                  <button
                    data-card-id={card.id}
                    className={`playing-card ${targetCard?.id === card.id ? "active-card" : ""}`}
                    key={card.id}
                    onClick={() => chooseCard(card)}
                    disabled={!!card.disabled_reason}
                    title={card.disabled_reason || card.definition.description}
                  >
                    <CardFace card={card.definition} />
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
          {state.phase !== "combat" && (
            <section className="outcome" ref={outcomeRef} aria-live="polite">
              {state.phase === "reward" && (
                <>
                  <p className="eyebrow">ROYAL CAPTURED</p>
                  <h2>전투 승리</h2>
                  <p>
                    남은 기물의 위치와 강화는 이어집니다. 덱에 추가할 카드 한
                    장을 선택하세요.
                  </p>
                  {state.node_kind === "elite" && state.node_reward && (
                    <p className="elite-reward">
                      추가 보상: {state.node_reward.name} ·{" "}
                      {state.node_reward.description}
                    </p>
                  )}
                  <div className="reward-cards">
                    {state.rewards.map((card, i) => (
                      <button
                        className="playing-card"
                        key={card.kind}
                        onClick={() =>
                          send({ type: "choose_reward", index: i })
                        }
                        aria-label={`보상 ${card.name}`}
                      >
                        <CardFace card={card} />
                        <span className="reward-select">덱에 추가 +</span>
                      </button>
                    ))}
                  </div>
                </>
              )}
              {state.phase === "treasure" && (
                <>
                  <p className="eyebrow">A GIFT ALONG THE WAY</p>
                  <h2>남겨진 보물</h2>
                  <div className="treasure-relic">◇</div>
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
              {state.phase === "rest" && (
                <>
                  <p className="eyebrow">A MOMENT OF REST</p>
                  <h2>작은 쉼터</h2>
                  <p>
                    기물을 한 번 강화하거나 부활시킬 수 있습니다. 템페스트 폰은
                    옆으로 이동·포획하고 뒤로 한 칸 이동할 수 있습니다.
                  </p>
                  <div className="rest-grid">
                    <div>
                      <h3>폰 강화</h3>
                      {state.rest_options
                        .filter((p) => p.can_upgrade)
                        .map((p) => (
                          <button
                            key={p.id}
                            onClick={() =>
                              send({ type: "rest_upgrade", id: p.id })
                            }
                          >
                            ♙ {p.square}의 폰 <span>→ 템페스트</span>
                          </button>
                        ))}
                      {!state.rest_options.some((p) => p.can_upgrade) && (
                        <p>강화할 일반 폰이 없습니다.</p>
                      )}
                    </div>
                    <div>
                      <h3>기물 부활</h3>
                      {state.rest_options
                        .filter((p) => p.dead)
                        .map((p) => (
                          <button
                            key={p.id}
                            disabled={!p.can_revive}
                            onClick={() =>
                              send({ type: "rest_revive", id: p.id })
                            }
                          >
                            {p.name} · {p.square}
                            <span>
                              {p.can_revive ? "부활" : "초기 칸 점유"}
                            </span>
                          </button>
                        ))}
                      {!state.rest_options.some((p) => p.dead) && (
                        <p>모든 아군이 생존했습니다.</p>
                      )}
                    </div>
                  </div>
                  <button
                    className="text-button"
                    onClick={() => send({ type: "rest_skip" })}
                  >
                    휴식을 마치고 맵으로 →
                  </button>
                </>
              )}
              {(state.phase === "victory" || state.phase === "defeat") && (
                <>
                  <span className="end-symbol">
                    {state.phase === "victory" ? "♔" : "♚"}
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
                      ? "마하라자를 직접 포획했습니다. 첫 번째 원정을 완수했습니다."
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
        CHESSLAYER{" "}
        <span>첫 번째 플레이테스트 · 임시 규칙이 적용되어 있습니다.</span>
      </footer>
    </>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);

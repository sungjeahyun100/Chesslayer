import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import init, { WebGame, catalog, compendium, simulator_catalog } from "../wasm/chesslayer_web";
import { GameScreen } from "../screens/GameScreen";
import { PieceImage } from "../PieceImage";
import { BuildInfo } from "../BuildInfo";
import { emptyEndlessRecord } from "../endless";
import { useEnemyTurn } from "../game/useEnemyTurn";
import { useGamePresentation } from "../game/useGamePresentation";
import type { CompendiumSquad } from "../Compendium";
import type { Character, Command, EnemyInspection, NodeKind, Snapshot } from "../types";
import { restoreSettings, saveSettings } from "../simulator/settings";
import { checkTransition, diff, formatValue, lifecycle, lifetimeFor, type Capture, type Change, type Check, type DebugState } from "./debugState";

type Starter = { id: number; kind: number; key: string; name: string };
type Squad = { id: string; deck: string[]; relics: string[]; pieces: Starter[] };
type Encounter = { index: number; name: string; floor: number; tier: "normal" | "elite" | "boss" };
type Content = { squads: Squad[]; encounters: Encounter[] };
type Config = { squad: string; seed: number; floor: number; gold: number; deck: string[]; relics: string[];
  upgrades: { id: number; target: number }[]; dead: number[]; cleared_stages: number; bosses_defeated: number; ammo: number | null;
  cards_burned: number; reinforcement_used: boolean; hunger_protection_used: boolean; next_battle_penalty: boolean; ambushed: boolean };
type Transition = { label: string; before: Capture; after: Capture; changes: Change[]; checks: Check[] };
type Trace = { order: number; cause: string; kind: string; actor: string; target: string; result: string; error: boolean; changes: Change[] };
const SETTINGS_KEY = "chesslayer.expedition-debugger.settings.v1";
const RUN_KEY = "chesslayer.expedition-debugger.run.v1";
const emptyContent: Content = { squads: [], encounters: [] };
const nodeKinds: NodeKind[] = ["combat", "elite", "rest", "treasure", "boss", "event", "shop"];
const kindNames: Record<NodeKind, string> = { combat: "일반 전투", elite: "정예 전투", rest: "휴식방", treasure: "보물", boss: "보스", event: "이벤트", shop: "상점" };
const tierFor: Partial<Record<NodeKind, Encounter["tier"]>> = { combat: "normal", elite: "elite", boss: "boss" };
const ruleNames: Record<string, string> = { preserved: "값 유지", equals: "값 일치", changed: "값 변경", relic: "유물 보유", dead: "기물 사망 유지", kind_stable: "영구 강화 단계 유지", temporary_cleared: "일시 강화 정리", log_once: "로그 정확히 1회" };

function download(name: string, value: unknown) {
  const url = URL.createObjectURL(new Blob([JSON.stringify(value, null, 2)], { type: "application/json" }));
  const anchor = document.createElement("a"); anchor.href = url; anchor.download = name; anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}
function safeSettings(content: Content, entries: CompendiumSquad[]) {
  return restoreSettings(content, entries, SETTINGS_KEY);
}

export function ExpeditionDebugger() {
  const engine = useRef<WebGame | null>(null);
  const config = useRef<Config | null>(null);
  const battleBefore = useRef<Capture | null>(null);
  const battleEntry = useRef<{ type: string; kind?: NodeKind; encounter?: number; command?: Command } | null>(null);
  const [ready, setReady] = useState(false);
  const [content, setContent] = useState<Content>(emptyContent);
  const [characters, setCharacters] = useState<Character[]>([]);
  const [entries, setEntries] = useState<CompendiumSquad[]>([]);
  const [squadId, setSquadId] = useState("");
  const [deck, setDeck] = useState<string[]>([]);
  const [relics, setRelics] = useState<string[]>([]);
  const [upgrades, setUpgrades] = useState<Record<number, number>>({});
  const [dead, setDead] = useState<number[]>([]);
  const [floor, setFloor] = useState(1);
  const [stage, setStage] = useState(0);
  const [gold, setGold] = useState(0);
  const [seed, setSeed] = useState(1);
  const [clearedStages, setClearedStages] = useState(0);
  const [bossesDefeated, setBossesDefeated] = useState(0);
  const [ammo, setAmmo] = useState(10);
  const [cardsBurned, setCardsBurned] = useState(0);
  const [reinforcementUsed, setReinforcementUsed] = useState(false);
  const [hungerProtectionUsed, setHungerProtectionUsed] = useState(false);
  const [nextBattlePenalty, setNextBattlePenalty] = useState(false);
  const [ambushed, setAmbushed] = useState(false);
  const [nodeKind, setNodeKind] = useState<NodeKind>("combat");
  const [startPhase, setStartPhase] = useState<"map" | "node" | "reward" | "defeat">("node");
  const [encounterId, setEncounterId] = useState(0);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [internal, setInternal] = useState<DebugState | null>(null);
  const [transitions, setTransitions] = useState<Transition[]>([]);
  const [selectedTransition, setSelectedTransition] = useState(0);
  const [trace, setTrace] = useState<Trace[]>([]);
  const [checks, setChecks] = useState<Check[]>([]);
  const [ruleKind, setRuleKind] = useState("preserved");
  const [rulePath, setRulePath] = useState("gold");
  const [ruleExpected, setRuleExpected] = useState("");
  const [showAll, setShowAll] = useState(false);
  const [repeatCount, setRepeatCount] = useState(3);
  const [repeatResult, setRepeatResult] = useState("");
  const [repeatFailures, setRepeatFailures] = useState<{ run: number; label: string; check: Check; before: Capture; after: Capture }[]>([]);
  const [error, setError] = useState("");
  const [screen, setScreen] = useState<"setup" | "run">("setup");
  const [revision, setRevision] = useState(0);
  const [cardMovement, setCardMovement] = useState<{ sequence: number; before: Snapshot; next: Snapshot } | null>(null);
  const sequence = useRef(0);
  const presentation = useGamePresentation(useCallback(() => {}, []));
  const { present } = presentation;

  useEffect(() => {
    let active = true;
    init().then(() => {
      if (!active) return;
      const data: Content = JSON.parse(simulator_catalog());
      const book: CompendiumSquad[] = JSON.parse(compendium());
      const restored = safeSettings(data, book);
      setContent(data); setCharacters(JSON.parse(catalog())); setEntries(book);
      if (restored) {
        setSquadId(restored.squad); setDeck(restored.deck); setRelics(restored.relics);
        setUpgrades(restored.upgrades); setEncounterId(restored.encounter);
      }
      try {
        const extra = JSON.parse(localStorage.getItem(SETTINGS_KEY + ".extra") ?? "{}");
        if (Number.isInteger(extra.floor)) setFloor(extra.floor);
        if (Number.isInteger(extra.stage)) setStage(extra.stage);
        if (Number.isInteger(extra.gold)) setGold(extra.gold);
        if (Number.isInteger(extra.seed)) setSeed(extra.seed);
        if (Number.isInteger(extra.clearedStages)) setClearedStages(extra.clearedStages);
        if (Number.isInteger(extra.bossesDefeated)) setBossesDefeated(extra.bossesDefeated);
        if (Number.isInteger(extra.ammo)) setAmmo(extra.ammo);
        if (Number.isInteger(extra.cardsBurned)) setCardsBurned(extra.cardsBurned);
        if (typeof extra.reinforcementUsed === "boolean") setReinforcementUsed(extra.reinforcementUsed);
        if (typeof extra.hungerProtectionUsed === "boolean") setHungerProtectionUsed(extra.hungerProtectionUsed);
        if (typeof extra.nextBattlePenalty === "boolean") setNextBattlePenalty(extra.nextBattlePenalty);
        if (typeof extra.ambushed === "boolean") setAmbushed(extra.ambushed);
        if (nodeKinds.includes(extra.nodeKind)) setNodeKind(extra.nodeKind);
        if (["map", "node", "reward", "defeat"].includes(extra.startPhase)) setStartPhase(extra.startPhase);
        if (Array.isArray(extra.dead)) setDead(extra.dead);
      } catch { /* Damaged local settings do not prevent a new run. */ }
      setReady(true);
    }).catch((cause) => setError(`엔진 로딩 실패: ${String(cause)}`));
    return () => { active = false; };
  }, []);
  useEffect(() => {
    if (!ready) return;
    saveSettings({ squad: squadId, encounter: encounterId, deck, relics, upgrades }, SETTINGS_KEY);
    try { localStorage.setItem(SETTINGS_KEY + ".extra", JSON.stringify({ floor, stage, gold, seed, clearedStages, bossesDefeated, ammo, cardsBurned, reinforcementUsed, hungerProtectionUsed, nextBattlePenalty, ambushed, nodeKind, startPhase, dead })); }
    catch { /* Settings remain usable in memory. */ }
  }, [ready, squadId, encounterId, deck, relics, upgrades, floor, stage, gold, seed, clearedStages, bossesDefeated, ammo, cardsBurned, reinforcementUsed, hungerProtectionUsed, nextBattlePenalty, ambushed, nodeKind, startPhase, dead]);

  const squad = content.squads.find((item) => item.id === squadId);
  const entry = entries.find((item) => item.id === squadId);
  const encounters = content.encounters.filter((item) => item.tier === tierFor[nodeKind]);
  const chosenEncounter = encounters.find((item) => item.index === encounterId) ?? encounters[0];
  const selected = transitions[selectedTransition] ?? transitions.at(-1);
  const availableCards = entry?.cards.filter((card) => card.available) ?? [];
  const availableRelics = entry?.relics ?? [];
  const capture = (game: WebGame): Capture => ({ state: JSON.parse(game.debug_inspect()), snapshot: JSON.parse(game.snapshot()), saved: game.save() });

  function install(game: WebGame, source: "start" | "resume" = "resume") {
    engine.current?.free(); engine.current = game;
    try { config.current = JSON.parse(game.save()).expedition_debugger ?? config.current; } catch { /* Valid game can still run. */ }
    const current = capture(game);
    battleBefore.current = null; battleEntry.current = null;
    if (current.snapshot.phase === "combat") {
      try {
        const record = JSON.parse(current.saved);
        const index = record.debug_ops?.findLastIndex((op: { type: string; command?: Command }) =>
          op.type === "enter_node" || (op.type === "command" && ["select_node", "event_choice"].includes(op.command?.type ?? ""))) ?? -1;
        if (index >= 0) {
          const beforeGame = WebGame.restore(JSON.stringify({ ...record, debug_ops: record.debug_ops.slice(0, index) }));
          battleBefore.current = capture(beforeGame);
          battleEntry.current = record.debug_ops[index];
          beforeGame.free();
        }
      } catch { /* A restored battle remains playable without a restart marker. */ }
    }
    setSnapshot(current.snapshot); setInternal(current.state); setCardMovement(null);
    setRevision((n) => n + 1); present(current.snapshot, null, source);
    setScreen("run"); setError("");
    setRepeatResult(""); setRepeatFailures([]);
    try { localStorage.setItem(RUN_KEY, current.saved); } catch { /* Manual export still works. */ }
  }
  function apply(label: string, action: (game: WebGame) => string, command?: Command, directDrag = false): boolean {
    const game = engine.current;
    if (!game) return false;
    const before = capture(game);
    try {
      const next: Snapshot = JSON.parse(action(game));
      const after: Capture = { state: JSON.parse(game.debug_inspect()), snapshot: next, saved: game.save() };
      const changes = diff(before.state, after.state);
      const automatic = lifecycle(before, after);
      const results = checks.map((check) => checkTransition(check, before, after, changes));
      if (before.snapshot.phase !== "combat" && after.snapshot.phase === "combat") {
        battleBefore.current = before;
        battleEntry.current = JSON.parse(after.saved).debug_ops?.at(-1) ?? null;
      }
      setSnapshot(next); setInternal(after.state);
      setCardMovement({ sequence: ++sequence.current, before: before.snapshot, next });
      present(next, before.snapshot, command ?? "resume", { directDrag });
      setTransitions((current) => {
        const updated = [...current, { label, before, after, changes, checks: [...automatic, ...results] }];
        setSelectedTransition(updated.length - 1);
        return updated;
      });
      setChecks(results);
      setRepeatResult(""); setRepeatFailures([]);
      type EngineEvent = { kind: string; actor: string | null; target: string | null; result: string; state: DebugState };
      const previousEvents = before.state.trace_events as EngineEvent[] | undefined;
      const currentEvents = after.state.trace_events as EngineEvent[] | undefined;
      const addedEvents = currentEvents?.slice(previousEvents?.length ?? 0) ?? [];
      let previousEventState = previousEvents?.at(-1)?.state;
      const eventRows = addedEvents.map((event, index) => {
        const eventChanges = previousEventState ? diff(previousEventState, event.state).filter((item) => item.kind !== "same") : [];
        previousEventState = event.state;
        return { order: index + 1, cause: label, kind: event.kind, actor: event.actor ?? "—",
          target: event.target ?? "—", result: event.result, error: false, changes: eventChanges };
      });
      setTrace((current) => [...current, ...eventRows.map((row, index) => ({ ...row, order: current.length + index + 1 })),
        { order: current.length + eventRows.length + 1, cause: label, kind: command?.type ?? "state_diff",
          actor: "—", target: "상태", result: "전이 직후 스냅샷", error: false,
          changes: changes.filter((c) => c.kind !== "same") }]);
      try { localStorage.setItem(RUN_KEY, after.saved); } catch { /* Manual export still works. */ }
      setError(""); return true;
    } catch (cause) {
      const message = String(cause);
      setError(message);
      setTrace((current) => [...current, { order: current.length + 1, cause: label, kind: command?.type ?? "debug",
        actor: "—", target: "—", result: message, error: true, changes: [] }]);
      return false;
    }
  }
  const send = useCallback((command: Command, directDrag = false) => apply(command.type, (game) => game.dispatch(JSON.stringify(command)), command, directDrag), [checks, present]);
  useEnemyTurn(screen === "run" && !presentation.presentationBusy, snapshot, engine, send, setError);

  function start() {
    if (!squad) return;
    if (startPhase === "reward" && !tierFor[nodeKind]) { setError("보상 화면은 전투 노드에서만 시작할 수 있습니다."); return; }
    const nextConfig: Config = { squad: squadId, seed: seed >>> 0, floor, gold, deck: [...deck], relics: [...relics],
      upgrades: squad.pieces.filter((piece) => upgrades[piece.id] !== undefined && upgrades[piece.id] !== piece.kind)
        .map((piece) => ({ id: piece.id, target: upgrades[piece.id] })), dead, cleared_stages: clearedStages,
      bosses_defeated: bossesDefeated, ammo, cards_burned: cardsBurned,
      reinforcement_used: reinforcementUsed, hunger_protection_used: hungerProtectionUsed,
      next_battle_penalty: nextBattlePenalty, ambushed };
    try {
      config.current = nextConfig;
      const game = WebGame.start_expedition_debugger(JSON.stringify(nextConfig));
      install(game, "start"); setTransitions([]); setTrace([]); setChecks([]);
      if (stage > 0) apply("강제 단계 이동", (item) => item.debug_jump(floor, stage));
      if (startPhase === "node" || startPhase === "reward" || startPhase === "defeat") {
        const entered = enterForcedNode(game);
        if (entered && startPhase === "reward") apply("설정: 전투 승리 후 보상", (item) => item.debug_finish_battle(true));
        if (entered && startPhase === "defeat") {
          if (tierFor[nodeKind]) apply("설정: 전투 패배", (item) => item.debug_finish_battle(false));
          else apply("설정: 원정 종료", (item) => item.debug_end_run());
        }
      }
    } catch (cause) { setError(String(cause)); }
  }
  function enterForcedNode(game = engine.current) {
    if (!game) return false;
    return apply(`강제 ${kindNames[nodeKind]} 진입`, (item) => item.debug_enter_node(nodeKind, chosenEncounter?.index ?? null));
  }
  function nextNode() {
    const id = snapshot?.map.nodes.find((node) => node.available)?.id;
    if (id !== undefined) send({ type: "select_node", id });
    else setError("정상 경로에 진입 가능한 노드가 없습니다. 현재 보상 또는 비전투 노드를 먼저 마치세요.");
  }
  function restore(saved: string): boolean {
    try { install(WebGame.restore(saved)); setError(""); return true; }
    catch (cause) { setError(`복원 실패: ${String(cause)}`); return false; }
  }
  function restoreBefore() { if (selected) restore(selected.before.saved); }
  function restartBattle() {
    const before = battleBefore.current, op = battleEntry.current;
    if (!before || !op) { setError("현재 전투의 시작 전 상태가 없습니다."); return; }
    if (!restore(before.saved)) return;
    if (op.type === "enter_node" && op.kind) apply("현재 전투 재시작", (game) => game.debug_enter_node(op.kind!, op.encounter ?? null));
    else if (op.type === "command" && op.command) send(op.command);
    else setError("전투 진입 기록을 재생할 수 없습니다.");
  }
  function saveCurrent() { if (engine.current) download("chesslayer-expedition-snapshot.json", {
    format: "chesslayer-expedition-debugger-v1", config: config.current, saved: engine.current.save(), transitions, trace,
    setup: { squadId, deck, relics, upgrades, dead, floor, stage, gold, seed, clearedStages, bossesDefeated, ammo,
      cardsBurned, reinforcementUsed, hungerProtectionUsed, nextBattlePenalty, ambushed, nodeKind, startPhase, encounterId },
  }); }
  async function importSnapshot(file: File) {
    try {
      const parsed = JSON.parse(await file.text());
      if (parsed.format !== "chesslayer-expedition-debugger-v1" || typeof parsed.saved !== "string") throw new Error("원정 디버거 파일이 아닙니다.");
      if (!restore(parsed.saved)) return;
      if (Array.isArray(parsed.transitions)) setTransitions(parsed.transitions);
      if (Array.isArray(parsed.trace)) setTrace(parsed.trace);
      if (parsed.config) config.current = parsed.config;
      const setup = parsed.setup;
      if (setup && typeof setup === "object") {
        if (typeof setup.squadId === "string" && content.squads.some((item) => item.id === setup.squadId)) setSquadId(setup.squadId);
        if (Array.isArray(setup.deck)) setDeck(setup.deck);
        if (Array.isArray(setup.relics)) setRelics(setup.relics);
        if (setup.upgrades && typeof setup.upgrades === "object") setUpgrades(setup.upgrades);
        if (Array.isArray(setup.dead)) setDead(setup.dead);
        if (Number.isInteger(setup.floor)) setFloor(setup.floor);
        if (Number.isInteger(setup.stage)) setStage(setup.stage);
        if (Number.isInteger(setup.gold)) setGold(setup.gold);
        if (Number.isInteger(setup.seed)) setSeed(setup.seed);
        if (Number.isInteger(setup.clearedStages)) setClearedStages(setup.clearedStages);
        if (Number.isInteger(setup.bossesDefeated)) setBossesDefeated(setup.bossesDefeated);
        if (Number.isInteger(setup.ammo)) setAmmo(setup.ammo);
        if (Number.isInteger(setup.cardsBurned)) setCardsBurned(setup.cardsBurned);
        if (typeof setup.reinforcementUsed === "boolean") setReinforcementUsed(setup.reinforcementUsed);
        if (typeof setup.hungerProtectionUsed === "boolean") setHungerProtectionUsed(setup.hungerProtectionUsed);
        if (typeof setup.nextBattlePenalty === "boolean") setNextBattlePenalty(setup.nextBattlePenalty);
        if (typeof setup.ambushed === "boolean") setAmbushed(setup.ambushed);
        if (nodeKinds.includes(setup.nodeKind)) setNodeKind(setup.nodeKind);
        if (["map", "node", "reward", "defeat"].includes(setup.startPhase)) setStartPhase(setup.startPhase);
        if (Number.isInteger(setup.encounterId)) setEncounterId(setup.encounterId);
      }
    } catch (cause) { setError(`불러오기 실패: ${String(cause)}`); }
  }
  function repeat() {
    if (!engine.current) return;
    const saved = engine.current.save();
    const expected = JSON.stringify(JSON.parse(engine.current.debug_inspect()));
    let mismatches = 0;
    const failures: { run: number; label: string; check: Check; before: Capture; after: Capture }[] = [];
    try {
      for (let n = 0; n < Math.min(20, Math.max(1, repeatCount)); n++) {
        const replay = WebGame.restore(saved);
        if (JSON.stringify(JSON.parse(replay.debug_inspect())) !== expected) mismatches++;
        replay.free();
        for (const transition of transitions) {
          const beforeGame = WebGame.restore(transition.before.saved);
          const afterGame = WebGame.restore(transition.after.saved);
          const before = capture(beforeGame), after = capture(afterGame);
          const changes = diff(before.state, after.state);
          beforeGame.free(); afterGame.free();
          for (const rule of transition.checks) {
            const result = checkTransition(rule, before, after, changes);
            if (result.result === "failure" && failures.length < 20)
              failures.push({ run: n + 1, label: transition.label, check: result, before, after });
          }
        }
      }
      setRepeatFailures(failures);
      setRepeatResult(mismatches || failures.length ? `${repeatCount}회 실행: 재생 불일치 ${mismatches}회, 검증 실패 ${failures.length}건` : `${repeatCount}회 동일 시드 재생 상태 일치`);
    } catch (cause) { setRepeatResult(`반복 실행 실패: ${String(cause)}`); }
  }
  function addCheck() {
    setChecks((current) => [...current, { id: Date.now(), kind: ruleKind, path: rulePath.trim(), expected: ruleExpected.trim(), result: "unverified", detail: "다음 상태 전이에서 검증" }]);
  }
  const filteredChanges = showAll ? selected?.changes : selected?.changes.filter((item) => item.kind !== "same");
  const rawInspector = useMemo(() => internal ? JSON.stringify(internal, null, 2) : "", [internal]);

  return <div className="ed-app">
    <header className="ed-header"><div><strong>CHESSLAYER / LOCAL TOOLS</strong><h1>원정 진행 디버거</h1><BuildInfo /></div>
      <div className="ed-head-status">{snapshot ? `${snapshot.floor}층 · ${snapshot.phase} · ${snapshot.node_kind ?? "지도"} · ${snapshot.gold}골드` : "설정 중"}
        <a href="./battle-simulator.html">전투 시뮬레이터</a></div></header>
    {error && <p className="ed-error" role="alert">{error}<button onClick={() => setError("")}>닫기</button></p>}
    {!ready && <p>엔진과 콘텐츠를 불러오는 중입니다.</p>}
    {ready && <>
      <div className="ed-toolbar">
        <button onClick={() => setScreen("setup")}>설정</button><button onClick={start}>원정 초기화</button>
        <button disabled={!snapshot || snapshot.phase !== "combat"} onClick={() => apply("강제 승리", (game) => game.debug_finish_battle(true))}>강제 승리</button>
        <button disabled={!snapshot || snapshot.phase !== "combat"} onClick={() => apply("강제 패배", (game) => game.debug_finish_battle(false))}>강제 패배</button>
        <button disabled={!battleBefore.current} onClick={restartBattle}>현재 전투 재시작</button>
        <button disabled={!snapshot || snapshot.phase !== "map"} onClick={nextNode}>다음 노드 (정상)</button>
        <button disabled={!snapshot || snapshot.phase !== "map"} onClick={() => {
          const match = content.encounters.find((item) => item.tier === "normal" && item.floor === snapshot?.floor);
          apply("강제 다음 전투", (game) => game.debug_enter_node("combat", match?.index ?? null));
        }}>다음 전투 (강제)</button>
        <button disabled={!snapshot || snapshot.phase !== "map"} onClick={() => {
          const match = content.encounters.find((item) => item.tier === "boss" && item.floor === snapshot?.floor);
          apply("강제 보스 진입", (game) => game.debug_enter_node("boss", match?.index ?? null));
        }}>보스 진입 (강제)</button>
        <button disabled={!snapshot || snapshot.phase !== "reward"} onClick={() => send({ type: "continue_reward" })}>보상 건너뛰기 / 계속</button>
        <button disabled={!snapshot || snapshot.phase === "combat"} onClick={() => apply("강제 다음 층", (game) => game.debug_jump(Math.min(4, snapshot!.floor + 1), 0))}>다음 층 (강제)</button>
        <button disabled={!snapshot || snapshot.phase === "combat"} onClick={() => apply("원정 종료", (game) => game.debug_end_run())}>원정 종료</button>
      </div>
      <div className="ed-columns">
        <aside className="ed-side"><details open><summary>원정 설정</summary>
          <label>플레이어 부대<select aria-label="플레이어 부대" value={squadId} onChange={(event) => {
            const next = content.squads.find((item) => item.id === event.target.value);
            if (next) { setSquadId(next.id); setDeck([...next.deck]); setRelics([...next.relics]); setUpgrades({}); setDead([]); }
          }}>{characters.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></label>
          <div className="ed-fields"><label>층<input type="number" min="1" max="4" value={floor} onChange={(e) => setFloor(Number(e.target.value))} /></label>
            <label>진행 단계<input type="number" min="0" max="15" value={stage} onChange={(e) => setStage(Number(e.target.value))} /></label>
            <label>골드<input type="number" min="0" value={gold} onChange={(e) => setGold(Number(e.target.value))} /></label>
            <label>랜덤 시드<input type="number" min="0" value={seed} onChange={(e) => setSeed(Number(e.target.value))} /></label>
            <label>누적 클리어<input type="number" min="0" value={clearedStages} onChange={(e) => setClearedStages(Number(e.target.value))} /></label>
            <label>보스 격파<input type="number" min="0" value={bossesDefeated} onChange={(e) => setBossesDefeated(Number(e.target.value))} /></label>
            <label>탄약<input type="number" min="0" value={ammo} onChange={(e) => setAmmo(Number(e.target.value))} /></label>
            <label>카드 소각 횟수<input type="number" min="0" value={cardsBurned} onChange={(e) => setCardsBurned(Number(e.target.value))} /></label></div>
          <div className="ed-flags"><label><input type="checkbox" checked={reinforcementUsed} onChange={(e) => setReinforcementUsed(e.target.checked)} /> 충원 사용</label>
            <label><input type="checkbox" checked={hungerProtectionUsed} onChange={(e) => setHungerProtectionUsed(e.target.checked)} /> 허기 보호 사용</label>
            <label><input type="checkbox" checked={nextBattlePenalty} onChange={(e) => setNextBattlePenalty(e.target.checked)} /> 다음 전투 페널티</label>
            <label><input type="checkbox" checked={ambushed} onChange={(e) => setAmbushed(e.target.checked)} /> 기습 발생</label></div>
          <label>현재 노드 종류<select value={nodeKind} onChange={(e) => setNodeKind(e.target.value as NodeKind)}>{nodeKinds.map((kind) => <option key={kind} value={kind}>{kindNames[kind]}</option>)}</select></label>
          <label>시작 진행 상태<select value={startPhase} onChange={(e) => setStartPhase(e.target.value as typeof startPhase)}>
            <option value="map">지도 · 노드 선택 전</option><option value="node">선택 노드 진입</option>
            <option value="reward">전투 보상</option><option value="defeat">원정 종료</option></select></label>
          {tierFor[nodeKind] && <label>전투 상대<select value={chosenEncounter?.index ?? ""} onChange={(e) => setEncounterId(Number(e.target.value))}>{encounters.map((item) => <option key={item.index} value={item.index}>{item.name} · {item.floor}층</option>)}</select></label>}
          <button className="primary" onClick={start}>설정으로 원정 시작</button>
        </details>
        <details><summary>기물 · 강화 · 생존</summary><div className="ed-list">{squad?.pieces.map((piece) => {
          const options = entry?.pieces.filter((item) => item.kind === piece.kind || item.upgrades.some((edge) => edge.from === piece.kind || edge.to === piece.kind)) ?? [];
          const actual = entry?.pieces.find((item) => item.kind === (upgrades[piece.id] ?? piece.kind));
          return <div className="ed-piece" key={piece.id}><PieceImage pieceKey={actual?.key ?? piece.key} name={actual?.name ?? piece.name} /><span>#{piece.id} {piece.name}</span>
            <select aria-label={`#${piece.id} 강화 단계`} value={upgrades[piece.id] ?? piece.kind} onChange={(e) => setUpgrades((current) => ({ ...current, [piece.id]: Number(e.target.value) }))}>
              {options.map((item) => <option key={item.kind} value={item.kind}>{item.name}</option>)}</select>
            <label><input type="checkbox" checked={!dead.includes(piece.id)} onChange={(e) => setDead((current) => e.target.checked ? current.filter((id) => id !== piece.id) : [...current, piece.id])} /> 생존</label></div>;
        })}</div></details>
        <details><summary>덱 · {deck.length}장</summary><div className="ed-list">{deck.map((kind, index) => <div key={`${kind}-${index}`}>{availableCards.find((item) => item.definition.kind === kind)?.definition.name ?? kind}
          <button onClick={() => setDeck((current) => current.filter((_, i) => i !== index))}>×</button></div>)}</div>
          <select aria-label="카드 추가" defaultValue="" onChange={(e) => { if (e.target.value) setDeck((current) => [...current, e.target.value]); e.target.value = ""; }}><option value="">카드 추가…</option>{availableCards.map((item) => <option key={item.definition.kind} value={item.definition.kind}>{item.definition.name}</option>)}</select></details>
        <details><summary>유물 · {relics.length}개</summary><div className="ed-list">{relics.map((id) => <div key={id}>{availableRelics.find((item) => item.id === id)?.name ?? id}<button onClick={() => setRelics((current) => current.filter((item) => item !== id))}>×</button></div>)}</div>
          <select aria-label="유물 추가" defaultValue="" onChange={(e) => { if (e.target.value) setRelics((current) => [...current, e.target.value]); e.target.value = ""; }}><option value="">유물 추가…</option>{availableRelics.filter((item) => !relics.includes(item.id)).map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}</select></details>
        {screen === "run" && snapshot && <details open><summary>진행 제어</summary>
          <div className="ed-fields"><label>강제 이동 층<input type="number" min="1" max="4" value={floor} onChange={(e) => setFloor(Number(e.target.value))} /></label>
            <label>진행 단계<input type="number" min="0" max="15" value={stage} onChange={(e) => setStage(Number(e.target.value))} /></label></div>
          <button disabled={snapshot.phase === "combat"} onClick={() => apply("강제 위치 이동", (game) => game.debug_jump(floor, stage))}>해당 단계로 강제 이동</button>
          <button disabled={snapshot.phase !== "map"} onClick={() => enterForcedNode()}>선택 노드 강제 진입</button>
          <div className="ed-quick">{(["rest", "shop", "event", "treasure"] as NodeKind[]).map((kind) => <button key={kind} disabled={snapshot.phase !== "map"} onClick={() => apply(`강제 ${kindNames[kind]} 진입`, (game) => game.debug_enter_node(kind, null))}>{kindNames[kind]}</button>)}</div>
        </details>}
        <details><summary>스냅샷 · 재현</summary><div className="ed-quick">
          <button disabled={!engine.current} onClick={saveCurrent}>JSON 내보내기</button>
          <label className="ed-upload">JSON 불러오기<input type="file" accept="application/json,.json" onChange={(e) => { const file = e.target.files?.[0]; if (file) void importSnapshot(file); e.target.value = ""; }} /></label>
          <button disabled={!selected} onClick={restoreBefore}>선택 전이 직전 복원</button>
          <button onClick={() => { const saved = localStorage.getItem(RUN_KEY); if (saved) restore(saved); else setError("저장된 디버거 상태가 없습니다."); }}>자동 저장 불러오기</button>
          <button disabled={!engine.current} onClick={() => download("chesslayer-expedition-trace.json", trace)}>이벤트 기록 내보내기</button></div>
          <label>동일 기록 반복<input type="number" min="1" max="20" value={repeatCount} onChange={(e) => setRepeatCount(Number(e.target.value))} /></label>
          <button disabled={!engine.current} onClick={repeat}>반복 실행</button><p role="status">{repeatResult}</p>
          {repeatFailures.length > 0 && <div className="ed-list">{repeatFailures.map((failure, index) => <div key={index}>
            {failure.run}회 · {failure.label} · {failure.check.detail}
            <button onClick={() => download(`chesslayer-failure-${index + 1}.json`, failure)}>실패 스냅샷</button>
          </div>)}</div>}
        </details></aside>
        <section className="ed-game" aria-label="실제 게임 화면">
          {screen === "setup" && <div className="ed-placeholder"><h2>원정 설정</h2><p>좌측에서 부대와 시드, 노드, 자원을 설정한 뒤 원정을 시작하세요.</p><button className="primary" onClick={start}>원정 시작</button></div>}
          {screen === "run" && snapshot && presentation.displayState && <GameScreen key={revision} debuggerMode snapshot={snapshot} displayState={presentation.displayState} cardMovement={cardMovement}
            send={send} inspectEnemy={(cell): EnemyInspection | null => {
              try { return engine.current ? JSON.parse(engine.current.inspect_enemy(cell)) : null; }
              catch (cause) { setError(String(cause)); return null; }
            }} setError={setError} onRestart={() => setScreen("setup")} developerMode upgradeHistory={[]}
            rewardUpgradeHistory={[]} presentationBusy={presentation.presentationBusy} lastEnemyMove={presentation.lastEnemyMove}
            boardRevision={presentation.boardRevision} animation={presentation.animation} statusEffects={presentation.statusEffects}
            endlessBest={emptyEndlessRecord} endlessRunBest={emptyEndlessRecord} />}
        </section>
        <aside className="ed-inspector"><details open><summary>실시간 상태 검사기</summary>
          {snapshot && internal ? <><div className="ed-metrics"><span>층 {snapshot.floor}</span><span>단계 {snapshot.map.nodes.find((n) => n.current)?.row ?? 0}</span><span>{snapshot.node_kind ?? "지도"}</span><span>{snapshot.gold} G</span><span>마나 {snapshot.mana}</span><span>행동 {snapshot.actions_left}</span></div>
            <details open><summary>원정 상태</summary><pre>{JSON.stringify({ floor: internal.floor, phase: internal.phase, current_node: internal.current_node, path: internal.path, completed: internal.completed, last_cleared_node: internal.last_cleared_node, gold: internal.gold, relics: internal.relics, cleared_stages: internal.cleared_stages, bosses_defeated: internal.bosses_defeated, event: internal.event, reward_choice: internal.reward_choice, upgrade_claimed: internal.upgrade_claimed, shop_offers: internal.shop_offers, shop_bought: internal.shop_bought, ammo: internal.ammo, progress: internal.progress }, null, 2)}</pre></details>
            <details><summary>기물 영구 상태 · 생존 {((internal.roster as { alive: boolean }[]) ?? []).filter((p) => p.alive).length}</summary><pre>{JSON.stringify(internal.roster, null, 2)}</pre></details>
            <details><summary>전투 배치 · 상태이상</summary><pre>{JSON.stringify(snapshot.pieces.map((p) => ({ id: p.id, kind: p.kind, owner: p.owner, cell: p.cell, temporary_upgrade: p.temporary_upgrade, statuses: p.statuses, limiter: p.limiter, stats: p.stats })), null, 2)}</pre></details>
            <details><summary>상태이상 수치 · 효과 사용 플래그</summary><pre>{JSON.stringify({ statuses: (internal.battle as { mechanics?: { statuses?: unknown; used?: unknown; turn_used?: unknown } })?.mechanics?.statuses,
              used: (internal.battle as { mechanics?: { used?: unknown } })?.mechanics?.used,
              turn_used: (internal.battle as { mechanics?: { turn_used?: unknown } })?.mechanics?.turn_used }, null, 2)}</pre></details>
            <details><summary>카드 · 전투 자원</summary><pre>{JSON.stringify({ deck: internal.deck, cards: internal.cards, round: internal.round, actions_left: internal.actions_left, battle_result: internal.battle_result }, null, 2)}</pre></details>
            <details><summary>전체 엔진 내부 상태</summary><pre>{rawInspector}</pre></details>
          </> : <p>원정을 시작하면 엔진 상태가 표시됩니다.</p>}</details></aside>
      </div>
      <div className="ed-bottom"><section><details open><summary>이벤트 추적기 · {trace.length}</summary>
        <div className="ed-scroll"><table><thead><tr><th>#</th><th>원인</th><th>종류</th><th>주체</th><th>대상</th><th>처리 결과</th><th>변경</th></tr></thead><tbody>{trace.map((item) => <tr key={item.order} className={item.error ? "ed-fail" : ""}><td>{item.order}</td><td>{item.cause}</td><td>{item.kind}</td><td>{item.actor}</td><td>{item.target}</td><td>{item.result}</td><td>{item.changes.length ? <details><summary>{item.changes.length}</summary><pre>{JSON.stringify(item.changes, null, 2)}</pre></details> : "0"}</td></tr>)}</tbody></table></div>
        <p className="ed-note">순서는 엔진 처리 지점과 로그가 기록된 순서입니다. 변경 항목은 인접 기록 시점의 엔진 상태 비교이며, 다음 기록 전까지 일어난 효과가 함께 포함될 수 있습니다. 구조화되지 않은 주체와 대상은 —입니다.</p></details></section>
        <section><details open><summary>상태 전이 비교기 · {transitions.length}</summary>
          <div className="ed-compare-head"><select aria-label="상태 전이 선택" value={selectedTransition} onChange={(e) => setSelectedTransition(Number(e.target.value))}>{transitions.map((item, index) => <option key={index} value={index}>{index + 1}. {item.label}</option>)}</select>
            <label><input type="checkbox" checked={showAll} onChange={(e) => setShowAll(e.target.checked)} /> 유지된 값 포함</label></div>
          {selected && <><div className="ed-scroll"><table><thead><tr><th>상태 항목</th><th>이전</th><th>이후</th><th>변경</th><th>수명</th></tr></thead><tbody>{filteredChanges?.map((item) => <tr key={item.path} data-kind={item.kind}><td>{item.path}</td><td>{formatValue(item.before)}</td><td>{formatValue(item.after)}</td><td>{item.kind}</td><td>{lifetimeFor(item.path)}</td></tr>)}</tbody></table></div>
            <details><summary>전이 시점 JSON</summary><pre>{JSON.stringify({ before: selected.before, after: selected.after }, null, 2)}</pre></details></>}
        </details></section>
        <section><details open><summary>상태 수명 · 검증 조건</summary>
          <div className="ed-rule"><select aria-label="검증 종류" value={ruleKind} onChange={(e) => setRuleKind(e.target.value)}>{Object.entries(ruleNames).map(([id, name]) => <option key={id} value={id}>{name}</option>)}</select>
            <input aria-label="상태 경로" placeholder="gold / cards.mana" value={rulePath} onChange={(e) => setRulePath(e.target.value)} />
            <input aria-label="예상 값" placeholder="예상 값 · 유물 ID · 기물 ID" value={ruleExpected} onChange={(e) => setRuleExpected(e.target.value)} />
            <button onClick={addCheck}>조건 추가</button></div>
          <div className="ed-scroll"><table><thead><tr><th>조건</th><th>결과</th><th>근거</th></tr></thead><tbody>{[...(selected?.checks ?? []), ...checks.filter((item) => item.result === "unverified")].map((item, index) => <tr key={`${item.id}-${index}`}><td>{ruleNames[item.kind]} {item.path || item.expected}</td><td>{item.result === "success" ? "성공" : item.result === "failure" ? "실패" : "미검증"}</td><td>{item.detail}</td></tr>)}</tbody></table></div>
          <p className="ed-note">수명 열은 상태의 소유 범위입니다. 자동 판정은 골드·덱·유물 유지와 전투 종료 정리처럼 구현에서 확인된 규칙에만 적용합니다. 조건이 없는 항목은 미검증입니다.</p>
        </details></section></div>
    </>}
  </div>;
}

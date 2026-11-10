import { useCallback, useEffect, useRef, useState } from "react";
import init, { WebGame, catalog, compendium } from "../wasm/chesslayer_web";
import type { Character, Command, EnemyInspection, RunMode, Snapshot } from "../types";
import type { CompendiumSquad } from "../Compendium";
import { developerProgress } from "../progress";
import { readSavedRun, useRunStorage } from "./useRunStorage";
import { useGamePresentation } from "./useGamePresentation";
import { useEnemyTurn } from "./useEnemyTurn";
import { selfDestructKeys } from "./combatPlan";

export function useGameSession(active: boolean, onEnterGame: () => void) {
  const engine = useRef<WebGame | null>(null);
  const developerRun = useRef(false);
  const [ready, setReady] = useState(false);
  const [choices, setChoices] = useState<Character[]>([]);
  const [entries, setEntries] = useState<CompendiumSquad[]>([]);
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [runId, setRunId] = useState(0);
  const [error, setError] = useState("");
  const storage = useRunStorage();
  const { load, persist } = storage;
  const presentation = useGamePresentation(onEnterGame);
  const { present } = presentation;

  useEffect(() => {
    let mounted = true;
    init().then(() => {
      if (!mounted) return;
      setChoices(JSON.parse(catalog()));
      setEntries(JSON.parse(compendium()));
      setReady(true);
      load();
    }).catch((cause) => { if (mounted) setError(`게임 엔진을 불러오지 못했습니다. ${String(cause)}`); });
    return () => { mounted = false; };
  }, [load]);

  // A new run frees the previous engine. Do not free live state during Vite HMR;
  // the browser releases this WASM instance when the document closes.
  const send = useCallback((command: Command): boolean => {
    if (!engine.current) return false;
    try {
      const before: Snapshot = JSON.parse(engine.current.snapshot());
      const context: { enemyTarget?: number; blast?: number[] } = {};
      if (command.type === "enemy_step_selected" && command.from !== null && command.index !== null) {
        context.enemyTarget = before.boss
          ? before.boss.plan?.targets[0]?.cell ?? before.boss.plan?.to
          : engine.current.enemy_action_target(command.from, command.index);
        if (context.enemyTarget === command.from && before.pieces.some((piece) => piece.cell === command.from && selfDestructKeys.has(piece.key))) {
          context.blast = JSON.parse(engine.current.blast_cells(command.from));
        }
      }
      if (command.type === "special_action" && command.from === command.to && before.pieces.some((piece) => piece.cell === command.from && selfDestructKeys.has(piece.key))) {
        context.blast = JSON.parse(engine.current.blast_cells(command.from));
      }
      const next: Snapshot = JSON.parse(engine.current.dispatch(JSON.stringify(command)));
      setSnapshot(next);
      present(next, before, command, context);
      persist(engine.current.save(), next, developerRun.current);
      setError("");
      return true;
    } catch (cause) { setError(String(cause)); return false; }
  }, [persist, present]);
  useEnemyTurn(active && !presentation.presentationBusy, snapshot, engine, send, setError);

  function install(game: WebGame, isDeveloperRun: boolean, source: "start" | "resume") {
    game.set_developer_mode(storage.developerMode);
    engine.current?.free();
    engine.current = game;
    developerRun.current = isDeveloperRun;
    const next: Snapshot = JSON.parse(game.snapshot());
    setSnapshot(next);
    setRunId((id) => id + 1);
    present(next, null, source);
    persist(game.save(), next, isDeveloperRun);
    setError("");
  }
  function start(character: string, floor: number, mode: RunMode = "campaign") {
    try {
      const progress = storage.developerMode ? developerProgress(storage.progress, choices.map((choice) => choice.id)) : storage.progress;
      const seed = crypto.getRandomValues(new Uint32Array(1))[0];
      const game = WebGame.start_run_with_mode(character, seed, mode === "endless" ? 1 : floor, JSON.stringify(progress), mode);
      if (mode === "endless") storage.beginEndlessRun();
      install(game, storage.developerMode, "start");
    } catch (cause) { setError(String(cause)); }
  }
  function resume(mode: RunMode = "campaign") {
    try {
      const { saved, developerRun, previousBest } = readSavedRun(mode);
      if (saved) {
        const game = WebGame.restore(saved);
        if (mode === "endless") storage.beginEndlessRun(previousBest);
        install(game, developerRun, "resume");
      }
    } catch (cause) { setError(String(cause)); }
  }
  function inspectEnemy(cell: number): EnemyInspection | null {
    try {
      if (!engine.current) return null;
      const inspection: EnemyInspection = JSON.parse(engine.current.inspect_enemy(cell));
      setError("");
      return inspection;
    } catch (cause) { setError(String(cause)); return null; }
  }
  return { ready, choices, entries, snapshot, runId, error, setError, send, start, resume, inspectEnemy, storage, presentation };
}

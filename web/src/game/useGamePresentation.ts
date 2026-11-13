import { useCallback, useEffect, useRef, useState } from "react";
import type { Command, Snapshot } from "../types";
import { combatPresentationSteps, type CombatPlan } from "./combatPlan";
import { statusChanges, type StatusChange } from "./statusVisuals";

export type FloorTransition = { next: Snapshot; stage: "out" | "in" };
export type BoardMove = { from: number; to: number };

export function useGamePresentation(onEnterGame: () => void) {
  const [displayState, setDisplayState] = useState<Snapshot | null>(null);
  const [floorTransition, setFloorTransition] = useState<FloorTransition | null>(null);
  const [lastEnemyMove, setLastEnemyMove] = useState<BoardMove | null>(null);
  const [boardRevision, setBoardRevision] = useState(0);
  const [animation, setAnimation] = useState<CombatPlan | null>(null);
  const [statusEffects, setStatusEffects] = useState<StatusChange[]>([]);
  const timers = useRef<number[]>([]);
  const statusTimers = useRef<number[]>([]);
  const animationSequence = useRef(0);
  const statusSequence = useRef(0);
  const clearAnimation = useCallback(() => {
    timers.current.forEach(window.clearTimeout);
    timers.current = [];
    statusTimers.current.forEach(window.clearTimeout);
    statusTimers.current = [];
    setAnimation(null);
    setStatusEffects([]);
  }, []);
  useEffect(() => clearAnimation, [clearAnimation]);

  // The engine result is committed before presentation starts. Keep the previous
  // snapshot on screen until contact, then reveal the committed next snapshot.
  const present = useCallback((next: Snapshot, before: Snapshot | null, source: "start" | "resume" | Command,
    context: { enemyTarget?: number; blast?: number[] } = {}) => {
    clearAnimation();
    const newRun = typeof source === "string";
    if (newRun || (before?.phase !== "combat" && next.phase === "combat")) {
      setLastEnemyMove(null);
      setBoardRevision((revision) => revision + 1);
    } else if (source.type === "enemy_step_selected" && next.phase === "combat") {
      const moved = before?.pieces.find((piece) => piece.owner === 1 && next.pieces.some((after) => after.id === piece.id && after.cell !== piece.cell));
      const destination = next.pieces.find((piece) => piece.id === moved?.id)?.cell;
      if (moved && destination !== undefined) setLastEnemyMove({ from: moved.cell, to: destination });
    }
    const changesFloor = typeof source !== "string" && ["choose_reward", "continue_reward", "advance_floor"].includes(source.type)
      && (before?.floor !== next.floor || before?.final_battle !== next.final_battle) && next.phase === "map";
    const steps = before && typeof source !== "string" && !window.matchMedia("(prefers-reduced-motion: reduce)").matches
      ? combatPresentationSteps(before, next, source, context) : [];
    const showStatusChanges = (oldState: Snapshot, newState: Snapshot, newBatch: boolean) => {
      const changes = statusChanges(oldState, newState, newBatch);
      if (!changes.length || window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
      const effects = changes.map((change) => ({ ...change, id: ++statusSequence.current }));
      setStatusEffects((current) => [...current, ...effects]);
      statusTimers.current.push(window.setTimeout(() => {
        const ids = new Set(effects.map((effect) => effect.id));
        setStatusEffects((current) => current.filter((effect) => !ids.has(effect.id)));
      }, 480));
    };
    if (steps.length) {
      const playStep = (index: number) => {
        const step = steps[index];
        const plan = { ...step.plan, sequence: ++animationSequence.current };
        setDisplayState(step.before);
        setAnimation(plan);
        timers.current.push(window.setTimeout(() => {
          // A finishing blow still needs its impact/fade on the board before
          // the victory or defeat screen replaces that board.
          setDisplayState(step.before.phase === "combat" && step.after.phase !== "combat"
            ? { ...step.after, phase: "combat" } : step.after);
          showStatusChanges(step.before, step.after, index === 0);
          setAnimation((current) => current?.sequence === plan.sequence ? { ...current, impactDone: true } : current);
        }, plan.impact));
        timers.current.push(window.setTimeout(() => {
          if (index + 1 < steps.length) playStep(index + 1);
          else {
            setDisplayState(next);
            setAnimation(null);
            timers.current = [];
          }
        }, plan.duration));
      };
      playStep(0);
      return;
    }
    if ((source === "start" || changesFloor) && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setFloorTransition({ next, stage: "out" });
    } else {
      setDisplayState(next);
      if (before) showStatusChanges(before, next, true);
      setFloorTransition(null);
      if (newRun) onEnterGame();
    }
  }, [onEnterGame, clearAnimation]);

  useEffect(() => {
    if (!floorTransition) return;
    const timer = window.setTimeout(() => {
      if (floorTransition.stage === "out") {
        setDisplayState(floorTransition.next);
        onEnterGame();
        setFloorTransition({ ...floorTransition, stage: "in" });
      } else setFloorTransition(null);
    }, floorTransition.stage === "out" ? 600 : 1100);
    return () => window.clearTimeout(timer);
  }, [floorTransition, onEnterGame]);

  const skipFloorTransition = useCallback(() => {
    if (!floorTransition) return;
    setDisplayState(floorTransition.next);
    onEnterGame();
    setFloorTransition(null);
  }, [floorTransition, onEnterGame]);
  useEffect(() => {
    if (!floorTransition) return;
    const escape = (event: KeyboardEvent) => { if (event.key === "Escape") skipFloorTransition(); };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, [floorTransition, skipFloorTransition]);

  return { displayState, floorTransition, lastEnemyMove, boardRevision, animation, statusEffects,
    presentationBusy: !!floorTransition || !!animation, present, skipFloorTransition };
}

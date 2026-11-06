import { useCallback, useEffect, useState } from "react";
import type { Command, Snapshot } from "../types";

export type FloorTransition = { next: Snapshot; stage: "out" | "in" };
export type BoardMove = { from: number; to: number };

export function useGamePresentation(onEnterGame: () => void) {
  const [displayState, setDisplayState] = useState<Snapshot | null>(null);
  const [floorTransition, setFloorTransition] = useState<FloorTransition | null>(null);
  const [lastEnemyMove, setLastEnemyMove] = useState<BoardMove | null>(null);
  const [boardRevision, setBoardRevision] = useState(0);

  // This is the handoff from committed engine state to presentation. Currently
  // only the existing floor transition delays display; no board state is cloned.
  const present = useCallback((next: Snapshot, before: Snapshot | null, source: "start" | "resume" | Command) => {
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
    if ((source === "start" || changesFloor) && !window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      setFloorTransition({ next, stage: "out" });
    } else {
      setDisplayState(next);
      setFloorTransition(null);
      if (newRun) onEnterGame();
    }
  }, [onEnterGame]);

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

  return { displayState, floorTransition, lastEnemyMove, boardRevision, presentationBusy: !!floorTransition, present, skipFloorTransition };
}

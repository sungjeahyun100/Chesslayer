import { useEffect, useRef, type RefObject } from "react";
import type { WebGame } from "../wasm/chesslayer_web";
import type { Command, Snapshot } from "../types";

export function useEnemyTurn(active: boolean, snapshot: Snapshot | null, engine: RefObject<WebGame | null>, send: (command: Command) => boolean, setError: (message: string) => void) {
  const enemyWorker = useRef<Worker | null>(null);
  useEffect(() => {
    if (!active || snapshot?.phase !== "combat" || snapshot.turn !== 1 || snapshot.effect_choice)
      return;
    const timer = setTimeout(() => {
      const saved = engine.current?.save();
      if (!saved) return;
      const worker = new Worker(new URL("../enemyWorker.ts", import.meta.url), { type: "module" });
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
  }, [active, snapshot, engine, send, setError]);
}

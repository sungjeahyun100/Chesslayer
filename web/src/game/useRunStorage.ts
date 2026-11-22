import { useCallback, useRef, useState } from "react";
import type { Command, RunMode, Snapshot } from "../types";
import { PROGRESS_KEY, DEVELOPER_MODE_KEY, DEVELOPER_RUN_KEY, initialProgress, readProgress, mergeProgress } from "../progress";
import { ENDLESS_SAVE_KEY, emptyEndlessRecord, parseEndlessRecord, readEndlessBest, mergeEndlessRecord, updateEndlessBest, type EndlessRecord } from "../endless";

// Keep the original slot and the engine's serialized command history unchanged.
const SAVE_KEY = "chesslayer.run.v1";
function playableSave(value: string | null): boolean {
  if (!value) return false;
  try {
    const save: { character?: unknown; version?: unknown } = JSON.parse(value);
    return save.character === "avenger" && (save.version === 23 || save.version === 24);
  } catch { return false; }
}

export function readSavedRun(mode: RunMode = "campaign") {
  if (mode === "endless") {
    const value = localStorage.getItem(ENDLESS_SAVE_KEY);
    const parsed = value ? JSON.parse(value) : null;
    return { saved: parsed?.run as string | undefined, developerRun: parsed?.developerRun === true, previousBest: parseEndlessRecord(parsed?.previousBest) };
  }
  return { saved: localStorage.getItem(SAVE_KEY), developerRun: localStorage.getItem(DEVELOPER_RUN_KEY) === "true" };
}

export function useRunStorage() {
  const [progress, setProgress] = useState(initialProgress);
  const [developerMode, setDeveloperMode] = useState(false);
  const [hasSave, setHasSave] = useState(false);
  const [hasEndlessSave, setHasEndlessSave] = useState(false);
  const [endlessBest, setEndlessBest] = useState(emptyEndlessRecord);
  const bestRef = useRef(emptyEndlessRecord);
  const [endlessRunBest, setEndlessRunBest] = useState(emptyEndlessRecord);
  const runBestRef = useRef(emptyEndlessRecord);
  const [saveNotice, setSaveNotice] = useState("");
  const [upgradeHistory, setUpgradeHistory] = useState<Extract<Command, { type: "rest_upgrade" }>[]>([]);
  const [rewardUpgradeHistory, setRewardUpgradeHistory] = useState<{ id: number; target: number }[]>([]);

  const load = useCallback(() => {
    bestRef.current = readEndlessBest();
    setEndlessBest(bestRef.current);
    try {
      const campaign = localStorage.getItem(SAVE_KEY);
      const endless = localStorage.getItem(ENDLESS_SAVE_KEY);
      setHasSave(playableSave(campaign));
      let endlessRun: string | null = null;
      try { endlessRun = endless ? JSON.parse(endless)?.run ?? null : null; } catch { /* Keep unreadable data untouched. */ }
      setHasEndlessSave(playableSave(endlessRun));
      if ((campaign && !playableSave(campaign)) || (endless && !playableSave(endlessRun))) {
        setSaveNotice("이전 부대 또는 규칙의 원정은 이어할 수 없습니다. 저장 데이터는 보존되며 새 복수귀 원정을 시작할 수 있습니다.");
      }
      setProgress(readProgress());
      setDeveloperMode(localStorage.getItem(DEVELOPER_MODE_KEY) === "true");
    } catch {
      setSaveNotice("브라우저 저장소를 사용할 수 없어 이번 원정은 저장되지 않습니다.");
    }
  }, []);

  const persist = useCallback((saved: string, snapshot: Snapshot, developerRun: boolean) => {
    const endless = snapshot.mode === "endless";
    if (!endless && !developerRun) setProgress((previous) => mergeProgress(previous, snapshot.progress ?? initialProgress));
    if (endless && !developerRun && (snapshot.map.current !== null || snapshot.cleared_stages > 0)) {
      bestRef.current = updateEndlessBest(mergeEndlessRecord(bestRef.current, {
        clearedStages: snapshot.cleared_stages, highestFloor: snapshot.floor,
        reachedStage: (snapshot.floor - 1) * 15 + (snapshot.map.nodes.find(n => n.current)?.row ?? 0) + 1, runSeed: snapshot.seed,
        squad: snapshot.character, achievedAt: new Date().toISOString(), bossesDefeated: snapshot.bosses_defeated ?? 0,
        deckSize: snapshot.deck_count, relicCount: snapshot.relics.length,
      }));
      setEndlessBest(bestRef.current);
    }
    // History is for presentation only; promotions must not light an upgrade edge.
    const commands: Command[] = JSON.parse(saved).commands;
    setUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" }> => c.type === "rest_upgrade"));
    setRewardUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" | "claim_upgrade" }> => c.type === "rest_upgrade" || c.type === "claim_upgrade").map(({ id, target }) => ({ id, target })));
    try {
      if (endless) {
        localStorage.setItem(ENDLESS_SAVE_KEY, JSON.stringify({ run: saved, developerRun, previousBest: runBestRef.current }));
        setHasEndlessSave(true);
      } else {
        localStorage.setItem(SAVE_KEY, saved);
        localStorage.setItem(DEVELOPER_RUN_KEY, String(developerRun));
        setHasSave(true);
      }
      if (!endless && !developerRun) {
        const nextProgress = mergeProgress(readProgress(), snapshot.progress ?? initialProgress);
        localStorage.setItem(PROGRESS_KEY, JSON.stringify(nextProgress));
        setProgress(nextProgress);
      }
      setSaveNotice("");
    } catch {
      setSaveNotice("진행 상태를 저장하지 못했습니다. 이 탭을 닫으면 최근 진행을 잃을 수 있습니다.");
    }
  }, []);

  function beginEndlessRun(previousBest?: EndlessRecord) {
    bestRef.current = mergeEndlessRecord(bestRef.current, readEndlessBest());
    setEndlessBest(bestRef.current);
    runBestRef.current = previousBest ?? bestRef.current;
    setEndlessRunBest(runBestRef.current);
  }

  function toggleDeveloperMode() {
    const next = !developerMode;
    setDeveloperMode(next);
    try {
      localStorage.setItem(DEVELOPER_MODE_KEY, String(next));
      setSaveNotice("");
    } catch {
      setSaveNotice("개발자 모드 상태를 저장하지 못했습니다. 새로고침 후에는 다시 설정해 주세요.");
    }
  }
  return { progress, developerMode, hasSave, hasEndlessSave, endlessBest, endlessRunBest, beginEndlessRun, saveNotice, upgradeHistory, rewardUpgradeHistory, load, persist, toggleDeveloperMode };
}

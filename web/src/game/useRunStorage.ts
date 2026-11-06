import { useCallback, useState } from "react";
import type { Command, Snapshot } from "../types";
import { PROGRESS_KEY, DEVELOPER_MODE_KEY, DEVELOPER_RUN_KEY, initialProgress, readProgress, mergeProgress } from "../progress";

// Keep the original slot and the engine's serialized command history unchanged.
const SAVE_KEY = "chesslayer.run.v1";

export function readSavedRun() {
  return { saved: localStorage.getItem(SAVE_KEY), developerRun: localStorage.getItem(DEVELOPER_RUN_KEY) === "true" };
}

export function useRunStorage() {
  const [progress, setProgress] = useState(initialProgress);
  const [developerMode, setDeveloperMode] = useState(false);
  const [hasSave, setHasSave] = useState(false);
  const [saveNotice, setSaveNotice] = useState("");
  const [upgradeHistory, setUpgradeHistory] = useState<Extract<Command, { type: "rest_upgrade" }>[]>([]);
  const [rewardUpgradeHistory, setRewardUpgradeHistory] = useState<{ id: number; target: number }[]>([]);

  const load = useCallback(() => {
    try {
      setHasSave(!!localStorage.getItem(SAVE_KEY));
      setProgress(readProgress());
      setDeveloperMode(localStorage.getItem(DEVELOPER_MODE_KEY) === "true");
    } catch {
      setSaveNotice("브라우저 저장소를 사용할 수 없어 이번 원정은 저장되지 않습니다.");
    }
  }, []);

  const persist = useCallback((saved: string, snapshot: Snapshot, developerRun: boolean) => {
    if (!developerRun) setProgress((previous) => mergeProgress(previous, snapshot.progress ?? initialProgress));
    // History is for presentation only; promotions must not light an upgrade edge.
    const commands: Command[] = JSON.parse(saved).commands;
    setUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" }> => c.type === "rest_upgrade"));
    setRewardUpgradeHistory(commands.filter((c): c is Extract<Command, { type: "rest_upgrade" | "claim_upgrade" }> => c.type === "rest_upgrade" || c.type === "claim_upgrade").map(({ id, target }) => ({ id, target })));
    try {
      localStorage.setItem(SAVE_KEY, saved);
      localStorage.setItem(DEVELOPER_RUN_KEY, String(developerRun));
      if (!developerRun) {
        const nextProgress = mergeProgress(readProgress(), snapshot.progress ?? initialProgress);
        localStorage.setItem(PROGRESS_KEY, JSON.stringify(nextProgress));
        setProgress(nextProgress);
      }
      setHasSave(true);
      setSaveNotice("");
    } catch {
      setSaveNotice("진행 상태를 저장하지 못했습니다. 이 탭을 닫으면 최근 진행을 잃을 수 있습니다.");
    }
  }, []);

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
  return { progress, developerMode, hasSave, saveNotice, upgradeHistory, rewardUpgradeHistory, load, persist, toggleDeveloperMode };
}

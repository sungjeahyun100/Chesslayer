import type { Progress } from "./types";

export const PROGRESS_KEY = "chesslayer.progress.v1";
export const DEVELOPER_MODE_KEY = "chesslayer.developer_mode.v1";
export const DEVELOPER_RUN_KEY = "chesslayer.run.developer.v1";
export const initialProgress: Progress = {
  highest_floor: 1,
  unlocked_squads: ["guardian"],
  final_unlocked: false,
  final_defeated: false,
};

export function mergeProgress(previous: Progress, next: Progress): Progress {
  return {
    highest_floor: Math.min(4, Math.max(1, previous.highest_floor || 1, next.highest_floor || 1)),
    unlocked_squads: [...new Set(["guardian", ...previous.unlocked_squads, ...next.unlocked_squads])],
    final_unlocked: previous.final_unlocked || next.final_unlocked,
    final_defeated: previous.final_defeated || next.final_defeated,
  };
}

export function developerProgress(progress: Progress, squadIds: string[]): Progress {
  return mergeProgress(progress, {
    highest_floor: 4,
    unlocked_squads: squadIds,
    final_unlocked: true,
    final_defeated: false,
  });
}

export function readProgress(): Progress {
  const value = localStorage.getItem(PROGRESS_KEY);
  if (!value) return initialProgress;
  try {
    const parsed: Partial<Progress> = JSON.parse(value);
    if (!Array.isArray(parsed.unlocked_squads)) return initialProgress;
    return mergeProgress(initialProgress, {
      highest_floor: typeof parsed.highest_floor === "number" ? parsed.highest_floor : 1,
      unlocked_squads: parsed.unlocked_squads.filter((value): value is string => typeof value === "string"),
      final_unlocked: parsed.final_unlocked === true,
      final_defeated: parsed.final_defeated === true,
    });
  } catch { return initialProgress; }
}

export const floorNames = ["최종 결전", "갈림길의 숲", "끝없는 만찬", "포화의 전선", "침묵의 작전"];
export const bossNames = ["부족의 독재자", "선봉대", "이단 부대", "포병대대", "특수부대"];
export function floorLabel(floor: number, final = false) {
  return final || floor === 0 ? "최종 결전" : `${floor}층`;
}

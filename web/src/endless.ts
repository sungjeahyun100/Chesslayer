export const ENDLESS_BEST_KEY = "chesslayer.endless.best.v1";
export const ENDLESS_SAVE_KEY = "chesslayer.endless.run.v1";

export interface EndlessRecord {
  clearedStages: number;
  highestFloor: number;
  reachedStage?: number;
  runSeed?: number;
  squad?: string;
  achievedAt?: string;
  bossesDefeated?: number;
  deckSize?: number;
  relicCount?: number;
}
export const emptyEndlessRecord: EndlessRecord = { clearedStages: 0, highestFloor: 0 };

export function parseEndlessRecord(value: unknown): EndlessRecord {
  const record = value as Partial<EndlessRecord> | null;
  const count = (n: unknown) => typeof n === "number" && Number.isSafeInteger(n) && n >= 0 ? n : 0;
  const result: EndlessRecord = { clearedStages: count(record?.clearedStages), highestFloor: count(record?.highestFloor) };
  if (typeof record?.squad === "string") result.squad = record.squad;
  if (typeof record?.achievedAt === "string") result.achievedAt = record.achievedAt;
  for (const key of ["reachedStage", "runSeed", "bossesDefeated", "deckSize", "relicCount"] as const) {
    if (typeof record?.[key] === "number") result[key] = count(record[key]);
  }
  return result;
}

export function readEndlessBest(): EndlessRecord {
  try { return parseEndlessRecord(JSON.parse(localStorage.getItem(ENDLESS_BEST_KEY) ?? "null")); }
  catch { return { ...emptyEndlessRecord }; }
}

export function mergeEndlessRecord(previous: EndlessRecord, next: EndlessRecord): EndlessRecord {
  const previousStage = previous.reachedStage ?? previous.clearedStages;
  const nextStage = next.reachedStage ?? next.clearedStages;
  const better = nextStage > previousStage || (nextStage === previousStage &&
    (next.clearedStages > previous.clearedStages || next.highestFloor > previous.highestFloor));
  if (better) return { ...next };
  if (nextStage === previousStage && next.clearedStages === previous.clearedStages && next.runSeed !== undefined && next.runSeed === previous.runSeed) {
    return { ...next, achievedAt: previous.achievedAt ?? next.achievedAt };
  }
  return { ...previous };
}

export function updateEndlessBest(record: EndlessRecord): EndlessRecord {
  const previous = readEndlessBest();
  const next = mergeEndlessRecord(previous, record);
  if (JSON.stringify(next) !== JSON.stringify(previous)) {
    try { localStorage.setItem(ENDLESS_BEST_KEY, JSON.stringify(next)); }
    catch { /* The in-memory record and the expedition remain usable. */ }
  }
  return next;
}

export function formatEndlessRecord(record: EndlessRecord) {
  return `${record.clearedStages}단계 클리어 · ${record.highestFloor}층 도달${record.reachedStage ? ` · 최고 ${record.reachedStage}단계` : ""}`;
}

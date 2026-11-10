export const ENDLESS_BEST_KEY = "chesslayer.endless.best.v1";
export const ENDLESS_SAVE_KEY = "chesslayer.endless.run.v1";

export interface EndlessRecord {
  clearedStages: number;
  highestFloor: number;
}
export const emptyEndlessRecord: EndlessRecord = { clearedStages: 0, highestFloor: 0 };

export function parseEndlessRecord(value: unknown): EndlessRecord {
  const record = value as Partial<EndlessRecord> | null;
  const count = (n: unknown) => typeof n === "number" && Number.isSafeInteger(n) && n >= 0 ? n : 0;
  return { clearedStages: count(record?.clearedStages), highestFloor: count(record?.highestFloor) };
}

export function readEndlessBest(): EndlessRecord {
  try { return parseEndlessRecord(JSON.parse(localStorage.getItem(ENDLESS_BEST_KEY) ?? "null")); }
  catch { return { ...emptyEndlessRecord }; }
}

export function mergeEndlessRecord(previous: EndlessRecord, next: EndlessRecord): EndlessRecord {
  return { clearedStages: Math.max(previous.clearedStages, next.clearedStages), highestFloor: Math.max(previous.highestFloor, next.highestFloor) };
}

export function updateEndlessBest(record: EndlessRecord): EndlessRecord {
  const previous = readEndlessBest();
  const next = mergeEndlessRecord(previous, record);
  if (next.clearedStages > previous.clearedStages || next.highestFloor > previous.highestFloor) {
    try { localStorage.setItem(ENDLESS_BEST_KEY, JSON.stringify(next)); }
    catch { /* The in-memory record and the expedition remain usable. */ }
  }
  return next;
}

export function formatEndlessRecord(record: EndlessRecord) {
  return `${record.clearedStages}단계 클리어 · ${record.highestFloor}층 도달`;
}

import type { Piece, Snapshot } from "../types";

export type StatusKind = "ingredient" | "blessing" | "hunger" | "madness" | "guilt" | "agility" | "weakness"
  | "incapacitated" | "dining" | "death_threshold" | "bell" | "temporary" | "aimed" | "propellant"
  | "entrenched_pending" | "entrenched_ready" | "suppressed" | "reckless" | "ward" | "pest" | "corpse_bags" | "spirit_bomb";
export type StatusVisual = { kind: StatusKind; name: string; icon: string; description: string; count: number; label: string };
export type StatusChange = { id: number; pieceId: number; cell: number; kind: StatusKind; event: "apply" | "stack" | "remove" | "trigger" };

export const hungerStages = [
  { label: "공복", name: "허기-공복", description: "이동과 포획에 제한이 없습니다. 허기가 한 단계 심해지면 굶주림이 됩니다." },
  { label: "굶주림", name: "허기-굶주림", description: "먹을 수 있는 시체가 있으면 다른 기물을 포획할 수 없습니다. 빈칸 이동은 가능합니다. 먹을 수 있는 시체가 없다면 행동 제한이 없습니다." },
  { label: "기아", name: "허기-기아", description: "먹을 수 있는 시체가 있으면 그 시체를 먹는 행동만 할 수 있습니다. 이동과 다른 포획은 불가능합니다. 먹을 수 있는 시체가 없다면 행동 제한이 없습니다." },
] as const;

export const statusDefinitions: Record<StatusKind, { name: string; icon: string; description: string }> = {
  ingredient: { name: "식재료", icon: "◇", description: "로얄이 아닌 이 기물이 포획되면 그 자리에 시체가 남습니다. 일부 폭발성 사격에는 적용되지 않습니다." },
  blessing: { name: "황충의 축복", icon: "✹", description: "시체를 포획하면 허기를 해소하고 다음 자기 진영 턴에 민첩 3을 얻습니다. 광기와 죄책감은 쌓이지 않습니다." },
  hunger: { name: hungerStages[0].name, icon: "◔", description: hungerStages[0].description },
  madness: { name: "광기", icon: "☷", description: "3중첩이 되면 광기를 모두 잃고 상대 진영으로 전향하며 황충의 축복을 얻습니다." },
  guilt: { name: "죄책감", icon: "◈", description: "2중첩이 되면 죄책감을 모두 잃고 전투불능 1을 얻습니다." },
  agility: { name: "민첩", icon: "➜", description: "수치만큼 해당 진영의 추가 행동을 할 수 있습니다." },
  weakness: { name: "약화", icon: "⌄", description: "해당 진영의 다음 턴에 중첩만큼 기물 단계가 내려갑니다." },
  incapacitated: { name: "전투불능", icon: "×", description: "중첩된 수만큼 자기 턴을 쉬며, 현재 층을 떠날 때까지 전투 사이에도 유지됩니다." },
  pest: { name: "페스트", icon: "☣", description: "턴 종료 시 1 감소하고 주변 8칸에 각각 50% 확률로 페스트 3을 전염시킵니다. 0이 되면 사망합니다." },
  corpse_bags: { name: "시체자루", icon: "▣", description: "시체 포획 시 획득하며 수레 특수능력에 1개를 소모합니다." },
  spirit_bomb: { name: "원기옥", icon: "◉", description: "이번 전투 동안 턴 종료 시 남은 행마권을 다음 턴으로 이월합니다." },
  dining: { name: "식사의 예법", icon: "♜", description: "인접한 시체를 먹을 수 있습니다." },
  death_threshold: { name: "죽음의 문턱", icon: "☠", description: "다음 전투 시작 시 살아 있다면 즉시 사망합니다." },
  bell: { name: "다음 수 만찬의 종소리", icon: "♬", description: "만찬의 종소리 대상으로 지정되었습니다. 다음 종소리가 울릴 때 허기가 한 단계 심해집니다." },
  temporary: { name: "임시 기물", icon: "◌", description: "이번 전투에서만 함께하며, 정규 아군의 손실로 계산되지 않습니다." },
  aimed: { name: "다음 사격 명중률", icon: "⌖", description: "다음 사격의 명중률 보정입니다." },
  propellant: { name: "다음 관통", icon: "⇥", description: "다음 관통 공격의 보정입니다." },
  entrenched_pending: { name: "진지 유지 중", icon: "▣", description: "진지를 유지하고 있습니다." },
  entrenched_ready: { name: "방어 진지 강화", icon: "⬟", description: "방어 진지가 강화되었습니다." },
  suppressed: { name: "이동 제압", icon: "⊘", description: "이동 제압이 적용 중입니다." },
  reckless: { name: "무모한 돌격", icon: "⚔", description: "무모한 돌격이 적용 중입니다." },
  ward: { name: "가호", icon: "✝", description: "다음 직접 포획 또는 명중한 사격을 한 번 막습니다." },
};

const orderedKinds = Object.keys(statusDefinitions) as StatusKind[];
const prefixes: [StatusKind, string][] = [
  ["ingredient", "식재료"], ["blessing", "황충의 축복"], ["madness", "광기"],
  ["guilt", "죄책감"], ["agility", "민첩"], ["weakness", "약화"],
  ["incapacitated", "전투불능"], ["pest", "페스트"], ["corpse_bags", "시체자루"], ["spirit_bomb", "원기옥"], ["dining", "식사의 예법"],
  ["death_threshold", "죽음의 문턱"], ["bell", "다음 수 만찬의 종소리"],
  ["temporary", "임시 기물"], ["aimed", "다음 사격 명중률"],
  ["propellant", "다음 관통"], ["entrenched_pending", "진지 유지 중"],
  ["entrenched_ready", "방어 진지 강화"], ["suppressed", "이동 제압"],
  ["reckless", "무모한 돌격"],
];

export function statusVisuals(piece: Pick<Piece, "statuses" | "ward">): StatusVisual[] {
  const result = new Map<StatusKind, StatusVisual>();
  for (const label of piece.statuses ?? []) {
    const kind = hungerStages.some((stage) => stage.label === label) ? "hunger" : prefixes.find(([, prefix]) => label === prefix || label.startsWith(`${prefix} `))?.[0];
    if (!kind) continue;
    const definition = statusDefinitions[kind];
    const count = kind === "hunger" ? hungerStages.findIndex((stage) => stage.label === label) + 1
      : Number(label.match(/^(?:광기|죄책감|민첩|약화|전투불능|페스트|시체자루) (\d+)$/)?.[1]
        ?? (kind === "aimed" || kind === "propellant" ? label.match(/\+(\d+)%p$/)?.[1] : undefined) ?? 1);
    const stage = kind === "hunger" ? hungerStages[count - 1] : undefined;
    result.set(kind, { kind, ...definition, ...stage, count, label });
  }
  if (piece.ward) result.set("ward", { kind: "ward", ...statusDefinitions.ward, count: 1, label: "가호" });
  return orderedKinds.flatMap((kind) => result.get(kind) ?? []);
}

// Compare the engine's visible state once per accepted command. Position changes
// alone do not produce events; status effects are tied to piece IDs.
export function statusChanges(before: Snapshot, after: Snapshot, newBatch = false): Omit<StatusChange, "id">[] {
  if (before.phase !== "combat" && after.phase !== "combat") return [];
  const oldPieces = new Map(before.pieces.map((piece) => [piece.id, piece]));
  const changes: Omit<StatusChange, "id">[] = [];
  for (const event of after.combat_events ?? []) {
    if (event.kind !== "status_triggered") continue;
    const seen = (before.combat_events ?? []).some((previous) => previous.kind === "status_triggered"
      && previous.piece_id === event.piece_id && previous.cell === event.cell && previous.status === event.status);
    if (newBatch || !seen) changes.push({ pieceId: event.piece_id, cell: event.cell, kind: event.status, event: "trigger" });
  }
  for (const piece of after.pieces) {
    const old = oldPieces.get(piece.id);
    const previous = new Map((old ? statusVisuals(old) : []).map((status) => [status.kind, status]));
    const current = new Map(statusVisuals(piece).map((status) => [status.kind, status]));
    for (const status of current.values()) {
      const prior = previous.get(status.kind);
      if (!prior) changes.push({ pieceId: piece.id, cell: piece.cell, kind: status.kind, event: "apply" });
      else if (status.count > prior.count) changes.push({ pieceId: piece.id, cell: piece.cell, kind: status.kind, event: "stack" });
    }
    if (old) for (const status of previous.values()) if (!current.has(status.kind)) {
      if (status.kind === "ward" && changes.some((change) => change.pieceId === piece.id && change.kind === "ward" && change.event === "trigger")) continue;
      changes.push({ pieceId: piece.id, cell: piece.cell, kind: status.kind, event: "remove" });
    }
  }
  return changes;
}

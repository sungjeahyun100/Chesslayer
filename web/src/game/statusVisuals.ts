import type { Piece, Snapshot } from "../types";

export type StatusKind = "ingredient" | "blessing" | "hunger" | "madness" | "guilt" | "agility" | "weakness"
  | "incapacitated" | "dining" | "death_threshold" | "bell" | "temporary" | "aimed" | "propellant"
  | "entrenched_pending" | "entrenched_ready" | "suppressed" | "reckless" | "ward";
export type StatusVisual = { kind: StatusKind; name: string; icon: string; description: string; count: number; label: string };
export type StatusChange = { id: number; pieceId: number; cell: number; kind: StatusKind; event: "apply" | "stack" | "remove" | "trigger" };

const definitions: Record<StatusKind, { name: string; icon: string; description: string }> = {
  ingredient: { name: "식재료", icon: "◇", description: "식사 효과의 대상이며, 포획 시 시체를 남길 수 있습니다." },
  blessing: { name: "황충의 축복", icon: "✹", description: "시체를 먹으면 다음 자기 진영 턴에 민첩 3을 얻습니다." },
  hunger: { name: "허기", icon: "◔", description: "공복 → 굶주림 → 기아 순으로 심해지며, 굶주림·기아에서는 시체를 우선합니다." },
  madness: { name: "광기", icon: "☷", description: "3중첩 시 상대 진영으로 전향하고 황충의 축복을 얻습니다." },
  guilt: { name: "죄책감", icon: "◈", description: "2중첩 시 영구 전투불능이 됩니다." },
  agility: { name: "민첩", icon: "➜", description: "남은 민첩 수치입니다. 해당 진영의 턴 종료 시 해제됩니다." },
  weakness: { name: "약화", icon: "⌄", description: "해당 진영의 다음 턴에 중첩만큼 기물 단계가 내려갑니다." },
  incapacitated: { name: "영구 전투불능", icon: "×", description: "영구 전투불능 · 휴식에서 고해성사로 해제할 수 있습니다." },
  dining: { name: "식사의 예법", icon: "♜", description: "인접한 시체를 먹을 수 있습니다." },
  death_threshold: { name: "죽음의 문턱", icon: "☠", description: "다음 전투 시작 시 살아 있다면 즉시 사망합니다." },
  bell: { name: "다음 수 만찬의 종소리", icon: "♬", description: "만찬의 종소리 대상으로 지정되었습니다." },
  temporary: { name: "임시 기물", icon: "◌", description: "이번 전투의 임시 기물입니다." },
  aimed: { name: "다음 사격 명중률", icon: "⌖", description: "다음 사격의 명중률 보정입니다." },
  propellant: { name: "다음 관통", icon: "⇥", description: "다음 관통 공격의 보정입니다." },
  entrenched_pending: { name: "진지 유지 중", icon: "▣", description: "진지를 유지하고 있습니다." },
  entrenched_ready: { name: "방어 진지 강화", icon: "⬟", description: "방어 진지가 강화되었습니다." },
  suppressed: { name: "이동 제압", icon: "⊘", description: "이동 제압이 적용 중입니다." },
  reckless: { name: "무모한 돌격", icon: "⚔", description: "무모한 돌격이 적용 중입니다." },
  ward: { name: "가호", icon: "✝", description: "다음 직접 포획 또는 명중한 사격을 한 번 막습니다." },
};

const orderedKinds = Object.keys(definitions) as StatusKind[];
const prefixes: [StatusKind, string][] = [
  ["ingredient", "식재료"], ["blessing", "황충의 축복"], ["madness", "광기"],
  ["guilt", "죄책감"], ["agility", "민첩"], ["weakness", "약화"],
  ["incapacitated", "영구 전투불능"], ["dining", "식사의 예법"],
  ["death_threshold", "죽음의 문턱"], ["bell", "다음 수 만찬의 종소리"],
  ["temporary", "임시 기물"], ["aimed", "다음 사격 명중률"],
  ["propellant", "다음 관통"], ["entrenched_pending", "진지 유지 중"],
  ["entrenched_ready", "방어 진지 강화"], ["suppressed", "이동 제압"],
  ["reckless", "무모한 돌격"],
];

export function statusVisuals(piece: Pick<Piece, "statuses" | "ward">): StatusVisual[] {
  const result = new Map<StatusKind, StatusVisual>();
  for (const label of piece.statuses ?? []) {
    const kind = /^(공복|굶주림|기아)$/.test(label) ? "hunger" : prefixes.find(([, prefix]) => label === prefix || label.startsWith(`${prefix} `))?.[0];
    if (!kind) continue;
    const definition = definitions[kind];
    const count = kind === "hunger" ? ["공복", "굶주림", "기아"].indexOf(label) + 1
      : Number(label.match(/^(?:광기|죄책감|민첩|약화) (\d+)$/)?.[1]
        ?? (kind === "aimed" || kind === "propellant" ? label.match(/\+(\d+)%p$/)?.[1] : undefined) ?? 1);
    result.set(kind, { kind, ...definition, count, label });
  }
  if (piece.ward) result.set("ward", { kind: "ward", ...definitions.ward, count: 1, label: "가호" });
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

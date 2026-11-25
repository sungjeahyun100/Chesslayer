import type { Snapshot } from "../types";

export type DebugState = Record<string, unknown>;
export type Capture = { state: DebugState; snapshot: Snapshot; saved: string };
export type Change = { path: string; before: unknown; after: unknown; kind: "added" | "removed" | "changed" | "same" | "reset" };
export type Check = { id: number; kind: string; path: string; expected: string; result: "success" | "failure" | "unverified"; detail: string };

export function lifetimeFor(path: string): string {
  if (path.startsWith("progress.")) return "계정/영구";
  if (["battle.mechanics.pending_draw", "battle.mechanics.pending_mana", "battle.mechanics.pending_ammo"]
    .includes(path) || path.startsWith("battle.mechanics.pending_choices")) return "행동";
  if (path === "cards.mana" || path === "cards.draw_locked"
    || path === "battle.actions_left" || path === "battle.normal_actions_used"
    || path.startsWith("battle.mechanics.turn_used")) return "턴";
  if (path.startsWith("cards.") || path.startsWith("battle.mechanics.used")
    || path.startsWith("battle.temporary_upgrades") || path.startsWith("battle.combat_stats")
    || path === "round") return "전투";
  if (["reward_choice", "upgrade_claimed", "shop_offers", "shop_bought", "current_node"].some((key) => path === key || path.startsWith(`${key}[`))) return "노드";
  if (["path", "completed", "last_cleared_node", "last_counted_node", "reward_node"].some((key) => path === key || path.startsWith(`${key}[`))) return "층";
  if (["gold", "deck", "relics", "cleared_stages", "bosses_defeated", "cards_burned", "ammo", "seed"].some((key) => path === key || path.startsWith(`${key}[`))
    || /^roster\[\d+\]\.(kind|alive|upgrades|origin)/.test(path)) return "원정";
  return "미정의";
}

function flatten(value: unknown, path = "", out: Record<string, unknown> = {}): Record<string, unknown> {
  if (value && typeof value === "object" && !Array.isArray(value)) {
    for (const [key, child] of Object.entries(value)) flatten(child, path ? `${path}.${key}` : key, out);
  } else if (Array.isArray(value)) {
    value.forEach((child, index) => flatten(child, `${path}[${index}]`, out));
    if (!value.length) out[path] = [];
  } else out[path] = value;
  return out;
}

export function diff(before: DebugState, after: DebugState): Change[] {
  const { trace_events: _beforeEvents, trace_log: _beforeTrace, log: _beforeLog, ...beforeState } = before;
  const { trace_events: _afterEvents, trace_log: _afterTrace, log: _afterLog, ...afterState } = after;
  const a = flatten(beforeState), b = flatten(afterState);
  const combatBoundary = before.phase === "combat" && after.phase !== "combat";
  const battleStart = before.phase !== "combat" && after.phase === "combat";
  return [...new Set([...Object.keys(a), ...Object.keys(b)])].sort().map((path) => {
    const existsA = Object.hasOwn(a, path), existsB = Object.hasOwn(b, path);
    const oldValue = a[path], newValue = b[path];
    const reset = combatBoundary && ((path === "cards.mana" && typeof oldValue === "number" && newValue === 0)
      || ((path === "reinforcement_used" || path === "hunger_protection_used") && oldValue === true && newValue === false))
      || battleStart && path === "round" && typeof oldValue === "number" && oldValue !== 1 && newValue === 1;
    return { path, before: oldValue, after: newValue,
      kind: !existsA ? "added" : !existsB ? "removed" : Object.is(oldValue, newValue) ? "same"
        : reset ? "reset" : "changed" };
  });
}

export function checkTransition(check: Check, before: Capture, after: Capture, changes: Change[]): Check {
  const values = (state: DebugState) => flatten(state);
  const a = values(before.state), b = values(after.state);
  let valid: boolean | null = null;
  let detail = "명세가 없어 자동 판정하지 않았습니다.";
  if (check.kind === "preserved") {
    const oldValue = Object.hasOwn(before.state, check.path) ? before.state[check.path] : a[check.path];
    const newValue = Object.hasOwn(after.state, check.path) ? after.state[check.path] : b[check.path];
    if (oldValue !== undefined && newValue !== undefined) {
      valid = JSON.stringify(oldValue) === JSON.stringify(newValue);
      detail = `${formatValue(oldValue)} → ${formatValue(newValue)}`;
    }
  } else if (check.kind === "equals") {
    if (Object.hasOwn(b, check.path)) {
      try { valid = Object.is(b[check.path], JSON.parse(check.expected)); }
      catch { valid = String(b[check.path]) === check.expected; }
      detail = `실제 ${String(b[check.path])} / 예상 ${check.expected}`;
    }
  } else if (check.kind === "relic") {
    valid = Array.isArray(after.state.relics) && (after.state.relics as string[]).includes(check.expected);
    detail = `유물 ${check.expected}`;
  } else if (check.kind === "dead") {
    const id = Number(check.expected);
    const oldPiece = (before.state.roster as { id: number; alive: boolean }[] | undefined)?.find((p) => p.id === id);
    const newPiece = (after.state.roster as { id: number; alive: boolean }[] | undefined)?.find((p) => p.id === id);
    if (oldPiece && newPiece && !oldPiece.alive && after.snapshot.floor === before.snapshot.floor) {
      valid = !newPiece.alive;
      detail = `#${id}: 사망 → ${newPiece.alive ? "생존" : "사망"}`;
    }
  } else if (check.kind === "kind_stable") {
    const id = Number(check.expected);
    const oldPiece = (before.state.roster as { id: number; kind: number }[] | undefined)?.find((p) => p.id === id);
    const newPiece = (after.state.roster as { id: number; kind: number }[] | undefined)?.find((p) => p.id === id);
    if (oldPiece && newPiece) {
      valid = oldPiece.kind === newPiece.kind;
      detail = `#${id}: ${oldPiece.kind} → ${newPiece.kind}`;
    }
  } else if (check.kind === "temporary_cleared") {
    valid = after.snapshot.pieces.filter((p) => p.owner === 0).every((p) => !p.temporary_upgrade);
    detail = `남은 일시적 강화: ${after.snapshot.pieces.filter((p) => p.owner === 0 && p.temporary_upgrade).length}`;
  } else if (check.kind === "log_once") {
    const oldLogs = before.snapshot.log;
    const added = after.snapshot.log.slice(oldLogs.length);
    valid = added.filter((line) => line.includes(check.expected)).length === 1;
    detail = `새 로그에서 ${added.filter((line) => line.includes(check.expected)).length}회`;
  } else if (check.kind === "changed") {
    valid = changes.some((item) => item.path === check.path && item.kind !== "same");
    detail = `${check.path}: ${valid ? "변경" : "유지"}`;
  }
  return { ...check, result: valid === null ? "unverified" : valid ? "success" : "failure", detail };
}

export function lifecycle(before: Capture, after: Capture): Check[] {
  const result: Check[] = [];
  const add = (kind: string, path: string, expected = "") => result.push({ id: result.length, kind, path, expected, result: "unverified", detail: "" });
  if (before.snapshot.phase === "combat" && after.snapshot.phase === "reward") {
    add("preserved", "deck"); add("preserved", "relics");
    add("temporary_cleared", "");
    for (const piece of before.state.roster as { id: number; alive: boolean }[]) {
      add("kind_stable", "", String(piece.id));
      if (!piece.alive) add("dead", "", String(piece.id));
    }
  }
  if (before.snapshot.phase === "map" && after.snapshot.phase === "combat") {
    add("preserved", "gold"); add("preserved", "deck"); add("preserved", "relics");
  }
  if (after.snapshot.floor !== before.snapshot.floor) {
    add("preserved", "gold"); add("preserved", "deck"); add("preserved", "relics");
  }
  return result.map((item) => checkTransition(item, before, after, diff(before.state, after.state)));
}

export function formatValue(value: unknown): string {
  if (value === undefined) return "∅";
  return typeof value === "string" ? value : JSON.stringify(value);
}

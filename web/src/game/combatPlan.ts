import type { CombatEvent, Command, Piece, Snapshot } from "../types";

export const combatTiming = {
  move: 110,
  return: 140,
  projectile: 160,
  mortar: 400,
  impactFlash: 180,
  burst: 115,
  machineGun: 40,
  blast: 300,
  nuke: 800,
  shotgunReady: 75,
  shotgunFlight: 240,
  chainExtend: 180,
  chainCatch: 90,
  chainPull: 300,
  chainFade: 90,
} as const;

export type CombatPlan = {
  sequence?: number;
  kind: "move" | "return" | "tracer" | "mortar" | "blast" | "nuke" | "shotgun" | "chain";
  actor: Piece;
  victim?: Piece;
  origin: number;
  target: number;
  result: "none" | "hit" | "miss" | "ward" | "corpse";
  blast: number[];
  count: number;
  interval: number;
  impact: number;
  duration: number;
  impactDone: boolean;
  shotgun?: Extract<CombatEvent, { kind: "shotgun" }>;
  victims?: Piece[];
  chain?: { piece: Piece; from: number; to: number };
};

export type CombatPresentationStep = { plan: CombatPlan; before: Snapshot; after: Snapshot };
type CombatContext = { enemyTarget?: number; blast?: number[] };

// These keys mirror shot_profile in the Rust engine. The count is visual only:
// dispatch still performs precisely one engine action.
const shooters = new Set([
  "pistol_infantry", "rifleman", "mortar", "machine_gun_drone",
  "grenade_launcher_drone", "tank", "he_tank", "ap_tank",
  "fragment_tank", "double_barrel", "sniper", "marksman",
  "commando", "elite_commando", "special_commander",
  "shadow_commander", "tribal_dictator",
]);
export const selfDestructKeys = new Set([
  "drone", "c4_drone", "machine_gun_drone", "tactical_nuke_drone", "grenade_launcher_drone",
]);

function appendedLog(before: string[], next: string[]) {
  for (let overlap = Math.min(before.length, next.length); overlap > 0; overlap--) {
    if (before.slice(-overlap).every((line, index) => line === next[index])) {
      return next.slice(overlap).join(" ");
    }
  }
  return next.join(" ");
}

export function combatPlan(before: Snapshot, next: Snapshot, command: Command,
  context: CombatContext = {}): CombatPlan | null {
  if (before.phase !== "combat") return null;
  const shotgun = next.combat_events?.find((event) => event.kind === "shotgun");
  if (shotgun) {
    const actor = before.pieces.find((piece) => piece.id === shotgun.actor_id)
      ?? next.pieces.find((piece) => piece.id === shotgun.actor_id);
    if (!actor) return null;
    const impact = combatTiming.shotgunReady + combatTiming.shotgunFlight;
    return { kind: "shotgun", actor, origin: shotgun.from, target: shotgun.toward,
      result: "none", blast: shotgun.area, count: 0, interval: 0, shotgun,
      victims: shotgun.hits.flatMap((hit) => {
        const victim = before.pieces.find((piece) => piece.id === hit.piece_id);
        return victim && hit.result !== "miss" && !next.pieces.some((piece) => piece.id === hit.piece_id)
          ? [{ ...victim, cell: hit.cell }] : [];
      }),
      impact, duration: impact + combatTiming.impactFlash, impactDone: false };
  }
  const from = "from" in command ? command.from : undefined;
  const target = "to" in command ? command.to : context.enemyTarget;
  if (typeof from !== "number" || typeof target !== "number" || target < 0 || target >= 64) return null;
  const actor = before.pieces.find((piece) => piece.cell === from);
  if (!actor) return null;
  const afterActor = next.pieces.find((piece) => piece.id === actor.id);
  const beforeTarget = before.pieces.find((piece) => piece.cell === target && piece.id !== actor.id);
  const afterTarget = beforeTarget && next.pieces.find((piece) => piece.id === beforeTarget.id);
  const newLog = appendedLog(before.log, next.log);
  const ward = !!beforeTarget?.ward && !afterTarget?.ward && !!afterTarget;
  const corpse = !!beforeTarget && afterTarget?.key === "corpse" && beforeTarget.key !== "corpse";
  const miss = /사격 빗나감/.test(newLog);
  const result = ward ? "ward" : corpse ? "corpse" : miss ? "miss" : beforeTarget ? "hit" : "none";

  if (from === target && (command.type === "special_action" || command.type === "enemy_step_selected") &&
      selfDestructKeys.has(actor.key)) {
    const nuke = actor.key === "tactical_nuke_drone";
    return { kind: nuke ? "nuke" : "blast", actor, origin: from, target, result: "hit",
      blast: context.blast ?? [], count: 0, interval: 0, impact: 0,
      duration: nuke ? combatTiming.nuke : combatTiming.blast, impactDone: false };
  }
  // A copied single-target firearm still uses the ordinary tracer. The boss's
  // native shotgun is identified exclusively by the engine event above.
  const bossShot = actor.key === "tribal_dictator" && /사격 (명중|빗나감)/.test(newLog);
  if (from !== target && shooters.has(actor.key) && (actor.key !== "tribal_dictator" || bossShot) &&
      (command.type === "special_action" || command.type === "enemy_step_selected" || command.type === "move") &&
      (afterActor?.cell === from || !afterActor) && beforeTarget) {
    const kind = actor.key === "mortar" ? "mortar" : "tracer";
    const count = actor.key === "machine_gun_drone" ? 10 : ["pistol_infantry", "tribal_dictator"].includes(actor.key) ? 1 : 3;
    const interval = actor.key === "machine_gun_drone" ? combatTiming.machineGun : combatTiming.burst;
    const impact = kind === "mortar" ? combatTiming.mortar : (count - 1) * interval + combatTiming.projectile;
    return { kind, actor, victim: beforeTarget && !afterTarget ? beforeTarget : undefined,
      origin: from, target, result, blast: [], count, interval, impact,
      duration: impact + combatTiming.impactFlash, impactDone: false };
  }
  if (actor.key === "tribal_dictator" && afterActor?.cell === from) return null;
  if (from !== target && (afterActor?.cell === target || (beforeTarget && afterActor?.cell === from))) {
    const returns = afterActor?.cell === from;
    const kind = returns ? "return" : "move";
    return { kind, actor, victim: beforeTarget && !afterTarget ? beforeTarget : undefined,
      origin: from, target, result, blast: [], count: 0, interval: 0,
      impact: combatTiming.move,
      duration: returns ? combatTiming.move + combatTiming.return : beforeTarget ? combatTiming.move + combatTiming.impactFlash : combatTiming.move,
      impactDone: false };
  }
  return null;
}

function pieceAtCell(piece: Piece, cell: number): Piece {
  return { ...piece, cell, square: `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}` };
}

function movePresentedPiece(state: Snapshot, fallback: Snapshot, id: number, cell: number): Snapshot {
  const piece = state.pieces.find((entry) => entry.id === id) ?? fallback.pieces.find((entry) => entry.id === id);
  if (!piece) return state;
  return { ...state, pieces: [...state.pieces.filter((entry) => entry.id !== id), pieceAtCell(piece, cell)] };
}

function chainPlan(before: Snapshot, next: Snapshot,
  event: Extract<CombatEvent, { kind: "chain_pull" }>): CombatPlan | null {
  const actor = before.pieces.find((piece) => piece.id === event.boss_id)
    ?? next.pieces.find((piece) => piece.id === event.boss_id);
  const piece = before.pieces.find((piece) => piece.id === event.piece_id)
    ?? next.pieces.find((piece) => piece.id === event.piece_id);
  if (!actor || !piece) return null;
  const impact = combatTiming.chainExtend + combatTiming.chainCatch + combatTiming.chainPull;
  return { kind: "chain", actor: pieceAtCell(actor, event.boss_cell),
    origin: event.boss_cell, target: event.from, result: "none", blast: [], count: 0, interval: 0,
    chain: { piece: pieceAtCell(piece, event.from), from: event.from, to: event.to },
    impact, duration: impact + combatTiming.chainFade, impactDone: false };
}

// Only the displayed piece positions are staged. All commands have already been
// resolved by Rust, and every pull endpoint comes directly from that result.
export function combatPresentationSteps(before: Snapshot, next: Snapshot, command: Command,
  context: CombatContext = {}): CombatPresentationStep[] {
  if (before.phase !== "combat") return [];
  const pulls = (next.combat_events ?? []).filter((event) => event.kind === "chain_pull");
  if (!pulls.length) {
    const plan = combatPlan(before, next, command, context);
    return plan ? [{ plan, before, after: next }] : [];
  }
  const steps: CombatPresentationStep[] = [];
  // Entering the enemy turn happens after the player's action is resolved.
  // Rewind only the reported pulls so that action makes contact before the
  // chain carries its target away. An enemy action can instead follow a pull.
  const pullsAfterAction = before.turn !== 1;
  let current = before;
  if (pullsAfterAction) {
    const beforePulls = [...pulls].reverse().reduce((state, event) =>
      movePresentedPiece(state, before, event.piece_id, event.from), next);
    const plan = combatPlan(before, beforePulls, command, context);
    if (plan) steps.push({ plan, before, after: beforePulls });
    current = beforePulls;
  }
  for (const event of pulls) {
    const after = movePresentedPiece(current, next, event.piece_id, event.to);
    const plan = chainPlan(current, next, event);
    if (plan) steps.push({ plan, before: current, after });
    current = after;
  }
  if (!pullsAfterAction) {
    const plan = combatPlan(current, next, command, context);
    if (plan) steps.push({ plan, before: current, after: next });
  }
  if (steps.length) steps[steps.length - 1].after = next;
  return steps;
}

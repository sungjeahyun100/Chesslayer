import { readFileSync } from "node:fs";
import { initSync, WebGame } from "../../src/wasm/chesslayer_web.js";
import type { Command, Snapshot } from "../../src/types";

initSync({ module: readFileSync(new URL("../../src/wasm/chesslayer_web_bg.wasm", import.meta.url)) });
const progress = JSON.stringify({ highest_floor: 4, unlocked_squads: ["guardian", "vanguard", "heretic", "artillery", "special"], final_unlocked: true, final_defeated: false });
export const snapshot = (game: WebGame): Snapshot => JSON.parse(game.snapshot());
export const dispatch = (game: WebGame, command: Command): Snapshot => JSON.parse(game.dispatch(JSON.stringify(command)));
export function newRun(character = "guardian", seed = 2026) {
  const game = WebGame.start_run(character, seed, 1, progress);
  game.set_developer_mode(true);
  return game;
}
export function enemyTurn(game: WebGame) {
  let state = snapshot(game);
  while (state.phase === "combat" && state.turn === 1) state = dispatch(game, { type: "enemy_step" });
}

// Short, legal hill-king victories reach UI checkpoints without injecting state.
// These fixtures deliberately use the existing developer upgrade command.
const hillPlans = new Map<string, Command[]>();
export function finishHillBattle(game: WebGame) {
  const initial = game.save();
  let plan = hillPlans.get(initial);
  if (!plan) {
    let frontier: { saved: string; moves: Command[]; score: number }[] = [{ saved: initial, moves: [], score: 0 }];
    const seen = new Set<string>();
    search: for (let depth = 0; depth < 9; depth++) {
      const next: typeof frontier = [];
      for (const candidate of frontier) {
        const probe = WebGame.restore(candidate.saved);
        const state = snapshot(probe);
        probe.free();
        const king = state.pieces.find(p => p.owner === 0 && p.royal)!;
        const distance = (cell: number) => Math.min(...[27, 28, 35, 36].map(goal => Math.max(Math.abs(cell % 8 - goal % 8), Math.abs(Math.floor(cell / 8) - Math.floor(goal / 8)))));
        const moves = state.moves.filter(m => !m.promotion).sort((a, b) =>
          (a.from === king.cell ? distance(a.to) - 20 : Math.abs(a.from % 8 - king.cell % 8)) -
          (b.from === king.cell ? distance(b.to) - 20 : Math.abs(b.from % 8 - king.cell % 8))).slice(0, 12);
        for (const move of moves) {
          const trial = WebGame.restore(candidate.saved);
          const command: Command = { type: "move", from: move.from, to: move.to };
          try {
            dispatch(trial, command);
            enemyTurn(trial);
            const after = snapshot(trial);
            const commands = [...candidate.moves, command];
            if (after.phase === "reward") { plan = commands; break search; }
            if (after.phase !== "combat") continue;
            const key = JSON.stringify(after.pieces.map(p => [p.id, p.cell, p.kind]));
            if (seen.has(key)) continue;
            seen.add(key);
            const royal = after.pieces.find(p => p.owner === 0 && p.royal)!;
            next.push({ saved: trial.save(), moves: commands, score: -distance(royal.cell) * 100 + after.pieces.filter(p => p.owner === 0).length });
          } finally { trial.free(); }
        }
      }
      frontier = next.sort((a, b) => b.score - a.score).slice(0, 16);
    }
    if (!plan) throw new Error("Could not find a legal hill-king UI checkpoint");
    hillPlans.set(initial, plan);
  }
  for (const command of plan) { enemyTurn(game); dispatch(game, command); }
  if (snapshot(game).phase !== "reward") throw new Error("UI fixture must finish in reward phase");
}
export function winHillBattle(game: WebGame, node: number) {
  dispatch(game, { type: "select_node", id: node });
  finishHillBattle(game);
}
export function routesTo(nodes: Snapshot["map"]["nodes"], id: number): number[][] {
  if (nodes[id].row === 0) return [[id]];
  return nodes.filter((parent) => parent.next.includes(id))
    .flatMap((parent) => routesTo(nodes, parent.id).map((route) => [...route, id]));
}
export function travelRoute(game: WebGame, route: number[], claimCards = false) {
  for (const id of route) {
    const node = snapshot(game).map.nodes[id];
    if (node.kind === "combat" || node.kind === "elite" || node.kind === "boss") {
      winHillBattle(game, id);
      if (claimCards) {
        const card = snapshot(game).rewards.findIndex((reward) => reward.type === "card");
        if (card >= 0) dispatch(game, { type: "claim_reward", index: card });
      }
      dispatch(game, { type: "continue_reward" });
    } else {
      dispatch(game, { type: "select_node", id });
      if (node.kind === "rest") dispatch(game, { type: "rest_skip" });
      else if (node.kind === "treasure") dispatch(game, { type: "claim_treasure" });
      else if (node.kind === "shop") dispatch(game, { type: "shop_leave" });
      else if (node.kind === "event") resolveEvent(game);
    }
  }
}
const checkpoints = new Map<string, { saved: string; state: Snapshot }>();
export function checkpoint(target: "elite" | "rest" | "temporary" | "casualties") {
  const cached = checkpoints.get(target);
  if (cached) return cached;
  const game = newRun("guardian", target === "temporary" ? 0 : target === "elite" ? 2026 : 1);
  try {
    const king = snapshot(game).pieces.find((piece) => piece.key === "king")!;
    dispatch(game, { type: "developer_set_upgrade", id: king.id, target: 94 });
    if (target === "temporary") {
      const nodes = snapshot(game).map.nodes;
      const start = nodes.find((node) => node.row === 0
        && node.next.some((id) => nodes[id].kind === "combat"))!;
      winHillBattle(game, start.id);
      const index = snapshot(game).rewards.findIndex((reward) => reward.type === "card" && reward.definition.kind === "temporary_upgrade");
      if (index < 0) throw new Error("UI fixture requires a temporary-upgrade reward");
      dispatch(game, { type: "claim_reward", index });
      dispatch(game, { type: "continue_reward" });
      const next = snapshot(game).map.nodes.find((node) => node.available && !node.visited && node.kind === "combat")!;
      dispatch(game, { type: "select_node", id: next.id });
    } else {
      const nodes = snapshot(game).map.nodes;
      const destination = nodes.find((node) => node.row === 1
        && node.kind === (target === "elite" ? "elite" : "rest"))!;
      const route = routesTo(nodes, destination.id).find((ids) =>
        ids.slice(0, -1).every((id) => ["combat", "elite"].includes(nodes[id].kind)))!;
      if (!route) throw new Error(`No combat route reaches ${target}`);
      for (const [index, id] of route.entries()) {
        if (id === destination.id && target !== "elite") break;
        if (target === "casualties" && index === 0) {
          dispatch(game, { type: "select_node", id });
          for (const [from, to] of [[52,36],[60,52],[52,43],[43,35]]) { enemyTurn(game); dispatch(game, {type:"move",from,to}); }
        } else winHillBattle(game, id);
        if (id !== destination.id) dispatch(game, { type: "continue_reward" });
      }
      if (target === "rest" || target === "casualties") {
        dispatch(game, { type: "select_node", id: destination.id });
      }
    }
    const checkpoint = { saved: game.save(), state: snapshot(game) };
    checkpoints.set(target, checkpoint);
    return checkpoint;
  } finally { game.free(); }
}

export function resolveEvent(game: WebGame) {
  while (snapshot(game).phase === "event") {
    const event = snapshot(game).event!;
    const choice = event.choices.find(c => c.id === "leave" && !c.disabled) ?? event.choices.find(c => !c.disabled)!;
    dispatch(game, { type: "event_choice", choice: choice.id });
  }
  if (snapshot(game).phase === "combat") {
    finishHillBattle(game);
    dispatch(game, { type: "continue_reward" });
  }
}

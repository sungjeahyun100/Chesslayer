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
export function winHillBattle(game: WebGame, node: number) {
  dispatch(game, { type: "select_node", id: node });
  for (const [from, to] of [[52, 36], [60, 52], [52, 43], [43, 35]]) {
    enemyTurn(game);
    dispatch(game, { type: "move", from, to });
  }
  if (snapshot(game).phase !== "reward") throw new Error("UI fixture must finish in reward phase");
}
export function checkpoint(target: "elite" | "rest" | "temporary") {
  const game = newRun("guardian", target === "temporary" ? 0 : 2026);
  try {
    const king = snapshot(game).pieces.find((piece) => piece.key === "king")!;
    dispatch(game, { type: "developer_set_upgrade", id: king.id, target: 94 });
    if (target === "temporary") {
      winHillBattle(game, 0);
      const index = snapshot(game).rewards.findIndex((reward) => reward.type === "card" && reward.definition.kind === "temporary_upgrade");
      if (index < 0) throw new Error("UI fixture requires a temporary-upgrade reward");
      dispatch(game, { type: "claim_reward", index });
      dispatch(game, { type: "continue_reward" });
      const next = snapshot(game).map.nodes.find((node) => node.available && !node.visited && node.kind === "combat")!;
      dispatch(game, { type: "select_node", id: next.id });
    } else {
      for (const node of [1, 5, 8]) {
        winHillBattle(game, node);
        if (node !== 8 || target === "rest") dispatch(game, { type: "continue_reward" });
      }
      if (target === "rest") {
        dispatch(game, { type: "select_node", id: 12 });
        dispatch(game, { type: "event_choice", choice: "eat" });
        dispatch(game, { type: "select_node", id: 14 });
      }
    }
    return { saved: game.save(), state: snapshot(game) };
  } finally { game.free(); }
}

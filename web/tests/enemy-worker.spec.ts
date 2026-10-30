import { test, expect } from "@playwright/test";

test("enemy search runs in a worker and its legal result replays", async ({ page }) => {
  await page.goto("/");
  const result = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = new WebGame("guardian", 2026);
    const start = JSON.parse(game.snapshot());
    const node = start.map.nodes.find((item: { available: boolean; kind: string }) => item.available && item.kind === "combat");
    game.dispatch(JSON.stringify({ type: "select_node", id: node.id }));
    game.dispatch(JSON.stringify({ type: "end_turn" }));
    const saved = game.save();
    let ticks = 0;
    const interval = setInterval(() => ticks++, 1);
    const worker = new Worker(new URL("/src/enemyWorker.ts", location.href), { type: "module" });
    const message = await new Promise<{ choice: [number, number] | null; error?: string }>((resolve, reject) => {
      worker.onmessage = (event) => resolve(event.data);
      worker.onerror = (event) => reject(new Error(event.message));
      worker.postMessage(saved);
    });
    clearInterval(interval);
    worker.terminate();
    if (message.error) throw new Error(message.error);
    const [from, index] = message.choice ?? [null, null];
    const after = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step_selected", from, index })));
    const replay = WebGame.restore(game.save());
    const restored = JSON.parse(replay.snapshot());
    replay.free();
    game.free();
    return { ticks, choice: message.choice, turn: after.turn, restoredTurn: restored.turn };
  });
  expect(result.ticks).toBeGreaterThan(0);
  expect(result.choice).not.toBeNull();
  expect(result.turn).toBe(result.restoredTurn);
});

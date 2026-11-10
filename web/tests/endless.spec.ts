import { expect, test } from "@playwright/test";
import { WebGame } from "../src/wasm/chesslayer_web";
import { dispatch, enemyTurn, newRun, snapshot, winHillBattle } from "./helpers/uiRuns";
import { ENDLESS_BEST_KEY, ENDLESS_SAVE_KEY } from "../src/endless";

function checkpoints() {
  const game = WebGame.start_run_with_mode("guardian", 0, 1, "{}", "endless");
  game.set_developer_mode(true);
  try {
    dispatch(game, { type: "developer_set_upgrade", id: 5, target: 94 });
    winHillBattle(game, 1);
    const reward = game.save();
    dispatch(game, { type: "continue_reward" });
    dispatch(game, { type: "developer_set_upgrade", id: 5, target: 6 });
    dispatch(game, { type: "select_node", id: 5 });
    // Legal forward moves reach a real defeat; no injected snapshots or test API.
    for (let turn = 0; turn < 100 && snapshot(game).phase === "combat"; turn++) {
      enemyTurn(game);
      const state = snapshot(game);
      if (state.phase !== "combat") break;
      const king = state.pieces.find((piece) => piece.owner === 0 && piece.royal);
      const forward = state.moves.filter((move) => move.to < move.from).sort((a, b) => a.to - b.to);
      const move = forward.find((move) => move.from === king?.cell) ?? forward[0];
      dispatch(game, move ? { type: "move", from: move.from, to: move.to, ...(move.promotion ? { promotion: move.promotion.kind } : {}) } : { type: "end_turn" });
    }
    expect(snapshot(game).phase).toBe("defeat");
    expect(snapshot(game).cleared_stages).toBe(1);
    return { reward, defeat: game.save() };
  } finally { game.free(); }
}

test("endless records only write improvements, sanitize bad storage and tolerate failures", async ({ page }) => {
  await page.goto("/");
  const result = await page.evaluate(async () => {
    const { readEndlessBest, updateEndlessBest, ENDLESS_BEST_KEY } = await import("/src/endless.ts");
    const original = Storage.prototype.setItem;
    let writes = 0;
    Storage.prototype.setItem = function (key, value) { if (key === ENDLESS_BEST_KEY) writes++; original.call(this, key, value); };
    const missing = readEndlessBest();
    updateEndlessBest({ clearedStages: 47, highestFloor: 4 });
    updateEndlessBest({ clearedStages: 47, highestFloor: 4 });
    updateEndlessBest({ clearedStages: 3, highestFloor: 1 });
    const best = readEndlessBest();
    const bestWrites = writes;
    original.call(localStorage, ENDLESS_BEST_KEY, "broken json");
    const corrupt = readEndlessBest();
    original.call(localStorage, ENDLESS_BEST_KEY, JSON.stringify({ clearedStages: -1, highestFloor: "4" }));
    const invalid = readEndlessBest();
    Storage.prototype.setItem = () => { throw new Error("storage disabled"); };
    const unsaved = updateEndlessBest({ clearedStages: 2, highestFloor: 1 });
    Storage.prototype.setItem = original;
    return { missing, best, bestWrites, corrupt, invalid, unsaved };
  });
  expect(result).toEqual({ missing: { clearedStages: 0, highestFloor: 0 }, best: { clearedStages: 47, highestFloor: 4 }, bestWrites: 1,
    corrupt: { clearedStages: 0, highestFloor: 0 }, invalid: { clearedStages: 0, highestFloor: 0 }, unsaved: { clearedStages: 2, highestFloor: 1 } });
});

test("endless start, reward revisit, reload and defeat keep records and campaign saves separate", async ({ page }) => {
  const { reward, defeat } = checkpoints();
  const normal = newRun();
  const oldSave = JSON.parse(normal.save());
  delete oldSave.mode;
  normal.free();
  const campaign = JSON.stringify(oldSave);
  const progress = JSON.stringify(oldSave.progress);
  const previousBest = { clearedStages: 0, highestFloor: 0 };
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.evaluate(({ campaign, progress, reward, previousBest, key }) => {
    localStorage.setItem("chesslayer.run.v1", campaign);
    localStorage.setItem("chesslayer.progress.v1", progress);
    localStorage.setItem(key, JSON.stringify({ run: reward, developerRun: true, previousBest }));
  }, { campaign, progress, reward, previousBest, key: ENDLESS_SAVE_KEY });
  await page.reload();
  await expect(page.getByRole("region", { name: "무한 모드", exact: true })).toContainText("0단계 클리어 · 0층 도달");
  await page.getByRole("button", { name: "무한 원정 이어하기" }).click();
  await expect(page.locator(".node-navigation")).toContainText("누적 1단계 클리어");
  await page.locator(".reward-continue").click();
  await page.locator('.map-node[data-node-id="1"]').click();
  await page.locator(".reward-continue").click();
  await expect(page.locator(".map-heading")).toContainText("누적 1단계 클리어");
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), ENDLESS_BEST_KEY)).toEqual({ clearedStages: 1, highestFloor: 1 });
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(campaign);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.progress.v1"))).toBe(progress);

  await page.evaluate(({ defeat, previousBest, key }) => localStorage.setItem(key, JSON.stringify({ run: defeat, developerRun: true, previousBest })), { defeat, previousBest, key: ENDLESS_SAVE_KEY });
  await page.reload();
  await expect(page.getByRole("region", { name: "무한 모드", exact: true })).toContainText("1단계 클리어 · 1층 도달");
  await page.getByRole("button", { name: "무한 원정 이어하기" }).click();
  await expect(page.locator(".endless-result")).toContainText("신기록!");
  await expect(page.locator(".endless-result")).toContainText("이번 무한 원정: 1단계 클리어 · 1층 도달");
  await expect(page.locator(".endless-result")).toContainText("원정 시작 전 최고기록: 0단계 클리어 · 0층 도달");
  await page.reload();
  await page.getByRole("button", { name: "무한 원정 시작", exact: false }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator(".map-heading")).toContainText("누적 0단계 클리어");
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), ENDLESS_BEST_KEY)).toEqual({ clearedStages: 1, highestFloor: 1 });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator(".map-heading h1")).toHaveText("갈림길의 숲");
  await expect(page.locator(".map-heading")).not.toContainText("무한 원정");
});

test("unavailable storage still allows starting and playing endless mode on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.addInitScript(() => {
    Object.defineProperty(window, "localStorage", { get() { throw new Error("storage disabled"); } });
    crypto.getRandomValues = ((array: Uint32Array) => { array.fill(0); return array; }) as Crypto["getRandomValues"];
  });
  await page.goto("/");
  await page.getByRole("button", { name: "무한 원정 시작" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator(".map-heading")).toContainText("무한 원정");
  await page.locator('.map-node[data-node-id="1"]').click();
  await page.locator('.board [data-square="e2"]').click();
  await page.locator('.board [data-square="e4"]').click();
  await expect(page.getByRole("button", { name: "e4 백 폰", exact: true })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
});

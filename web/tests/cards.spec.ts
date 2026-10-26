import { test, expect } from "@playwright/test";

test("일시 강화: 경로 선택과 취소는 무소모, 확정만 전투 보드에 적용", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  const saved = await page.evaluate(async () => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = new WebGame("vanguard", 0);
    try {
      for (const command of [
        { type: "select_node", id: 0 },
        { type: "move", from: 50, to: 42 },
        { type: "move", from: 59, to: 32 },
        { type: "move", from: 32, to: 4 },
        { type: "choose_reward", index: 2 },
        { type: "select_node", id: 4 },
      ]) {
        let state = JSON.parse(game.snapshot());
        while (state.phase === "combat" && state.turn === 1)
          state = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" })));
        game.dispatch(JSON.stringify(command));
      }
      return game.save();
    } finally { game.free(); }
  });
  await page.evaluate((value) => localStorage.setItem("chesslayer.run.v1", value), saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  const card = page.locator(".hand .playing-card");
  await expect(card).toContainText("일시 강화");
  await expect(card.locator(".cost")).toHaveText("N");
  await expect(card.locator(".card-art")).toBeVisible();
  const before = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  await card.click();
  await page.locator('[data-square="a2"]').click();
  const panel = page.locator(".card-upgrade-picker");
  await expect(panel).toContainText("폰 · a2");
  await expect(panel).not.toContainText("CARD UPGRADE");
  await expect(panel.locator(".card-upgrade-confirm")).toHaveCount(0);
  const boardBox = await page.locator(".board-frame").boundingBox();
  const panelBox = await panel.boundingBox();
  expect(panelBox!.width).toBeGreaterThanOrEqual(380);
  expect(panelBox!.width).toBeLessThanOrEqual(500);
  expect(panelBox!.x).toBeGreaterThanOrEqual(boardBox!.x + boardBox!.width);
  await page.locator('.card-upgrade-picker button[data-kind="8"]').click();
  await expect(panel.locator(".card-upgrade-selection")).toContainText("전진");
  await expect(panel.locator(".card-upgrade-confirm")).toBeVisible();
  await page.screenshot({ path: "test-results/card-upgrade-panel.png", fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect((await panel.boundingBox())!.width).toBeLessThanOrEqual(390);
  await expect.poll(() => panel.locator(".card-upgrade-actions").evaluate((el) => el.getBoundingClientRect().bottom <= innerHeight)).toBe(true);
  await page.setViewportSize({ width: 390, height: 600 });
  await expect.poll(() => panel.locator(".card-upgrade-actions").evaluate((el) => el.getBoundingClientRect().bottom <= innerHeight)).toBe(true);
  await page.setViewportSize({ width: 1440, height: 1000 });
  await panel.locator(".card-upgrade-confirm").click();
  await expect(page.locator(".action-hint")).toContainText("선택 1기 · 현재 코스트 1 / 마나 3");
  await expect(card.locator(".cost")).toHaveText("1");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('[data-square="b2"]').click();
  await expect(page.locator(".card-upgrade-picker")).toBeInViewport();
  await page.locator('.card-upgrade-picker button[data-kind="8"]').click();
  await page.locator(".card-upgrade-confirm").click();
  await page.setViewportSize({ width: 1440, height: 1000 });
  await expect(card.locator(".cost")).toHaveText("2");
  await expect(page.locator(".action-hint")).toContainText("선택 2기 · 현재 코스트 2 / 마나 3");
  await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", "pawn");
  await page.keyboard.press("Escape");
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await expect(card).toBeVisible();
  await card.click();
  await page.locator('[data-square="a2"]').click();
  await page.locator('.card-upgrade-picker button[data-kind="8"]').click();
  await page.locator(".card-upgrade-confirm").click();
  await page.locator(".action-hint .primary").click();
  await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", "tempest_pawn");
  await expect(page.locator('[data-piece-id="9"] .temporary-mark')).toBeVisible();
  await expect(page.locator(".effect-panel")).toContainText("일시 강화");
  await expect(page.locator(".mana-orb")).toHaveText("2");
  await expect(card).toHaveCount(0);
  const rosterKind = await page.evaluate(async () => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()).rest_options.find((p: { id: number }) => p.id === 9).kind; }
    finally { game.free(); }
  });
  expect(rosterKind).toBe(1);
});

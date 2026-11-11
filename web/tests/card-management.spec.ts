import { expect, test, type Page } from "@playwright/test";
import type { Snapshot } from "../src/types";
import { dispatch, enemyTurn, newRun, snapshot } from "./helpers/uiRuns";

async function startCombat(page: Page) {
  await page.addInitScript(() => {
    const original = crypto.getRandomValues.bind(crypto);
    crypto.getRandomValues = ((array: Uint32Array) => {
      if (array instanceof Uint32Array && array.length === 1) { array[0] = 2026; return array; }
      return original(array);
    }) as typeof crypto.getRandomValues;
  });
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await page.locator("button.character").filter({ hasText: "선봉대" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator(".map-node:enabled")).toHaveCount(3);
  await page.locator('.map-node:enabled[data-kind="combat"]').first().click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
}

async function savedState(page: Page): Promise<Snapshot> {
  return page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()); } finally { game.free(); }
  });
}

async function resumeCombat(page: Page, value: string) {
  await page.goto("/");
  await page.evaluate((saved) => {
    localStorage.setItem("chesslayer.run.v1", saved);
    localStorage.setItem("chesslayer.developer_mode.v1", "true");
    localStorage.setItem("chesslayer.run.developer.v1", "true");
  }, value);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
}

async function moveOnce(page: Page) {
  const move = (await savedState(page)).moves.find((item) => !item.promotion);
  expect(move).toBeTruthy();
  const square = (cell: number) => `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;
  await page.locator(`[data-square="${square(move!.from)}"]`).click();
  await page.locator(`[data-square="${square(move!.to)}"]`).click();
}

test("초기 드로우와 덱·묘지 목록 조회가 전투 상태를 바꾸지 않는다", async ({ page }) => {
  await startCombat(page);
  await expect(page.locator(".card-flight-inner").first()).toBeVisible();
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator(".card-motion-layer")).toHaveCount(0);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  const state = await savedState(page);
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(state.hand.length);
  await expect(page.getByTestId("draw-count")).toHaveText(String(state.draw_count));
  const saved = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  await page.locator('[data-pile="draw"]').click();
  await expect(page.getByRole("dialog", { name: "덱 카드 목록" }).locator(".playing-card")).toHaveCount(state.draw_count);
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "덱 카드 목록" })).toHaveCount(0);
  await page.locator('[data-pile="discard"]').click();
  await expect(page.getByRole("dialog", { name: "묘지 카드 목록" }).locator(".playing-card")).toHaveCount(state.discard_count);
  await page.getByRole("button", { name: "카드 목록 닫기" }).click();
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(saved);
});

test("카드 사용은 손패에서 실제 목적지로 이동하고 표시가 엔진과 일치한다", async ({ page }) => {
  await startCombat(page);
  await expect(page.locator(".card-motion-layer")).toHaveCount(0);
  const before = await savedState(page);
  const card = before.hand.find((candidate) => !candidate.disabled_reason);
  expect(card).toBeTruthy();
  await page.locator(`.hand [data-card-id="${card!.id}"]`).click();
  if (card!.definition.targeted) await page.locator(".square.targetable").first().click();
  await expect(page.locator(`.hand [data-card-id="${card!.id}"]`)).toHaveCount(0);
  await expect(page.locator(".card-flight").first()).toBeVisible();
  await expect(page.locator(".card-motion-layer")).toHaveCount(0);
  const after = await savedState(page);
  expect(after.hand.map((item) => item.id)).not.toContain(card!.id);
  const destination = card!.definition.exhaust ? after.exhausted_pile : after.discard_pile;
  expect(destination.map((item) => item.id)).toContain(card!.id);
  await expect(page.locator('[data-pile="discard"]')).toContainText(String(after.discard_count));
});

test("턴 종료의 다중 버리기와 이어지는 드로우가 전투 상태를 따라간다", async ({ page }) => {
  const game = newRun("vanguard");
  dispatch(game, { type: "select_node", id: 0 });
  for (let turn = 0; turn < 4; turn++) {
    dispatch(game, { type: "end_turn" });
    enemyTurn(game);
  }
  const ready = snapshot(game);
  expect(ready.hand).toHaveLength(5);
  const value = game.save();
  game.free();
  await resumeCombat(page, value);
  await moveOnce(page);
  await expect(page.getByTestId("turn")).toContainText("1회 행동");
  await moveOnce(page);
  await expect(page.locator(".card-flight")).not.toHaveCount(0);
  await expect(page.locator(".card-motion-layer")).toHaveCount(0, { timeout: 10_000 });
  const after = await savedState(page);
  expect(after.discard_count).toBeGreaterThanOrEqual(5);
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(after.hand.length);
  await page.locator('[data-pile="discard"]').click();
  await expect(page.getByRole("dialog", { name: "묘지 카드 목록" }).locator(".playing-card")).toHaveCount(after.discard_count);
});

test("덱 소진 뒤 기존 재구성 규칙으로 묘지 카드가 다시 드로우된다", async ({ page }) => {
  const game = newRun("artillery");
  dispatch(game, { type: "select_node", id: 0 });
  for (let turn = 0; turn < 5; turn++) {
    dispatch(game, { type: "end_turn" });
    enemyTurn(game);
  }
  const ready = snapshot(game);
  expect(ready.draw_count).toBe(0);
  expect(ready.discard_count).toBeGreaterThan(0);
  const value = game.save();
  game.free();
  await resumeCombat(page, value);
  await moveOnce(page);
  await expect(page.locator(".card-flight")).not.toHaveCount(0);
  await expect(page.locator(".card-motion-layer")).toHaveCount(0, { timeout: 10_000 });
  const after = await savedState(page);
  expect(after.discard_count).toBe(0);
  expect(after.hand.length).toBeGreaterThan(ready.hand.length);
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(after.hand.length);
});

test("빠른 연속 카드 사용 중 화면을 다시 열어도 엔진 상태가 유지된다", async ({ page }) => {
  const game = newRun("vanguard");
  dispatch(game, { type: "select_node", id: 0 });
  for (let turn = 0; turn < 4; turn++) {
    dispatch(game, { type: "end_turn" });
    enemyTurn(game);
  }
  const value = game.save();
  game.free();
  await resumeCombat(page, value);
  const before = await savedState(page);
  const cards = before.hand.filter((card) => card.definition.kind === "stimpack" && !card.disabled_reason);
  expect(cards).toHaveLength(2);
  for (const card of cards) {
    await page.locator(`.hand [data-card-id="${card.id}"]`).click();
    await page.locator(".square.targetable").first().click();
  }
  await expect(page.locator(".card-flight")).not.toHaveCount(0);
  const committed = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1")!);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator(".card-motion-layer")).toHaveCount(0);
  const after = await savedState(page);
  expect(after.discard_pile.map((card) => card.id)).toEqual(expect.arrayContaining(cards.map((card) => card.id)));
  expect(after.hand.length).toBe(before.hand.length - 2);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(committed);
});

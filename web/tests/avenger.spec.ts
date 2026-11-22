import { test, expect } from "@playwright/test";
import type { Snapshot } from "../src/types";

test("복수귀는 카드로 행마권을 얻고 수동 턴 종료와 저장 복원을 사용한다", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", e => errors.push(e.message));
  await page.goto("/");
  const prepared = await page.evaluate(async () => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    for (let seed = 1; seed < 100; seed++) {
      const game = new WebGame("avenger", seed);
      try {
        const map = JSON.parse(game.snapshot()) as Snapshot;
        const state = JSON.parse(game.dispatch(JSON.stringify({ type: "select_node", id: map.map.nodes.find(n => n.available)!.id }))) as Snapshot;
        if (!state.hand.some(c => c.definition.kind === "pawn_order")) continue;
        localStorage.setItem("chesslayer.run.v1", game.save());
        return { card: state.hand.find(c => c.definition.kind === "pawn_order")!.id,
          pawn: state.pieces.find(p => p.id === 9)!.cell, handCount: state.hand.length };
      } finally { game.free(); }
    }
    throw new Error("pawn-order starter not found");
  });
  expect(prepared.handCount).toBe(5);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  await expect(page.locator(".combat-resources")).toContainText("마나 3 / 3");
  await expect(page.locator(".combat-resources")).toContainText("포탄량 10 / 10");
  await page.locator(".square").nth(prepared.pawn).click();
  await expect(page.locator(".square[data-legal=true]")).toHaveCount(0);
  await page.locator(`[data-card-id="${prepared.card}"]`).click();
  await page.locator(".square").nth(prepared.pawn).click();
  await expect(page.locator('.piece[data-piece-id="9"] .piece-stats')).toContainText("↪1");
  await page.locator(".square").nth(prepared.pawn).click();
  await page.locator(".square").nth(prepared.pawn - 8).click();
  await expect(page.getByTestId("turn")).toContainText("나의 턴");
  await expect(page.locator('.piece[data-piece-id="9"]')).toHaveAttribute("data-cell", String(prepared.pawn - 8));
  await expect(page.locator('.piece[data-piece-id="9"] .piece-stats')).toContainText("↪0");
  await page.screenshot({ path: "test-results/avenger-desktop.png", fullPage: true });
  await page.getByRole("button", { name: "턴 종료", exact: true }).click();
  await expect(page.getByTestId("turn")).toContainText("나의 턴", { timeout: 20000 });
  await expect(page.locator(".hand .playing-card")).toHaveCount(5);
  const saved = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1")!);
  expect(JSON.parse(saved).version).toBe(24);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.getByTestId("turn")).toContainText("나의 턴");
  await expect(page.locator('.piece[data-piece-id="9"]')).toHaveAttribute("data-cell", String(prepared.pawn - 8));
  await page.setViewportSize({ width: 390, height: 844 });
  const mana = await page.locator(".combat-resource").nth(0).boundingBox();
  const shells = await page.locator(".combat-resource").nth(1).boundingBox();
  expect(mana && shells && mana.y + mana.height <= shells.y).toBe(true);
  await expect(page.getByRole("button", { name: "턴 종료", exact: true })).toBeInViewport();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: "test-results/avenger-mobile.png", fullPage: true });
  expect(errors).toEqual([]);
});

test("캐릭터 선택 창에는 복수귀만 표시되고 선택 후 원정이 시작된다", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.getByRole("heading", { name: "원정에 나설 부대" })).toBeVisible();
  await expect(page.locator("button.character")).toHaveCount(1);
  await expect(page.locator('button.character[data-squad-id="avenger"]')).toBeVisible();
  await expect(page.getByRole("button", { name: "런 시작" })).toBeDisabled();
  await page.locator('button.character[data-squad-id="avenger"]').click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  const snapshot = await page.evaluate(async () => {
    const url="/src/wasm/chesslayer_web.js";
    const { default:init, WebGame, compendium } = await import(url); await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return { state:JSON.parse(game.snapshot()) as Snapshot, catalog:JSON.parse(compendium()) }; }
    finally {game.free();}
  });
  expect(snapshot.state.squad_id).toBe("avenger");
  expect(snapshot.state.deck_count).toBe(10);
  expect(snapshot.catalog).toHaveLength(1);
  const catalog = snapshot.catalog.find((c: {id:string}) => c.id === "avenger");
  expect(catalog.cards).toHaveLength(37);
  expect(catalog.cards.every((c: {available: boolean}) => c.available)).toBe(true);
  expect(catalog.relics).toHaveLength(15);
  expect(catalog.relics.some((r: {id:string}) => r.id === "palace_family")).toBe(true);
  expect(catalog.pieces.some((p: {key:string}) => p.key === "drone")).toBe(true);
});

test("제거된 부대의 저장 데이터는 보존하고 이어하기만 숨긴다", async ({ page }) => {
  await page.goto("/");
  const old = JSON.stringify({ version: 24, character: "guardian", seed: 7, floor: 1,
    progress: { highest_floor: 1, unlocked_squads: ["guardian"], final_unlocked: false, final_defeated: false }, commands: [] });
  await page.evaluate((saved) => localStorage.setItem("chesslayer.run.v1", saved), old);
  await page.reload();
  await expect(page.getByRole("button", { name: "저장된 원정 이어하기" })).toHaveCount(0);
  await expect(page.getByRole("status")).toContainText("저장 데이터는 보존");
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(old);
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.locator('button.character[data-squad-id="avenger"]').click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
});

test("무한 원정도 복수귀 선택 창을 거쳐 시작한다", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "무한 원정 시작" }).click();
  await expect(page.locator('button.character[data-squad-id="avenger"]')).toHaveCount(1);
  await page.locator('button.character[data-squad-id="avenger"]').click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  const mode = await page.evaluate(() => JSON.parse(localStorage.getItem("chesslayer.endless.run.v1") ?? "null")?.run);
  expect(mode).toBeTruthy();
});

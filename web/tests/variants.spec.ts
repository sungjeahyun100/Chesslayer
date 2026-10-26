import { test, expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Command, Snapshot } from "../src/types";
const saveKey = "chesslayer.run.v1"; // Storage slot is retained; the payload carries version 13.
const square = (cell: number) => `\"${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}\"`;
const saved = (page: Page) => page.evaluate((key) => localStorage.getItem(key)!, saveKey);

async function snapshot(page: Page): Promise<Snapshot> {
  return page.evaluate(async (key) => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = WebGame.restore(localStorage.getItem(key)!);
    try { return JSON.parse(game.snapshot()); } finally { game.free(); }
  }, saveKey);
}
async function resume(page: Page) {
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
}
async function clickMove(page: Page, from: number, to: number) {
  await expect(page.getByTestId("turn")).toContainText("나의 턴");
  await page.locator(`[data-square=${square(from)}]`).click();
  await expect(page.locator(`[data-square=${square(to)}]`)).toHaveAttribute("data-legal", "true");
  await page.locator(`[data-square=${square(to)}]`).click();
}
async function imagesLoaded(page: Page) {
  await expect(page.locator(".missing-piece")).toHaveCount(0);
  await expect.poll(() => page.locator("img.piece-image").evaluateAll((images) => images.every((i) => (i as HTMLImageElement).complete && (i as HTMLImageElement).naturalWidth > 0))).toBe(true);
}

for (const family of ["tempest", "bouncing"] as const) {
  test(`${family}: 정상 원정에서 분기 강화·모든 승격 선택·이동·저장·다음 전투`, async ({ page }) => {
    if (family === "bouncing") await page.setViewportSize({ width: 390, height: 844 });
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const commands: Command[] = JSON.parse(readFileSync(`../crates/chesslayer/tests/fixtures/${family}-promotion.json`, "utf8"));
    const restIndex = commands.findIndex((c) => c.type === "rest_upgrade");
    const promotionIndex = commands.findIndex((c) => c.type === "move" && c.promotion !== undefined);
    await page.goto("/");
    await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
    // Prepare only a legitimate rest save by replaying the production command API.
    await page.evaluate(async ({ commands, saveKey }) => {
      const url = "/src/wasm/chesslayer_web.js";
      const { default: init, WebGame } = await import(url); await init();
      const game = new WebGame("vanguard", 2026);
      for (const c of commands) {
        game.dispatch(JSON.stringify(c));
        let s = JSON.parse(game.snapshot());
        while (s.phase === "combat" && s.turn === 1) s = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" })));
      }
      localStorage.setItem(saveKey, game.save()); game.free();
    }, { commands: commands.slice(0, restIndex), saveKey });
    await resume(page);
    await page.locator('[data-square="a2"]').click();
    await expect(page.locator(".skill-node.available")).toHaveCount(2);
    await page.getByRole("button", { name: "템페스트 폰 · 선택 가능 · 상세 보기" }).click();
    await page.getByRole("button", { name: "바운싱 폰 · 선택 가능 · 상세 보기" }).click();
    await expect(page.locator('.skill-node[aria-pressed="true"]')).toHaveCount(1);
    if (family === "tempest") await page.getByRole("button", { name: "템페스트 폰 · 선택 가능 · 상세 보기" }).click();
    if (family === "bouncing") {
      const zoom = page.locator(".skill-map-toolbar output");
      const beforeZoom = await zoom.textContent();
      await page.getByRole("button", { name: "트리 확대", exact: true }).click();
      await expect(zoom).not.toHaveText(beforeZoom!);
      await page.getByRole("button", { name: "전체 보기", exact: true }).click();
      await expect(zoom).toHaveText(beforeZoom!);
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    }
    await imagesLoaded(page);
    await page.screenshot({ path: `test-results/${family}-tree.png`, fullPage: true, animations: "disabled" });
    await page.getByRole("button", { name: "강화 확정", exact: true }).click();
    expect(JSON.parse(await saved(page)).commands.at(-1)).toMatchObject(commands[restIndex]);
    await resume(page);
    await page.locator('[data-node-id="13"]').click();
    await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", `${family}_pawn`);
    for (const c of commands.slice(restIndex + 2, promotionIndex)) {
      if (c.type === "play_card") await page.locator(`[data-card-id="${c.id}"]`).click();
      else if (c.type === "move") await clickMove(page, c.from, c.to);
    }
    await expect(page.getByTestId("turn")).toContainText("나의 턴");
    const before = await saved(page);
    const promotion = commands[promotionIndex];
    if (promotion.type !== "move") throw new Error("missing promotion fixture");
    const candidates = (await snapshot(page)).moves.filter((m) => m.from === promotion.from && m.to === promotion.to).map((m) => m.promotion!);
    expect(candidates.length).toBe(family === "tempest" ? 4 : 3);
    for (const candidate of candidates) {
      // Every branch starts from the exact same real, unmodified command history.
      await page.evaluate(({ saveKey, before }) => localStorage.setItem(saveKey, before), { saveKey, before });
      await resume(page);
      await clickMove(page, promotion.from, promotion.to);
      await expect(page.getByRole("dialog", { name: "승격 선택" })).toBeVisible();
      await expect(page.locator(".promotion-options button")).toHaveCount(candidates.length);
      await imagesLoaded(page);
      await page.keyboard.press("Escape");
      await expect(page.getByRole("dialog", { name: "승격 선택" })).toHaveCount(0);
      expect(await saved(page)).toBe(before);
      await clickMove(page, promotion.from, promotion.to);
      await page.screenshot({ path: `test-results/${family}-promotion.png`, fullPage: true, animations: "disabled" });
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      await page.getByRole("button", { name: `${candidate.name}으로 승격`, exact: true }).click();
      await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", candidate.key);
      const after = await snapshot(page);
      expect(after.rest_options.find((p) => p.id === 9)).toMatchObject({ key: `${family}_pawn`, upgrades: [family] });
      expect(JSON.parse(await saved(page)).commands.at(-1)).toMatchObject({ ...promotion, promotion: candidate.kind });
      await resume(page);
      await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", candidate.key);
      // Queen is last: keep this branch for the fixture's win and next encounter.
      if (candidate.kind !== promotion.promotion) {
        const move = after.moves.find((m) => m.from === promotion.to);
        if (move) {
          await clickMove(page, move.from, move.to);
          expect(JSON.parse(await saved(page)).commands.some((c: Command) => c.type === "move" && c.from === move.from && c.to === move.to)).toBe(true);
        }
      }
    }
    for (const c of commands.slice(promotionIndex + 1)) {
      if (c.type === "play_card") await page.locator(`[data-card-id="${c.id}"]`).click();
      else if (c.type === "move") await clickMove(page, c.from, c.to);
      else if (c.type === "choose_reward") {
        await page.locator(".reward-cards .playing-card").nth(c.index).click();
        await page.getByRole("button", { name: /다음으로 진행/ }).click();
      }
      else if (c.type === "select_node") await page.locator(`[data-node-id="${c.id}"]`).click();
      else if (c.type === "claim_treasure") await page.getByRole("button", { name: /보물 받고 맵으로/ }).click();
    }
    await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
    await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", `${family}_pawn`);
    await resume(page);
    const final = await snapshot(page);
    expect(final.map.current).toBe(15);
    expect(final.rest_options.find((p) => p.id === 9)).toMatchObject({ key: `${family}_pawn`, cell: 40, origin: 48, upgrades: [family], dead: false });
    await imagesLoaded(page);
    expect(errors).toEqual([]);
  });
}

test("이전 저장 버전은 설명을 표시하고 원본 기록을 보존한다", async ({ page }) => {
  await page.goto("/");
  for (const version of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]) {
    const legacy = JSON.stringify({ version, character: "vanguard", seed: 2026, commands: [{ type: "rest_upgrade", id: 9, ...(version >= 2 ? { target: 8 } : {}) }] });
    await page.evaluate(({ saveKey, legacy }) => localStorage.setItem(saveKey, legacy), { saveKey, legacy });
    await resume(page);
    await expect(page.getByRole("alert")).toContainText("이전 게임 규칙으로 저장된 원정입니다");
    expect(await saved(page)).toBe(legacy);
  }
});

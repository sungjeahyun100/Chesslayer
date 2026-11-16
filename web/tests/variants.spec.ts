import { test, expect, type Page } from "@playwright/test";
import { checkpoint, dispatch, enemyTurn, snapshot as engineSnapshot, finishHillBattle } from "./helpers/uiRuns";
import { WebGame } from "../src/wasm/chesslayer_web";
import type { Snapshot } from "../src/types";
const saveKey = "chesslayer.run.v1"; // Storage slot is retained; the payload carries its own rules version.
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
    test.setTimeout(90000);
    if (family === "bouncing") await page.setViewportSize({ width: 390, height: 844 });
    const errors: string[] = [];
    page.on("pageerror", (e) => errors.push(e.message));
    const rest = checkpoint("rest");
    await page.goto("/");
    await page.evaluate(({saveKey,value}) => { localStorage.setItem(saveKey,value); localStorage.setItem("chesslayer.run.developer.v1","true"); }, {saveKey,value:rest.saved});
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
    expect(JSON.parse(await saved(page)).commands.at(-1)).toMatchObject({type:"rest_upgrade",id:9,target:family === "tempest" ? 8 : 13});
    const probe = WebGame.restore(await saved(page));
    let before: string;
    try {
      const node = engineSnapshot(probe).map.nodes.find(n => n.available && !n.visited && n.kind === "combat")!;
      dispatch(probe,{type:"select_node",id:node.id});
      for (let from = 48; from > 8; from -= 8) { dispatch(probe,{type:"move",from,to:from-8}); enemyTurn(probe); }
      const ward = engineSnapshot(probe).hand.find(c => c.definition.kind === "ward");
      if (ward) dispatch(probe,{type:"play_card",id:ward.id,target:9});
      before = probe.save();
    } finally { probe.free(); }
    await page.evaluate(({saveKey,before}) => localStorage.setItem(saveKey,before),{saveKey,before});
    await resume(page);
    const promotion = { type: "move" as const, from: 8, to: 0 };
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
      expect(JSON.parse(await saved(page)).commands).toContainEqual({ ...promotion, promotion: candidate.kind });
      await resume(page);
      await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute("data-piece-key", candidate.key);
    }
    const continuation = WebGame.restore(await saved(page));
    try {
      enemyTurn(continuation);
      finishHillBattle(continuation);
      dispatch(continuation,{type:"continue_reward"});
      const node = engineSnapshot(continuation).map.nodes.find(n => n.available && !n.visited && n.kind === "combat")!;
      dispatch(continuation,{type:"select_node",id:node.id});
      await page.evaluate(({saveKey,value}) => localStorage.setItem(saveKey,value),{saveKey,value:continuation.save()});
    } finally { continuation.free(); }
    await resume(page);
    const final = await snapshot(page);
    expect(final.phase).toBe("combat");
    expect(final.rest_options.find(p => p.id === 9)).toMatchObject({ key:`${family}_pawn`, origin:48, upgrades:[family] });
    const survivor = final.pieces.find(p => p.id === 9);
    if (survivor) expect(survivor.key).toBe(`${family}_pawn`);
    await imagesLoaded(page);
    expect(errors).toEqual([]);
  });
}

test("이전 저장 버전은 설명을 표시하고 원본 기록을 보존한다", async ({ page }) => {
  await page.goto("/");
  for (const version of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]) {
    const legacy = JSON.stringify({ version, character: "vanguard", seed: 2026, commands: [{ type: "rest_upgrade", id: 9, ...(version >= 2 ? { target: 8 } : {}) }] });
    await page.evaluate(({ saveKey, legacy }) => localStorage.setItem(saveKey, legacy), { saveKey, legacy });
    await resume(page);
    await expect(page.getByRole("alert")).toContainText("이전 게임 규칙으로 저장된 원정입니다");
    expect(await saved(page)).toBe(legacy);
  }
});

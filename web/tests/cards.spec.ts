import { test, expect } from "@playwright/test";
import { WebGame } from "../src/wasm/chesslayer_web.js";
import { checkpoint, dispatch, enemyTurn, snapshot } from "./helpers/uiRuns";

test("일시 강화: 유효한 프로모션 선택과 취소는 무소모, 카드 확인만 전투 보드에 적용", async ({ page }) => {
  const { saved, state } = checkpoint("temporary");
  const temporaryCard = state.hand.find((card) => card.definition.kind === "temporary_upgrade")!;
  const pawns = state.pieces.filter((piece) => piece.owner === 0 && piece.kind === 1 && temporaryCard.targets.includes(piece.id)).slice(0, 2);
  const path = state.upgrade_paths.find((path) => path.from === pawns[0].kind && path.unlocked)!;
  const square = (piece: typeof pawns[number]) => `[data-square="${piece.square}"]`;
  await page.goto("/");
  await page.evaluate((value) => localStorage.setItem("chesslayer.run.v1", value), saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  const card = page.locator(`.hand [data-card-id="${temporaryCard.id}"]`);
  await expect(card).toContainText("일시 강화");
  await expect(card.locator(".cost")).toHaveText("N");
  const before = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  await card.click();
  await page.locator(square(pawns[0])).click();
  const panel = page.locator(".card-upgrade-picker");
  await expect(panel).toHaveAttribute("role", "dialog");
  await expect(panel).toContainText(`일시 강화 · ${pawns[0].name} ${pawns[0].square}`);
  await expect(panel.locator(".skill-tree")).toHaveCount(0);
  const legalKinds = temporaryCard.upgrade_options.filter((option) => option.id === pawns[0].id)
    .map((option) => option.target).sort((a, b) => a - b);
  expect(await panel.locator(".promotion-options button").evaluateAll((buttons) => buttons.map((button) => Number((button as HTMLElement).dataset.kind)).sort((a, b) => a - b))).toEqual(legalKinds);
  await expect(page.locator(square(pawns[0]))).toBeDisabled();
  const boardBox = await page.locator(".board-frame").boundingBox();
  const panelBox = await panel.boundingBox();
  expect(panelBox!.width).toBeLessThanOrEqual(460);
  expect(panelBox!.x).toBeGreaterThanOrEqual(boardBox!.x);
  expect(panelBox!.x + panelBox!.width).toBeLessThanOrEqual(boardBox!.x + boardBox!.width);
  await page.keyboard.press("Escape");
  await expect(panel).toHaveCount(0);
  await expect(card).toHaveClass(/active-card/);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await page.locator(square(pawns[0])).click();
  await page.screenshot({ path: "test-results/card-upgrade-panel.png", fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect((await panel.boundingBox())!.width).toBeLessThanOrEqual(390);
  await page.screenshot({ path: "test-results/card-upgrade-mobile.png", fullPage: true });
  await page.setViewportSize({ width: 390, height: 600 });
  await expect.poll(() => panel.evaluate((el) => el.getBoundingClientRect().bottom <= innerHeight)).toBe(true);
  await page.setViewportSize({ width: 1440, height: 1000 });
  await panel.locator(`button[data-kind="${path.to}"]`).click();
  await expect(panel).toHaveCount(0);
  await expect(page.locator(".action-hint")).toContainText("선택 1개 · 비용 1");
  await expect(card.locator(".cost")).toHaveText("1");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator(square(pawns[1])).click();
  await expect(page.locator(".card-upgrade-picker")).toBeInViewport();
  await page.locator(`.card-upgrade-picker button[data-kind="${path.to}"]`).click();
  await page.setViewportSize({ width: 1440, height: 1000 });
  await expect(card.locator(".cost")).toHaveText("2");
  await expect(page.locator(".action-hint")).toContainText("선택 2개 · 비용 2");
  await expect(page.locator(`[data-piece-id="${pawns[0].id}"] img`)).toHaveAttribute("data-piece-key", pawns[0].key);
  await page.keyboard.press("Escape");
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await expect(card).toBeVisible();
  await card.click();
  await page.locator(square(pawns[0])).click();
  await page.locator(`.card-upgrade-picker button[data-kind="${path.to}"]`).click();
  await page.locator(".action-hint .primary").click();
  await expect(page.locator(`[data-piece-id="${pawns[0].id}"] img`)).toHaveAttribute("data-piece-key", path.to_key);
  await expect(page.locator(`[data-piece-id="${pawns[0].id}"] .temporary-mark`)).toBeVisible();
  await expect(page.locator(".mana-orb")).toHaveText(String(state.mana - 1));
  await expect(card).toHaveCount(0);
  const rosterKind = await page.evaluate(async (pieceId) => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()).rest_options.find((p: { id: number }) => p.id === pieceId).kind; }
    finally { game.free(); }
  }, pawns[0].id);
  expect(rosterKind).toBe(pawns[0].kind);
});

test("일시 강화: 바운싱 퀸과 구행·텐쿠 조건으로 행마의 신을 적용한다", async ({ page }) => {
  const { saved } = checkpoint("temporary");
  const game = WebGame.restore(saved);
  game.set_developer_mode(true);
  const original = snapshot(game);
  const queen = original.pieces.find((piece) => piece.owner === 0 && piece.key === "queen")!;
  const rook = original.pieces.find((piece) => piece.owner === 0 && piece.key === "rook")!;
  const bishop = original.pieces.find((piece) => piece.owner === 0 && piece.key === "bishop")!;
  const kind = (key: string) => original.upgrade_paths.find((path) => path.to_key === key)!.to;
  dispatch(game, { type: "developer_set_upgrade", id: queen.id, target: kind("bouncing_queen") });
  dispatch(game, { type: "developer_set_upgrade", id: rook.id, target: kind("hook") });
  dispatch(game, { type: "developer_set_upgrade", id: bishop.id, target: kind("tengu") });
  const state = snapshot(game);
  const composite = state.upgrade_paths.find((path) => path.from === kind("bouncing_queen") && path.to_key === "movement_god")!;
  expect(composite.unlocked).toBe(true);
  const card = state.hand.find((card) => card.definition.kind === "temporary_upgrade")!;
  expect(card.targets).toContain(queen.id);
  const prepared = game.save();
  game.free();

  await page.goto("/");
  await page.evaluate((value) => {
    localStorage.setItem("chesslayer.run.v1", value);
    localStorage.setItem("chesslayer.developer_mode.v1", "true");
    localStorage.setItem("chesslayer.run.developer.v1", "true");
  }, prepared);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator(`.hand [data-card-id="${card.id}"]`).click();
  await page.locator(`[data-square="${queen.square}"]`).click();
  const picker = page.locator(".card-upgrade-picker");
  await expect(picker.locator(`button[data-kind="${composite.to}"]`)).toBeVisible();
  await picker.locator(`button[data-kind="${composite.to}"]`).click();
  await page.locator(".action-hint .primary").click();
  await expect(page.locator(`[data-piece-id="${queen.id}"] img`)).toHaveAttribute("data-piece-key", composite.to_key);
  await expect(page.locator(`[data-piece-id="${queen.id}"] .temporary-mark`)).toBeVisible();
});

test("일시 강화: 전투 중에만 얻은 구행·텐쿠도 행마의 신 조건에 포함한다", async ({ page }) => {
  const { saved } = checkpoint("temporary");
  const game = WebGame.restore(saved);
  game.set_developer_mode(true);
  const original = snapshot(game);
  const queen = original.pieces.find((piece) => piece.owner === 0 && piece.key === "queen")!;
  const rook = original.pieces.find((piece) => piece.owner === 0 && piece.key === "rook")!;
  const bishop = original.pieces.find((piece) => piece.owner === 0 && piece.key === "bishop")!;
  const kind = (key: string) => original.upgrade_paths.find((path) => path.to_key === key)!.to;
  dispatch(game, { type: "developer_set_upgrade", id: queen.id, target: kind("bouncing_queen") });
  dispatch(game, { type: "developer_set_upgrade", id: rook.id, target: kind("bouncing_rook") });
  dispatch(game, { type: "developer_set_upgrade", id: bishop.id, target: kind("bouncing_bishop") });
  const card = snapshot(game).hand.find((card) => card.definition.kind === "temporary_upgrade")!;
  dispatch(game, { type: "play_card_selection", id: card.id, targets: [], upgrades: [
    { id: rook.id, target: kind("hook") }, { id: bishop.id, target: kind("tengu") },
  ] });
  expect(snapshot(game).rest_options.find((piece) => piece.id === rook.id)?.kind).toBe(kind("bouncing_rook"));
  expect(snapshot(game).rest_options.find((piece) => piece.id === bishop.id)?.kind).toBe(kind("bouncing_bishop"));
  for (let turn = 0; turn < 8 && !snapshot(game).hand.some((item) => item.id === card.id); turn++) {
    dispatch(game, { type: "end_turn" });
    enemyTurn(game);
  }
  const state = snapshot(game);
  expect(state.phase).toBe("combat");
  expect(state.hand.some((item) => item.id === card.id)).toBe(true);
  expect(state.pieces.find((piece) => piece.id === rook.id)?.kind).toBe(kind("hook"));
  expect(state.pieces.find((piece) => piece.id === bishop.id)?.kind).toBe(kind("tengu"));
  const composite = state.upgrade_paths.find((path) => path.from === kind("bouncing_queen") && path.to_key === "movement_god")!;
  expect(composite.unlocked).toBe(false);
  expect(state.hand.find((item) => item.id === card.id)?.upgrade_options).toContainEqual({ id: queen.id, target: composite.to });
  const prepared = game.save();
  game.free();

  await page.goto("/");
  await page.evaluate((value) => {
    localStorage.setItem("chesslayer.run.v1", value);
    localStorage.setItem("chesslayer.developer_mode.v1", "true");
    localStorage.setItem("chesslayer.run.developer.v1", "true");
  }, prepared);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator(`.hand [data-card-id="${card.id}"]`).click();
  await page.locator(`[data-square="${queen.square}"]`).click();
  const picker = page.locator(".card-upgrade-picker");
  await expect(picker.locator(`button[data-kind="${composite.to}"]`)).toBeVisible();
  await picker.locator(`button[data-kind="${composite.to}"]`).click();
  await page.locator(".action-hint .primary").click();
  await expect(page.locator(`[data-piece-id="${queen.id}"] img`)).toHaveAttribute("data-piece-key", composite.to_key);
});

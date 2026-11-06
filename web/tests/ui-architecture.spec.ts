import { expect, test, type Page } from "@playwright/test";
import type { Command, Snapshot } from "../src/types";
import { checkpoint, dispatch, enemyTurn, newRun, snapshot } from "./helpers/uiRuns";

const squareName = (cell: number) => `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;
const cell = (page: Page, at: number) => page.locator(`.board [data-square="${squareName(at)}"]`);
const saved = (page: Page) => page.evaluate(() => localStorage.getItem("chesslayer.run.v1")!);
const commands = async (page: Page): Promise<Command[]> => JSON.parse(await saved(page)).commands;
async function resume(page: Page, value: string) {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.evaluate((saved) => {
    localStorage.setItem("chesslayer.run.v1", saved);
    localStorage.setItem("chesslayer.developer_mode.v1", "true");
    localStorage.setItem("chesslayer.run.developer.v1", "true");
  }, value);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
}
async function restored(page: Page): Promise<Snapshot> {
  return page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()); } finally { game.free(); }
  });
}

test("multiple-card selection, switching cards and status overlay never leak or commit targets", async ({ page }) => {
  const game = newRun("artillery");
  dispatch(game, { type: "select_node", id: 0 });
  dispatch(game, { type: "end_turn" });
  enemyTurn(game);
  const state = snapshot(game);
  const value = game.save();
  game.free();
  await resume(page, value);
  const cards = state.hand.filter((card) => card.definition.selection_mode === "multiple" && !card.disabled_reason);
  expect(cards.length).toBeGreaterThanOrEqual(2);
  const ally = state.pieces.find((piece) => piece.owner === 0 && cards[0].targets.includes(piece.id))!;
  const choose = (id: number) => page.locator(`.hand [data-card-id="${id}"]`).click();
  await choose(cards[0].id);
  await cell(page, ally.cell).click();
  await expect(cell(page, ally.cell)).toHaveClass(/card-staged/);
  await cell(page, ally.cell).click();
  await expect(page.locator(".card-staged")).toHaveCount(0);
  await cell(page, ally.cell).click();
  await choose(cards[1].id);
  await expect(page.locator(".card-staged")).toHaveCount(0);
  await cell(page, ally.cell).click();
  await page.getByRole("button", { name: "원정 현황", exact: true }).click();
  await expect(cell(page, ally.cell)).toBeDisabled();
  await expect(page.locator(".hand .playing-card:enabled")).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog", { name: "원정 현황" })).toHaveCount(0);
  await expect(cell(page, ally.cell)).toHaveClass(/card-staged/);
  await page.keyboard.press("Escape");
  await expect(page.locator(".card-staged")).toHaveCount(0);
  await expect(page.locator(".active-card")).toHaveCount(0);
  expect(await saved(page)).toBe(value);
});

test("special-ability cancellation, enemy inspection and card selection stay exclusive", async ({ page }) => {
  const game = newRun("heretic");
  dispatch(game, { type: "select_node", id: 0 });
  const state = snapshot(game), value = game.save();
  game.free();
  await resume(page, value);
  const ability = state.abilities.find((ability) => ability.targets.length)!;
  const actor = state.pieces.find((piece) => piece.id === ability.piece_id)!;
  await cell(page, actor.cell).click();
  await page.locator(".ability-button").first().click();
  await expect(page.locator(".action-hint")).toContainText(ability.name);
  await cell(page, ability.targets[0].cell).hover();
  await page.locator(".action-hint .text-button").click();
  await expect(page.locator(".aim-target")).toHaveCount(0);
  await expect(page.locator(".blast-preview")).toHaveCount(0);
  const enemy = state.pieces.find((piece) => piece.owner === 1)!;
  await cell(page, enemy.cell).click();
  await expect(page.locator(".enemy-piece-info")).toBeVisible();
  await cell(page, actor.cell).click();
  await expect(page.locator(".enemy-piece-info")).toHaveCount(0);
  await page.locator(".ability-button").first().click();
  await page.locator(".hand .playing-card:enabled").first().click();
  await expect(page.locator(".aim-target")).toHaveCount(0);
  await expect(page.locator(".ability-button.selected")).toHaveCount(0);
  await page.keyboard.press("Escape");
  expect(await saved(page)).toBe(value);
});

test("temporary upgrade staging survives focus changes and only confirmation spends resources", async ({ page }) => {
  const { saved: value, state } = checkpoint("temporary");
  await resume(page, value);
  const card = state.hand.find((card) => card.definition.kind === "temporary_upgrade")!;
  const pawns = state.pieces.filter((piece) => piece.owner === 0 && piece.kind === 1 && card.targets.includes(piece.id)).slice(0, 2);
  const path = state.upgrade_paths.find((path) => path.from === 1 && path.unlocked)!;
  const handCard = page.locator(`.hand [data-card-id="${card.id}"]`);
  const stage = async (at: number) => {
    await cell(page, at).click();
    await page.locator(`.card-upgrade-picker button[data-kind="${path.to}"]`).click();
    await page.locator(".card-upgrade-confirm").click();
  };
  await handCard.click();
  await stage(pawns[0].cell);
  await cell(page, pawns[1].cell).click();
  await page.locator(".card-upgrade-cancel").click();
  await expect(page.locator(".card-staged")).toHaveCount(1);
  await page.setViewportSize({ width: 390, height: 844 });
  await stage(pawns[1].cell);
  await expect(handCard.locator(".cost")).toHaveText("2");
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(await saved(page)).toBe(value);
  await page.keyboard.press("Escape");
  await expect(page.locator(".card-staged")).toHaveCount(0);
  await handCard.click();
  await stage(pawns[0].cell);
  await page.locator(".action-hint .primary").click();
  const after = await restored(page);
  expect(after.pieces.find((piece) => piece.id === pawns[0].id)?.temporary_upgrade).toBe(true);
  expect(after.mana).toBe(state.mana - 1);
  expect(after.rest_options.find((piece) => piece.id === pawns[0].id)?.kind).toBe(1);
  expect((await commands(page)).at(-1)).toEqual({ type: "play_card_selection", id: card.id, targets: [], upgrades: [{ id: pawns[0].id, target: path.to }] });
});

test("promotion blocks other board input, cancels without saving and commits the chosen kind", async ({ page }) => {
  const game = newRun();
  dispatch(game, { type: "select_node", id: 1 });
  for (const [from, to] of [[48, 32], [32, 24], [24, 16], [16, 8]]) {
    dispatch(game, { type: "move", from, to });
    enemyTurn(game);
  }
  const state = snapshot(game), value = game.save();
  game.free();
  const move = state.moves.find((move) => move.from === 8 && move.to === 0 && move.promotion)!;
  await resume(page, value);
  const open = async () => { await cell(page, 8).click(); await cell(page, 0).click(); };
  await open();
  await expect(page.getByRole("dialog", { name: "승격 선택" })).toBeVisible();
  await expect(page.locator(".board .square:enabled")).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(page.locator(".promotion-picker")).toHaveCount(0);
  expect(await saved(page)).toBe(value);
  await open();
  await page.getByRole("button", { name: `${move.promotion!.name}으로 승격`, exact: true }).click();
  expect(await commands(page)).toContainEqual({ type: "move", from: 8, to: 0, promotion: move.promotion!.kind });
  await expect(page.locator(".promotion-picker")).toHaveCount(0);
});

test("rest reuses the board for preview, cancellation, permanent upgrade and save restoration", async ({ page }) => {
  const { saved: value, state } = checkpoint("rest");
  await resume(page, value);
  const pawn = state.rest_options.find((piece) => piece.kind === 1 && piece.can_upgrade && piece.cell !== null)!;
  const path = state.upgrade_paths.find((path) => path.from === pawn.kind && path.unlocked)!;
  await cell(page, pawn.cell!).click();
  await page.locator(`.rest-panel button[data-kind="${path.to}"]`).click();
  await expect(page.locator(`[data-piece-id="${pawn.id}"] img`)).toHaveAttribute("data-piece-key", path.to_key);
  await page.screenshot({ path: "test-results/refactor-rest-preview.png", fullPage: true });
  await page.keyboard.press("Escape");
  expect(await saved(page)).toBe(value);
  await expect(page.locator(`[data-piece-id="${pawn.id}"] img`)).toHaveAttribute("data-piece-key", pawn.key);
  await cell(page, pawn.cell!).click();
  await page.locator(`.rest-panel button[data-kind="${path.to}"]`).click();
  await page.getByRole("button", { name: "강화 확정", exact: true }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  const committed = await saved(page);
  expect((await commands(page)).at(-1)).toEqual({ type: "rest_upgrade", id: pawn.id, target: path.to });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  expect(await saved(page)).toBe(committed);
  expect((await restored(page)).rest_options.find((piece) => piece.id === pawn.id)?.kind).toBe(path.to);
});

test("reward upgrade returns on Escape and claims exactly one upgrade before map continuation", async ({ page }) => {
  const { saved: value, state } = checkpoint("elite");
  await resume(page, value);
  const reward = page.getByRole("dialog", { name: "전투 승리" });
  await expect(reward).toBeVisible();
  await reward.getByRole("button", { name: "기물 강화", exact: true }).click();
  const pawn = state.rest_options.find((piece) => piece.kind === 1 && piece.can_upgrade)!;
  const at = state.next_battle_pieces.find((piece) => piece.id === pawn.id)!.cell;
  const path = state.upgrade_paths.find((path) => path.from === pawn.kind && path.unlocked)!;
  await cell(page, at).click();
  await page.keyboard.press("Escape");
  await expect(reward).toBeVisible();
  expect(await saved(page)).toBe(value);
  await reward.getByRole("button", { name: "기물 강화", exact: true }).click();
  await cell(page, at).click();
  await page.locator(`.rest-panel button[data-kind="${path.to}"]`).click();
  await page.getByRole("button", { name: "강화 확정", exact: true }).click();
  await expect(reward).toBeVisible();
  await expect(reward.getByRole("button", { name: "기물 강화", exact: true })).toBeDisabled();
  expect((await commands(page)).filter((command) => command.type === "claim_upgrade")).toEqual([{ type: "claim_upgrade", id: pawn.id, target: path.to }]);
  await reward.locator(".reward-continue").click();
  await expect(reward).toHaveCount(0);
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  expect(await page.locator("#root").evaluate((root) => (root as HTMLElement).inert)).toBe(false);
});

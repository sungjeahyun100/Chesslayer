import { test, expect, type Page } from "@playwright/test";
import { WebGame } from "../src/wasm/chesslayer_web.js";
import type { Card } from "../src/types";
import { dispatch, snapshot } from "./helpers/uiRuns";

async function resume(page: Page, saved: string) {
  await page.goto("/");
  await page.evaluate((value) => localStorage.setItem("chesslayer.run.v1", value), saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
}

async function drag(page: Page, from: import("@playwright/test").Locator, to: import("@playwright/test").Locator | { x: number; y: number }) {
  const source = await from.boundingBox();
  const target = "boundingBox" in to ? await to.boundingBox() : null;
  const x = target ? target.x + target.width / 2 : "x" in to ? to.x : 0;
  const y = target ? target.y + target.height / 2 : "y" in to ? to.y : 0;
  await page.mouse.move(source!.x + source!.width / 2, source!.y + source!.height / 2);
  await page.mouse.down();
  await page.mouse.move(x, y, { steps: 8 });
  await page.mouse.up();
}

function preparedCard(predicate: (card: Card) => boolean) {
  for (let seed = 1; seed < 100; seed++) {
    const game = new WebGame("avenger", seed);
    try {
      const map = snapshot(game);
      const state = dispatch(game, { type: "select_node", id: map.map.nodes.find((node) => node.available && node.kind === "combat")!.id });
      const card = state.hand.find(predicate);
      if (card) return { saved: game.save(), state, card };
    } finally { game.free(); }
  }
  throw new Error("No suitable card in test seeds");
}

async function startSimulator(page: Page, kind: string) {
  await page.goto("/battle-simulator.html");
  await page.evaluate((cardKind) => localStorage.setItem("chesslayer.simulator.settings.v1", JSON.stringify({
    squad: "avenger", encounter: 0, deck: Array(10).fill(cardKind), relics: [], upgrades: {},
  })), kind);
  await page.reload();
  await page.getByRole("button", { name: "전투 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
}

test("card drag cancels outside, then plays a valid board drop once and rejects insufficient mana", async ({ page }) => {
  await startSimulator(page, "divinity");
  const handCard = page.locator(".hand [data-card-id]").first();
  const initialId = await handCard.getAttribute("data-card-id");
  const before = await page.locator(".hand [data-card-id]").count();
  await drag(page, handCard, { x: 2, y: 2 });
  await expect(handCard).toBeVisible();
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(before);
  const source = await handCard.boundingBox();
  const target = await page.locator(".board .square").nth(27).boundingBox();
  await page.mouse.move(source!.x + source!.width / 2, source!.y + source!.height / 2);
  await page.mouse.down();
  await page.mouse.move(target!.x + target!.width / 2, target!.y + target!.height / 2, { steps: 8 });
  await expect(handCard).toHaveClass(/drag-untargeted/);
  await expect(page.locator(".card-drag-arrow")).toHaveCount(0);
  expect(await handCard.evaluate((node) => (node as HTMLElement).style.transform)).toContain("translate(");
  await page.mouse.up();
  await expect(page.locator(`.hand [data-card-id="${initialId}"]`)).toHaveCount(0);
  await expect(page.locator(".combat-resource").first()).toContainText("마나 1");
  const unavailable = page.locator(".hand [data-unavailable]").first();
  await drag(page, unavailable, page.locator(".board .square").nth(27));
  await expect(unavailable).toBeVisible();
  await expect(page.locator(".combat-resource").first()).toContainText("마나 1");
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(before - 1);
});

test("a shape restricted card cannot be spent by dropping it on the board", async ({ page }) => {
  await startSimulator(page, "castling");
  const card = page.locator(".hand [data-unavailable]").first();
  await expect(card).toHaveAttribute("aria-disabled", "true");
  const count = await page.locator(".hand [data-card-id]").count();
  await drag(page, card, page.locator(".board .square").nth(27));
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(count);
  await expect(page.locator(".combat-resource").first()).toContainText("마나 3");
});

test("targeted card drag uses engine targets; escape restores the card", async ({ page }) => {
  const { saved, state, card } = preparedCard((entry) => entry.definition.targeted && entry.definition.selection_mode === "single"
    && !["limiter_release", "sacred_domain", "rename"].includes(entry.definition.kind) && !entry.disabled_reason && entry.targets.length > 0);
  const target = state.pieces.find((piece) => card.targets.includes(piece.id));
  const cell = target?.cell ?? card.targets.find((id) => id >= 10000)! - 10000;
  const invalid = Array.from({ length: 64 }, (_, i) => i).find((i) => {
    const piece = state.pieces.find((entry) => entry.cell === i);
    return !card.targets.includes(piece?.id ?? 10000 + i);
  });
  await resume(page, saved);
  const handCard = page.locator(`.hand [data-card-id="${card.id}"]`);
  const before = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  if (invalid !== undefined) {
    await drag(page, handCard, page.locator(".board .square").nth(invalid));
    await expect(handCard).toBeVisible();
    expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  }
  const cardBox = await handCard.boundingBox();
  const squareBox = await page.locator(".board .square").nth(cell).boundingBox();
  await page.mouse.move(cardBox!.x + cardBox!.width / 2, cardBox!.y + cardBox!.height / 2);
  await page.mouse.down();
  await page.mouse.move(squareBox!.x + squareBox!.width / 2, squareBox!.y + squareBox!.height / 2, { steps: 8 });
  await expect(page.locator(".card-drag-arrow")).toBeVisible();
  await expect(handCard).toHaveClass(/drag-targeted/);
  await expect(page.locator(".board .square").nth(cell)).toHaveAttribute("data-drop", "valid");
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await page.keyboard.press("Escape");
  await page.mouse.up();
  await expect(handCard).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await drag(page, handCard, page.locator(".board .square").nth(cell));
  await expect(handCard).toHaveCount(0);
});

test("piece drag shows legal cells, cancels invalid drop, and executes one legal move", async ({ page }) => {
  const game = new WebGame("avenger", 1);
  const map = snapshot(game);
  let state = dispatch(game, { type: "select_node", id: map.map.nodes.find((node) => node.available && node.kind === "combat")!.id });
  const order = state.hand.find((card) => card.definition.kind === "pawn_order")!;
  state = dispatch(game, { type: "play_card", id: order.id, target: order.targets[0] });
  const saved = game.save();
  game.free();
  const move = state.moves.find((entry) => !entry.promotion && state.pieces.some((piece) => piece.owner === 0 && piece.cell === entry.from))!;
  await resume(page, saved);
  const origin = page.locator(".board .square").nth(move.from);
  const destination = page.locator(".board .square").nth(move.to);
  const before = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  await drag(page, origin, { x: 2, y: 2 });
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await expect(page.locator(".board-drag-visual.returning")).toHaveCount(0);
  const sourceBox = await origin.boundingBox();
  await page.mouse.move(sourceBox!.x + sourceBox!.width / 2, sourceBox!.y + sourceBox!.height / 2);
  await page.mouse.down();
  await page.mouse.move(sourceBox!.x + sourceBox!.width / 2 + 16, sourceBox!.y + sourceBox!.height / 2, { steps: 3 });
  await expect(page.locator(".board-drag-visual.piece:not(.returning)")).toBeVisible();
  await expect(page.locator(".piece.drag-ghost")).toHaveCount(1);
  await expect(destination).toHaveAttribute("data-legal", "true");
  await page.mouse.move(sourceBox!.x + sourceBox!.width / 2, sourceBox!.y + sourceBox!.height / 2, { steps: 3 });
  await page.mouse.up();
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await drag(page, origin, destination);
  await expect(page.locator(`.piece.owner-0[data-cell="${move.to}"]`)).toHaveCount(1);
  await expect(page.locator(".combat-moving-piece")).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).not.toBe(before);
});

test("expedition debugger shares the card drag controls", async ({ page }) => {
  await page.goto("/expedition-debugger.html");
  await page.getByRole("button", { name: "설정으로 원정 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  const card = page.locator(".hand .playing-card:not([data-unavailable])").first();
  const id = await card.getAttribute("data-card-id");
  const before = await page.locator(".hand [data-card-id]").count();
  await drag(page, card, { x: 2, y: 2 });
  await expect(page.locator(".hand [data-card-id]")).toHaveCount(before);
  const validSquare = page.locator(".board .square.targetable").first();
  await card.click();
  await expect(validSquare).toBeVisible();
  await drag(page, card, validSquare);
  await expect(page.locator(`.hand [data-card-id="${id}"]`)).toHaveCount(0);
});

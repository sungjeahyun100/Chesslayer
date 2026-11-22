import { expect, test } from "@playwright/test";

test("sandbox keeps all pieces and their upgrade paths available", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "기물 테스트장" }).click();
  const board = page.locator(".sandbox-v2-board");
  const pieces = page.locator(".sandbox-v2-starters button");
  await expect(pieces.filter({ hasText: "드론" }).first()).toBeVisible();
  await expect(pieces.filter({ hasText: "저격수" })).toBeVisible();
  await pieces.filter({ hasText: /^폰$/ }).click();
  await board.locator('[data-square="d4"]').click();
  await expect(board.locator('[data-square="d4"] img')).toHaveAttribute("data-piece-key", "pawn");
  await page.locator(".sandbox-v2-tree button").filter({ hasText: "템페스트 폰" }).click();
  await expect(board.locator('[data-square="d4"] img')).toHaveAttribute("data-piece-key", "tempest_pawn");
  await expect(board.locator('[data-square="c4"]')).toHaveAttribute("data-legal", "true");
  await board.locator('[data-square="c4"]').click();
  await expect(board.locator('[data-square="c4"] img')).toHaveAttribute("data-piece-key", "tempest_pawn");
  await page.getByRole("button", { name: "보드 초기화" }).click();
  await expect(board.locator("img")).toHaveCount(0);
});

test("sandbox relic controls remain available for the Avenger", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "기물 테스트장" }).click();
  const relic = page.locator(".sandbox-v2-cards label").filter({ hasText: "호신용품" });
  await expect(relic).toBeVisible();
  await relic.locator("input").check();
  await expect(relic.locator("input")).toBeChecked();
});

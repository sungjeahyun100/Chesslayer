import { expect, test } from "@playwright/test";

test("drone and tank branches work in the sandbox with self-destruct and fallout", async ({ page }) => {
  test.setTimeout(30000);
  await page.goto("/");
  await page.getByRole("button", { name: "기물 테스트장" }).click();
  await page.locator(".sandbox-v2-pieces .sandbox-mini-tabs").getByRole("button", { name: "포병대대" }).click();
  const board = page.locator(".sandbox-v2-board");
  const starters = page.locator(".sandbox-v2-starters button");
  const tree = page.locator(".sandbox-v2-tree");
  await starters.filter({ hasText: "수류탄 드론" }).click();
  for (const name of ["C4 드론", "기관총 드론", "전술핵 드론", "유탄발사기 드론"]) {
    await expect(tree).toContainText(name);
  }
  await board.locator('[data-square="d4"]').click();
  await tree.getByRole("button", { name: /전술핵 드론/ }).click();
  await expect(board.locator('[data-square="d4"] img')).toHaveAttribute("data-piece-key", "tactical_nuke_drone");
  await starters.filter({ hasText: "전차" }).first().click();
  for (const name of ["고폭탄 전차", "열화우라늄날개안정분리철갑탄 전차", "쌍열포", "파편 첨가물"]) {
    await expect(tree).toContainText(name);
  }
  await page.getByRole("button", { name: "흑 배치" }).click();
  await board.locator('[data-square="e4"]').click();
  await starters.filter({ hasText: "수류탄 드론" }).click();
  await board.locator('[data-square="d4"]').click();
  await page.getByRole("button", { name: /전술핵 자폭 · 범위 보기/ }).click();
  await expect(board.locator(".blast-preview")).toHaveCount(17);
  await board.locator('[data-square="d4"]').click();
  await expect(board.locator('[data-square="d4"] img')).toHaveCount(0);
  await expect(board.locator('[data-square="e4"] img')).toHaveCount(0);
  await expect(board.locator(".fallout-cell")).toHaveCount(17);
  await expect(board.locator('[data-square="d4"] .fallout-badge')).toContainText("2");
  await page.getByRole("button", { name: "효과 진행" }).click();
  await expect(board.locator('[data-square="d4"] .fallout-badge')).toContainText("1");
  await page.getByRole("button", { name: "효과 진행" }).click();
  await expect(board.locator(".fallout-cell")).toHaveCount(0);
});

test("machine gun offers separate shots and self-destruct; fragment paths are distinct", async ({ page }) => {
  test.setTimeout(30000);
  await page.goto("/");
  await page.getByRole("button", { name: "기물 테스트장" }).click();
  await page.locator(".sandbox-v2-pieces .sandbox-mini-tabs").getByRole("button", { name: "포병대대" }).click();
  const board = page.locator(".sandbox-v2-board");
  const starters = page.locator(".sandbox-v2-starters button");
  const tree = page.locator(".sandbox-v2-tree");
  await starters.filter({ hasText: "수류탄 드론" }).click();
  await board.locator('[data-square="d4"]').click();
  await tree.getByRole("button", { name: /기관총 드론/ }).click();
  await starters.filter({ hasText: "전차" }).first().click();
  await page.getByRole("button", { name: "흑 배치" }).click();
  await board.locator('[data-square="d2"]').click();
  await starters.filter({ hasText: "수류탄 드론" }).click();
  await board.locator('[data-square="d4"]').click();
  await expect(page.getByRole("button", { name: /특수 능력.*사격/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /십자포화 자폭 · 범위 보기/ })).toBeVisible();
  await page.getByRole("button", { name: /특수 능력.*사격/ }).click();
  await expect(board.locator('[data-square="d2"]')).toHaveAttribute("data-legal", "true");
  await page.getByRole("button", { name: "보드 초기화" }).click();

  await starters.filter({ hasText: "전차" }).first().click();
  await page.getByRole("button", { name: "백 배치" }).click();
  await board.locator('[data-square="d4"]').click();
  await tree.getByRole("button", { name: /파편 첨가물/ }).click();
  await starters.filter({ hasText: "수류탄 드론" }).click();
  await page.getByRole("button", { name: "흑 배치" }).click();
  await board.locator('[data-square="d2"]').click();
  await starters.filter({ hasText: "전차" }).first().click();
  await board.locator('[data-square="d4"]').click();
  await page.getByRole("button", { name: /특수 능력.*사격/ }).click();
  await board.locator('[data-square="d2"]').hover();
  await expect(board.locator(".pierce-path")).toHaveCount(3);
  for (const branch of [0, 1, 2]) await expect(board.locator(`.path-${branch}`)).toHaveCount(1);
});

test("tank can target an enemy on the far board edge", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "기물 테스트장" }).click();
  await page.locator(".sandbox-v2-pieces .sandbox-mini-tabs").getByRole("button", { name: "포병대대" }).click();
  const board = page.locator(".sandbox-v2-board");
  const starters = page.locator(".sandbox-v2-starters button");
  await starters.filter({ hasText: "전차" }).first().click();
  await board.locator('[data-square="d1"]').click();
  await starters.filter({ hasText: "수류탄 드론" }).click();
  await page.getByRole("button", { name: "흑 배치" }).click();
  await board.locator('[data-square="d8"]').click();
  await board.locator('[data-square="d1"]').click();
  await page.getByRole("button", { name: /특수 능력.*사격/ }).click();
  await expect(board.locator('[data-square="d8"]')).toHaveAttribute("data-legal", "true");
});

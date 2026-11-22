import { expect, test } from "@playwright/test";

test("piece tabs, search, and rarity order", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: /도감 보기/ }).click();
  await page.getByRole("navigation", { name: "도감 종류" }).getByRole("button", { name: /기물/ }).click();
  const list = page.getByRole("navigation", { name: "기물 선택" });
  await expect(page.getByRole("button", { name: "플레이어블 기물" })).toHaveAttribute("aria-pressed", "true");
  await expect(list.getByRole("button", { name: /창고/ })).toHaveCount(0);
  await expect(list.getByRole("button", { name: /제왕/ })).toHaveCount(0);
  await expect(list.getByRole("button", { name: /부족의 독재자/ })).toHaveCount(0);
  await page.getByRole("button", { name: "몬스터 전용 기물" }).click();
  await expect(list.getByRole("button", { name: /창고/ })).toBeVisible();
  await expect(list.getByRole("button", { name: /제왕/ })).toBeVisible();
  await expect(list.getByRole("button", { name: /부족의 독재자/ })).toBeVisible();
  await page.getByRole("searchbox", { name: "기물 찾기" }).fill("창고");
  await expect(list.getByRole("button")).toHaveCount(1);
  await page.getByRole("navigation", { name: "도감 종류" }).getByRole("button", { name: /유물/ }).click();
  const rarities = await page.locator(".compendium-relic").evaluateAll((nodes) => nodes.map((node) => node.getAttribute("data-rarity")));
  const order = ["common", "rare", "epic", "legendary"];
  expect(rarities.map((rarity) => order.indexOf(rarity || "common"))).toEqual(rarities.map((rarity) => order.indexOf(rarity || "common")).sort((a, b) => a - b));
});

test("right click opens a separate piece detail without moving", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.locator('button.character[data-squad-id="avenger"]').click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await page.locator('.map-node:enabled[data-kind="combat"]').first().click();
  const piece = page.locator(".piece.owner-0").first();
  const cell = await piece.getAttribute("data-cell");
  const bounds = await piece.boundingBox();
  expect(bounds).not.toBeNull();
  await page.mouse.click(bounds!.x + bounds!.width / 2, bounds!.y + bounds!.height / 2, { button: "right" });
  const popup = page.getByRole("dialog", { name: /상세정보/ });
  await expect(popup).toBeVisible();
  await expect(popup.getByRole("heading", { name: "행마법" })).toBeVisible();
  await expect(popup.getByRole("heading", { name: "특수능력" })).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(popup).toHaveCount(0);
  await expect(piece).toHaveAttribute("data-cell", cell!);
});

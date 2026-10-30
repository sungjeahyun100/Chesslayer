import { test, expect } from "@playwright/test";

test("최종 보스는 HP·탄약·다음 행동과 위험 범위를 전투 시작부터 공개한다", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await page.locator("button.final-destination").click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  await page.locator('.map-node[data-kind="boss"]').click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  const panel = page.getByRole("region", { name: "부족의 독재자 상태와 행동 예고" });
  await expect(panel).toContainText("HP 5");
  await expect(panel).toContainText("샷건 2/2");
  await expect(panel).toContainText("독재자 행동 예고:");
  await expect(panel).not.toContainText("호위 행동 예고:");
  await expect(page.locator(".piece-layer .piece.owner-1")).toHaveCount(1);
  await expect(page.locator('.piece-layer .piece.owner-1 img[data-piece-key="tribal_dictator"]')).toBeVisible();
  await expect(page.locator(".board-caption")).toHaveCount(0);
  await expect(page.locator(".square.boss-risk")).not.toHaveCount(0);
  await expect(page.getByRole("button", { name: "턴 종료" })).toHaveCount(0);
});

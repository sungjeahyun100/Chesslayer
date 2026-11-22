import { expect, test } from "@playwright/test";

test("compendium shows a separate shooting board only for shooting pieces", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: /도감 보기/ }).click();
  await page.getByRole("navigation", { name: "도감 종류" }).getByRole("button", { name: /기물/ }).click();
  await expect(page.locator(".compendium-shooting-board")).toHaveCount(0);

  await page.getByRole("navigation", { name: "기물 선택" }).getByRole("button", { name: /소총수/ }).click();
  const shooting = page.locator(".compendium-shooting-board");
  await expect(shooting).toBeVisible();
  await expect(shooting.locator(".square").nth(27)).toContainText("95%");
  await expect(shooting.locator(".square").nth(19)).toContainText("90%");
  await expect(shooting.locator(".square").nth(43)).not.toHaveClass(/shot-range/);

  await page.getByRole("navigation", { name: "기물 선택" }).getByRole("button", { name: /박격포병/ }).click();
  await expect(shooting.locator(".square").nth(27)).not.toHaveClass(/shot-range/);
  await expect(shooting.locator(".square").nth(19)).toContainText("80%");
});

test("compendium keeps monster-only pieces visible and labeled", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: /도감 보기/ }).click();
  await page.getByRole("navigation", { name: "도감 종류" }).getByRole("button", { name: /기물/ }).click();
  for (const name of ["선봉대장", "말을 탄 선봉대장", "선봉 부대장", "말을 탄 선봉 부대장", "정찰병", "유령 정찰병", "창고"]) {
    await page.locator(".compendium-piece-list button").filter({ has: page.locator("strong").getByText(name, { exact: true }) }).click({ timeout: 10_000 });
    await expect(page.locator(".compendium-piece-intro .eyebrow")).toHaveText("몬스터 전용");
  }
});

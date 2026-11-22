import { expect, test } from "@playwright/test";

test("v1 developer upgrade tree shows the merged queen branch and can undo it", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await page.locator('button.character[data-squad-id="avenger"]').click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await page.getByRole("button", { name: "원정 현황", exact: true }).click();
  await page.getByRole("button", { name: /아군 퀸$/ }).click();
  const tree = page.getByRole("region", { name: /강화 경로 지도/ });
  for (const name of ["가면의 귀부인", "박격포병", "특수부대 지휘관"]) {
    await expect(tree.getByRole("button", { name: new RegExp(`${name} · 즉시 변경`) })).toBeAttached();
  }
  await tree.getByRole("button", { name: /가면의 귀부인 · 즉시 변경/ }).click();
  await expect(page.getByRole("button", { name: /아군 가면의 귀부인$/ })).toBeVisible();
  await tree.getByRole("button", { name: /붉은 귀부인 · 즉시 변경/ }).click();
  await expect(page.getByRole("button", { name: /아군 붉은 귀부인$/ })).toBeVisible();
  await tree.locator('.skill-node[data-kind="5"]').click();
  await expect(page.getByRole("button", { name: /아군 퀸$/ })).toBeVisible();
  const saved = await page.evaluate(() => JSON.parse(localStorage.getItem("chesslayer.run.v1")!));
  expect(saved.commands.filter((command: { type: string }) => command.type === "developer_set_upgrade")).toHaveLength(3);
});

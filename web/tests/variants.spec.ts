import { test, expect } from "@playwright/test";

const saveKey = "chesslayer.run.v1";

test("이전 저장 버전은 설명을 표시하고 원본 기록을 보존한다", async ({ page }) => {
  await page.goto("/");
  for (const version of [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]) {
    const legacy = JSON.stringify({
      version,
      squad: "guardian",
      seed: 2026,
      commands: [{ type: "rest_upgrade", id: 9, target: 8 }],
    });
    await page.evaluate(({ saveKey, legacy }) => localStorage.setItem(saveKey, legacy), { saveKey, legacy });
    await page.reload();
    await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
    await expect(page.getByRole("alert")).toContainText("이전 게임 규칙으로 저장된 원정입니다");
    expect(await page.evaluate((key) => localStorage.getItem(key), saveKey)).toBe(legacy);
  }
});

import { expect, test } from "@playwright/test";

const progressKey = "chesslayer.progress.v1";
const modeKey = "chesslayer.developer_mode.v1";

test("developer mode toggles unlocks without changing earned progress", async ({ page }) => {
  const earned = {
    highest_floor: 2,
    unlocked_squads: ["guardian", "vanguard"],
    final_unlocked: false,
    final_defeated: false,
  };
  await page.goto("/");
  await page.evaluate(([key, value]) => localStorage.setItem(key, value), [progressKey, JSON.stringify(earned)]);
  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.locator(".character").nth(1)).toBeEnabled();
  await expect(page.locator(".character").nth(4)).toBeDisabled();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await expect(page.getByRole("button", { name: "개발자 모드 끄기" })).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator(".character").nth(4)).toBeEnabled();
  await page.locator(".character").nth(4).click();
  await page.getByRole("button", { name: /최종 결전 · 부족의 독재자/ }).click();
  await expect(page.locator(".final-destination")).toHaveAttribute("aria-pressed", "true");
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), progressKey)).toEqual(earned);

  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.getByRole("button", { name: "개발자 모드 끄기" })).toBeVisible();
  await page.getByRole("button", { name: "개발자 모드 끄기" }).click();
  await expect(page.locator(".character").nth(1)).toBeEnabled();
  await expect(page.locator(".character").nth(4)).toBeDisabled();
  await expect(page.locator(".final-destination")).toHaveCount(0);
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), progressKey)).toEqual(earned);
  expect(await page.evaluate((key) => localStorage.getItem(key), modeKey)).toBe("false");
});

test("older full unlock records are preserved without assuming developer mode", async ({ page }) => {
  const oldUnlock = {
    highest_floor: 4,
    unlocked_squads: ["guardian", "vanguard", "heretic", "artillery", "special"],
    final_unlocked: true,
    final_defeated: false,
  };
  await page.goto("/");
  await page.evaluate(([key, value]) => {
    localStorage.setItem(key, value);
    localStorage.removeItem("chesslayer.progress.separated.v1");
  }, [progressKey, JSON.stringify(oldUnlock)]);
  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.getByRole("button", { name: "개발자 모드 켜기" })).toBeVisible();
  await expect(page.locator(".character").nth(4)).toBeEnabled();
  await expect(page.locator(".floor-selection")).toHaveCount(0);
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), progressKey)).toEqual(oldUnlock);
});

test("developer mode off keeps earned squads available", async ({ page }) => {
  const oldUnlock = {
    highest_floor: 4,
    unlocked_squads: ["guardian", "vanguard", "heretic", "artillery", "special"],
    final_unlocked: true,
    final_defeated: false,
  };
  await page.goto("/");
  await page.evaluate(([key, mode, value]) => {
    localStorage.setItem(key, value);
    localStorage.setItem(mode, "false");
    localStorage.removeItem("chesslayer.progress.separated.v1");
  }, [progressKey, modeKey, JSON.stringify(oldUnlock)]);
  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.getByRole("button", { name: "개발자 모드 켜기" })).toBeVisible();
  await expect(page.locator(".character").nth(4)).toBeEnabled();
  await expect(page.locator(".floor-selection")).toHaveCount(0);
  await expect(page.locator(".final-destination")).toHaveCount(0);
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), progressKey)).toEqual(oldUnlock);

  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await expect(page.locator(".character").nth(4)).toBeEnabled();
  await page.getByRole("button", { name: "개발자 모드 끄기" }).click();
  await expect(page.locator(".character").nth(4)).toBeEnabled();
});

test("separated earned completion stays unlocked with developer mode off", async ({ page }) => {
  const earned = {
    highest_floor: 4,
    unlocked_squads: ["guardian", "vanguard", "heretic", "artillery", "special"],
    final_unlocked: true,
    final_defeated: true,
  };
  await page.goto("/");
  await page.evaluate(([key, mode, value]) => {
    localStorage.setItem(key, value);
    localStorage.setItem(mode, "false");
    localStorage.setItem("chesslayer.progress.separated.v1", "true");
  }, [progressKey, modeKey, JSON.stringify(earned)]);
  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.getByRole("button", { name: "개발자 모드 켜기" })).toBeVisible();
  await expect(page.locator(".character").nth(4)).toBeEnabled();
  await expect(page.locator(".final-destination")).toHaveCount(0);
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), progressKey)).toEqual(earned);
});

test("resuming a developer run does not write its unlocks into campaign progress", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await page.locator(".character").nth(4).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator(".act-map-screen")).toBeVisible();

  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 끄기" }).click();
  await expect(page.locator(".character").nth(4)).toBeDisabled();
  await page.getByRole("button", { name: "← 처음으로" }).click();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator(".act-map-screen")).toBeVisible();
  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await expect(page.locator(".character").nth(4)).toBeDisabled();
  expect(await page.evaluate((key) => localStorage.getItem(key), modeKey)).toBe("false");
});

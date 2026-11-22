import { expect, test } from "@playwright/test";
import { statusCatalog } from "../src/game/statusCatalog";
import { hungerStages, statusVisuals } from "../src/game/statusVisuals";

test("status catalog covers active battle badges without duplicate entries", () => {
  const kinds = statusCatalog.map((status) => status.kind);
  expect(new Set(kinds).size).toBe(kinds.length);
  const active = statusVisuals({ statuses: [
    "식재료", "황충의 축복", "공복", "광기 1", "죄책감 1", "민첩 1",
    "전투불능 3", "페스트 3", "시체자루 1", "원기옥", "죽음의 문턱", "다음 수 만찬의 종소리", "임시 기물",
  ], ward: true });
  for (const badge of active) {
    if (badge.kind === "hunger") continue;
    expect(statusCatalog.find((status) => status.kind === badge.kind)).toMatchObject({ name: badge.name, icon: badge.icon });
  }
  for (const [index, stage] of hungerStages.entries()) {
    const badge = statusVisuals({ statuses: [stage.label], ward: false })[0];
    expect(badge).toMatchObject({ kind: "hunger", name: stage.name, description: stage.description, count: index + 1 });
    expect(statusCatalog.find((status) => status.kind === `hunger_${index + 1}`)).toMatchObject({
      name: stage.name, icon: badge.icon, effect: stage.description,
    });
  }
  expect(kinds).not.toContain("hunger");
  for (const unused of ["weakness", "dining", "aimed", "propellant", "entrenched_pending", "entrenched_ready", "suppressed", "reckless", "limiter", "temporary_upgrade"]) {
    expect(kinds).not.toContain(unused);
  }
});

test("compendium status list opens details and filters by name", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: /도감 보기/ }).click();
  await page.getByRole("navigation", { name: "도감 종류" }).getByRole("button", { name: /상태이상/ }).click();
  const list = page.getByRole("navigation", { name: "상태이상 선택" });
  await expect(list.getByRole("button")).toHaveCount(statusCatalog.length);
  for (const stage of hungerStages) {
    await list.getByRole("button", { name: stage.name, exact: true }).click();
    await expect(page.locator(".compendium-status-detail h3")).toHaveText(stage.name);
    await expect(page.locator(".compendium-status-detail dd").first()).toHaveText(stage.description);
  }
  await page.getByRole("searchbox", { name: "상태이상 찾기" }).fill("호신");
  await expect(list.getByRole("button")).toHaveCount(1);
  await expect(page.locator(".compendium-status-detail h3")).toHaveText("호신용품");
});

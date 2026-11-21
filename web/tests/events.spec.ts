import { expect, test } from "@playwright/test";
import { newRun, dispatch, snapshot, winHillBattle } from "./helpers/uiRuns";

test("운수 좋은 날은 모바일에서 후보 중 한 장만 획득하고 결과·저장을 복원한다", async ({ page }) => {
  const { saved, before } = (() => {
    for (let seed = 0; seed < 200; seed++) {
      const game = newRun("guardian", seed);
      try {
        dispatch(game, { type: "developer_set_upgrade", id: 5, target: 94 });
        const nodes = snapshot(game).map.nodes;
        const event = nodes.find((node) => node.row === 1 && node.kind === "event"
          && nodes.some((parent) => parent.row === 0 && parent.next.includes(node.id)));
        if (!event) continue;
        const start = nodes.find((node) => node.row === 0 && node.next.includes(event.id))!;
        winHillBattle(game, start.id);
        dispatch(game, { type: "continue_reward" });
        dispatch(game, { type: "select_node", id: event.id });
        const before = snapshot(game);
        if (before.event?.kind === "lucky_day") return { saved: game.save(), before };
      } finally { game.free(); }
    }
    throw new Error("No early lucky-day event found");
  })();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await page.evaluate(value => {
    localStorage.setItem("chesslayer.run.v1", value);
    localStorage.setItem("chesslayer.run.developer.v1", "true");
  }, saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.getByRole("heading", { name: "운수 좋은 날", exact: true })).toBeVisible();
  const choiceCount = before.event!.choices.length;
  expect(choiceCount).toBe(8);
  await expect(page.locator(".event-card-grid button")).toHaveCount(choiceCount);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: "test-results/event-lucky-mobile.png", fullPage: true });
  const selected = choiceCount - 1;
  await page.locator(".event-card-grid button").nth(selected).click();
  await expect(page.locator(".event-result")).toContainText(before.event!.choices[selected].card!.name);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator(".event-result")).toContainText("획득");
  await expect(page.locator(".event-card-grid button")).toHaveCount(0);
  const after = await page.evaluate(async () => {
    const {default:init, WebGame} = await import("/src/wasm/chesslayer_web.js"); await init();
    const g = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(g.snapshot()); } finally { g.free(); }
  });
  expect(after.deck_count).toBe(before.deck_count + 1);
  expect(after.deck.at(-1).definition.kind).toBe(before.event!.choices[selected].card!.kind);
  await page.locator(".location-actions button").click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
});

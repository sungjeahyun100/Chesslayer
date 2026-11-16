import { expect, test } from "@playwright/test";
import { newRun, dispatch, snapshot, winHillBattle } from "./helpers/uiRuns";

test("운수 좋은 날은 모바일에서 15장 중 한 장만 획득하고 결과·저장을 복원한다", async ({ page }) => {
  const game = newRun("guardian", 11);
  let saved: string;
  let before: ReturnType<typeof snapshot>;
  try {
    dispatch(game, { type: "developer_set_upgrade", id: 5, target: 94 });
    const nodes = snapshot(game).map.nodes;
    const event = nodes.find((node) => node.row === 3 && node.kind === "event")!;
    const routesTo = (id: number): number[][] => nodes[id].row === 0 ? [[id]]
      : nodes.filter((parent) => parent.next.includes(id))
        .flatMap((parent) => routesTo(parent.id).map((route) => [...route, id]));
    const route = routesTo(event.id).find((ids) =>
      ids.slice(0, -1).every((id) => ["combat", "elite"].includes(nodes[id].kind)))!;
    for (const id of route.slice(0, -1)) { winHillBattle(game, id); dispatch(game, { type: "continue_reward" }); }
    dispatch(game, { type: "select_node", id: event.id });
    before = snapshot(game);
    expect(before.event?.kind).toBe("lucky_day");
    saved = game.save();
  } finally { game.free(); }
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await page.evaluate(value => {
    localStorage.setItem("chesslayer.run.v1", value);
    localStorage.setItem("chesslayer.run.developer.v1", "true");
  }, saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.getByRole("heading", { name: "운수 좋은 날", exact: true })).toBeVisible();
  await expect(page.locator(".event-card-grid button")).toHaveCount(15);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: "test-results/event-lucky-mobile.png", fullPage: true });
  await page.locator(".event-card-grid button").nth(7).click();
  await expect(page.locator(".event-result")).toContainText(before.event!.choices[7].card!.name);
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
  expect(after.deck.at(-1).definition.kind).toBe(before.event!.choices[7].card!.kind);
  await page.locator(".location-actions button").click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
});

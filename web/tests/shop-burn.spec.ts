import { test, expect } from "@playwright/test";
import { dispatch, newRun, routesTo, snapshot, travelRoute } from "./helpers/uiRuns";

test("상점 카드 소각은 덱과 골드를 즉시 갱신하고 저장 복원된다", async ({ page }) => {
  const scenario = (() => {
    for (let seed = 0; seed < 500; seed++) {
      const game = newRun("guardian", seed);
      const nodes = snapshot(game).map.nodes;
      const route = nodes.filter((node) => node.kind === "shop")
        .flatMap((node) => routesTo(nodes, node.id))
        .filter((ids) => ids.slice(0, -1).reduce((gold, id) =>
          gold + (nodes[id].kind === "elite" ? 30 : nodes[id].kind === "combat" ? 15 : 0), snapshot(game).gold) >= 100)
        .sort((a, b) => a.length - b.length)[0];
      if (route) return { game, route };
      game.free();
    }
    throw new Error("No shop route earns enough gold to burn a card");
  })();
  const { game, route } = scenario;
  let saved: string;
  let before: ReturnType<typeof snapshot>;
  try {
    const king = snapshot(game).pieces.find((piece) => piece.key === "king")!;
    dispatch(game, { type: "developer_set_upgrade", id: king.id, target: 94 });
    travelRoute(game, route.slice(0, -1), true);
    dispatch(game, { type: "select_node", id: route.at(-1)! });
    before = snapshot(game);
    saved = game.save();
  } finally { game.free(); }
  expect(before.gold).toBeGreaterThanOrEqual(100);
  await page.goto("/");
  await page.evaluate((value) => localStorage.setItem("chesslayer.run.v1", value), saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "shop");
  await expect(page.locator(".expedition-gold")).toContainText(String(before.gold));
  await page.getByRole("button", { name: /카드 소각/ }).click();
  await expect(page.locator(".shop-burn-list button")).toHaveCount(before.deck_count);
  const burned = before.deck[0];
  await page.locator(".shop-burn-list button").first().click();
  await expect(page.locator(".shop-burn-detail")).toContainText(burned.definition.name);
  await expect(page.locator(".shop-burn-detail")).toContainText("100금화");
  await page.getByRole("button", { name: "이 카드 소각하기" }).click();
  await expect(page.locator(".expedition-gold")).toContainText(String(before.gold - 100));
  await expect(page.getByRole("button", { name: /카드 소각/ })).toContainText("200금화");
  await expect(page.locator(".shop-burn-list button")).toHaveCount(before.deck_count - 1);
  await page.locator(".shop-burn-list button").first().click();
  await expect(page.getByRole("button", { name: "금화가 부족합니다" })).toBeDisabled();
  const restored = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()); } finally { game.free(); }
  });
  expect(restored.gold).toBe(before.gold - 100);
  expect(restored.burn_cost).toBe(200);
  expect(restored.deck.some((card: { id: number }) => card.id === burned.id)).toBe(false);
  await page.getByRole("button", { name: "원정 현황" }).click();
  await expect(page.locator(".status-deck-list article")).toHaveCount(before.deck_count - 1);
  await page.getByRole("button", { name: "원정 현황 닫기" }).click();
  await page.getByRole("button", { name: /맵으로 돌아가기/ }).click();
  await expect(page.locator(".expedition-gold")).toContainText(String(before.gold - 100));
});

import { test, expect } from "@playwright/test";
import { dispatch, enemyTurn, newRun, snapshot } from "./helpers/uiRuns";

test("상점 카드 소각은 덱과 골드를 즉시 갱신하고 저장 복원된다", async ({ page }) => {
  const game = newRun("guardian", 2026);
  let saved: string;
  let before: ReturnType<typeof snapshot>;
  try {
    const king = snapshot(game).pieces.find((piece) => piece.key === "king")!;
    dispatch(game, { type: "developer_set_upgrade", id: king.id, target: 94 });
    const nodes = snapshot(game).map.nodes;
    const shop = nodes.find((node) => node.kind === "shop")!;
    const pathCost = (id: number): number => {
      if (id === shop.id) return 0;
      const onward = Math.min(...nodes[id].next.map(pathCost));
      return onward + (nodes[id].kind === "elite" ? 100 : nodes[id].kind === "combat" ? 1 : 0);
    };
    while (snapshot(game).phase !== "shop") {
      const state = snapshot(game);
      const next = state.map.nodes.filter((node) => node.available && Number.isFinite(pathCost(node.id)))
        .sort((a, b) => pathCost(a.id) - pathCost(b.id))[0];
      if (!next) throw new Error("상점으로 이어지는 경로가 없습니다.");
      if (next.kind === "combat" || next.kind === "elite") {
        dispatch(game, { type: "select_node", id: next.id });
        for (const [from, to] of [[52, 36], [60, 52], [52, 43], [43, 35]]) {
          enemyTurn(game);
          if (snapshot(game).phase !== "combat") break;
          try { dispatch(game, { type: "move", from, to }); }
          catch (error) { throw new Error(`노드 ${next.id} (${next.kind}), 이동 ${from}→${to}, 턴 ${snapshot(game).turn}, 상태 ${snapshot(game).phase}: ${String(error)}`); }
          if (snapshot(game).phase !== "combat") break;
        }
        if (snapshot(game).phase !== "reward") throw new Error(`노드 ${next.id} 승리 실패`);
        const cardIndex = snapshot(game).rewards.findIndex((reward) => reward.type === "card");
        if (cardIndex >= 0) dispatch(game, { type: "claim_reward", index: cardIndex });
        dispatch(game, { type: "continue_reward" });
      } else {
        dispatch(game, { type: "select_node", id: next.id });
        if (next.kind === "treasure") dispatch(game, { type: "claim_treasure" });
        if (next.kind === "rest") dispatch(game, { type: "rest_skip" });
        if (next.kind === "event") dispatch(game, { type: "event_choice", choice: "eat" });
      }
      enemyTurn(game);
    }
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

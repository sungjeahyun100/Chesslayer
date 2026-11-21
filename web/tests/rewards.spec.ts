import { test, expect } from "@playwright/test";
import { newRun, dispatch, snapshot, winHillBattle, checkpoint } from "./helpers/uiRuns";
import type { Snapshot } from "../src/types";

test("혼합 보상: 표시 슬롯 획득·다음 전투 드로우·저장 재현", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const game = newRun("guardian", 0);
  let prepared: { saved: string; snapshot: Snapshot };
  try {
    dispatch(game, { type: "developer_set_upgrade", id: 5, target: 94 });
    winHillBattle(game, 1);
    prepared = { saved: game.save(), snapshot: snapshot(game) };
  } finally { game.free(); }
  await page.goto("/");
  expect(JSON.parse(prepared.saved).version).toBe(24);
  expect(prepared.snapshot.phase).toBe("reward");
  expect(prepared.snapshot.rewards).toHaveLength(3);
  for (const type of ["card", "relic"] as const) {
    await page.setViewportSize(type === "card" ? { width: 1440, height: 1000 } : { width: 390, height: 844 });
    const index = prepared.snapshot.rewards.findIndex((r) => r.type === type);
    expect(index).toBeGreaterThanOrEqual(0);
    const chosen = prepared.snapshot.rewards[index];
    await page.evaluate((saved) => localStorage.setItem("chesslayer.run.v1", saved), prepared.saved);
    await page.reload();
    await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
    await expect(page.locator(".reward-cards button")).toHaveCount(3);
    await page.getByRole("button", { name: "원정 현황" }).click();
    await expect(page.getByRole("dialog", { name: "원정 현황" })).toBeVisible();
    await page.getByRole("button", { name: "원정 현황 닫기" }).click();
    await expect(page.locator(".reward-cards button")).toHaveCount(3);
    await expect(page.locator(".relic-reward .card-top").first()).toBeVisible();
    for (const [slot, reward] of prepared.snapshot.rewards.entries()) {
      await expect(page.locator(".reward-cards button").nth(slot)).toHaveAttribute("data-reward-type", reward.type);
      await expect(page.locator(".reward-cards button").nth(slot)).toContainText(reward.definition.name);
      const image = page.locator(".reward-cards button").nth(slot).locator("img");
      await expect(image).toBeVisible();
      expect(await image.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0)).toBe(true);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: `test-results/mixed-rewards-${type}.png`, fullPage: true });
    await page.locator(".reward-cards button").nth(index).click();
    await expect(page.locator("main")).toHaveAttribute("data-phase", "reward");
    await expect(page.locator(".reward-cards button").nth(index)).toContainText("획득 완료");
    await page.getByRole("button", { name: /다음으로 진행/ }).click();
    await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
    const after: Snapshot = await page.evaluate(async ({ saved, index }) => {
      const url = "/src/wasm/chesslayer_web.js";
      const { default: init, WebGame } = await import(url);
      await init();
      const expected = WebGame.restore(saved);
      const resumed = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
      try {
        expected.dispatch(JSON.stringify({ type: "claim_reward", index }));
        expected.dispatch(JSON.stringify({ type: "continue_reward" }));
        if (expected.snapshot() !== resumed.snapshot()) throw new Error("UI selected a different reward slot");
        const result = JSON.parse(resumed.snapshot());
        const nextNode = result.map.nodes.find((n: {available: boolean; visited: boolean; kind: string}) => n.available && !n.visited && n.kind === "combat");
        for (const c of [{ type: "select_node", id: nextNode.id }, { type: "end_turn" }]) {
          expected.dispatch(JSON.stringify(c)); resumed.dispatch(JSON.stringify(c));
          const restored = WebGame.restore(resumed.save());
          try { if (expected.snapshot() !== restored.snapshot()) throw new Error("future RNG diverged after reward resume"); }
          finally { restored.free(); }
        }
        return result;
      } finally { expected.free(); resumed.free(); }
    }, { saved: prepared.saved, index });
    expect(after.rewards).toEqual([]);
    if (type === "card") {
      expect(after.deck_count).toBe(prepared.snapshot.deck_count + 1);
      expect(after.relics).toEqual(prepared.snapshot.relics);
      expect(after.deck.some(c => c.definition.kind === chosen.definition.kind)).toBe(true);
    } else {
      expect(after.deck_count).toBe(prepared.snapshot.deck_count);
      expect(after.relics).toHaveLength(prepared.snapshot.relics.length + 1);
      expect(after.relics.at(-1)?.name).toBe(chosen.definition.name);
    }
  }
  expect(errors).toEqual([]);
});

test("엘리트 강화 보상을 건너뛴 뒤 노드에서 한 번만 받는다", async ({ page }) => {
  const { saved } = checkpoint("elite");
  await page.goto("/");
  await page.evaluate((value) => localStorage.setItem("chesslayer.run.v1", value), saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.getByRole("dialog", { name: "전투 승리" })).toBeVisible();
  await expect(page.locator(".reward-cards [data-reward-type=relic]")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "기물 강화", exact: true })).toBeEnabled();
  const initial = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()) as Snapshot; } finally { game.free(); }
  });
  await page.locator(".reward-cards button").first().click();
  await expect(page.locator(".reward-cards button").first()).toContainText("획득 완료");
  await page.getByRole("button", { name: /다음으로 진행/ }).click();
  await expect(page.locator(".map-reward-badge")).toHaveCount(1);
  const elite = initial.map.nodes.find((node) => node.current)!.id;
  await page.locator(`[data-node-id="${elite}"]`).click();
  await expect(page.getByRole("dialog", { name: "전투 승리" })).toBeVisible();
  await expect(page.locator(".reward-cards button").first()).toContainText("획득 완료");
  await page.getByRole("button", { name: "기물 강화", exact: true }).click();
  await expect(page.getByRole("dialog", { name: "전투 승리" })).toHaveCount(0);
  await page.getByRole("button", { name: /보상 화면으로 돌아가기/ }).click();
  await expect(page.getByRole("dialog", { name: "전투 승리" })).toBeVisible();
  await expect(page.locator(".reward-cards button").first()).toContainText("획득 완료");
  await page.getByRole("button", { name: "기물 강화", exact: true }).click();
  const upgradePiece = initial.rest_options.find((piece) => piece.can_upgrade)!;
  const boardPiece = initial.next_battle_pieces.find((piece) => piece.id === upgradePiece.id)!;
  await page.locator(`[data-square="${boardPiece.square}"]`).click();
  await expect(page.locator(".rest-workshop .skill-tree")).toBeVisible();
  await page.locator(".rest-workshop .skill-node.available").first().click();
  await page.getByRole("button", { name: "강화 확정", exact: true }).click();
  await expect(page.getByRole("dialog", { name: "전투 승리" })).toBeVisible();
  await expect(page.getByRole("button", { name: "기물 강화", exact: true })).toBeDisabled();
  await expect(page.locator(".reward-cards button").first()).toContainText("획득 완료");
  const claimed = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()) as Snapshot; } finally { game.free(); }
  });
  expect(claimed.upgrade_claimed).toBe(true);
  expect(claimed.reward_choice).toBe(0);
  expect(claimed.rest_options.some((piece, index) => piece.kind !== initial.rest_options[index].kind)).toBe(true);
  await page.getByRole("button", { name: /맵으로 돌아가기/ }).click();
  await expect(page.locator(".map-reward-badge")).toHaveCount(0);
});

import { test, expect } from "@playwright/test";
import { newRun, dispatch, snapshot, winHillBattle, enemyTurn } from "./helpers/uiRuns";
import type { Command, RelicId, Snapshot } from "../src/types";

const relicIds: RelicId[] = ["kings_indian", "queens_indian", "hedgehog", "initiative", "ice_cream"];
test("신규 유물 5종: 실제 획득·배치·선공권·누적 코스트·명령별 저장 복원", async ({ page }) => {
  test.setTimeout(120000);
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  for (const relicId of relicIds) {
    const run = { relic: relicId };
    let result: {start: Snapshot; saved: string; next: Snapshot; nextSave: string} | undefined;
    for (let seed = 0; seed < 80 && !result; seed++) {
      const game = newRun("guardian", seed);
      try {
        dispatch(game, { type: "developer_set_upgrade", id: 5, target: 94 });
        winHillBattle(game, snapshot(game).map.nodes.find(n => n.available)!.id);
        const index = snapshot(game).rewards.findIndex(r => r.type === "relic" && r.definition.id === relicId);
        if (index < 0) continue;
        dispatch(game, { type: "claim_reward", index });
        dispatch(game, { type: "continue_reward" });
        const nextNode = snapshot(game).map.nodes.find(n => n.available && !n.visited && n.kind === "combat")!;
        dispatch(game, { type: "select_node", id: nextNode.id });
        const start = snapshot(game), saved = game.save();
        if (relicId === "initiative" || relicId === "ice_cream") { dispatch(game, { type: "end_turn" }); enemyTurn(game); }
        const restored = (await import("../src/wasm/chesslayer_web")).WebGame.restore(game.save());
        try { expect(restored.snapshot()).toBe(game.snapshot()); } finally { restored.free(); }
        result = {start, saved, next: snapshot(game), nextSave: game.save()};
      } finally { game.free(); }
    }
    expect(result).toBeTruthy();
    if (!result) throw new Error(`No legitimate reward fixture for ${relicId}`);
    expect(JSON.parse(result.saved).version).toBe(24);
    expect(result.start.relics.some((r) => r.id === run.relic)).toBe(true);
    if (run.relic === "initiative") {
      expect(result.start.initiative_turns_left).toBe(10);
      expect(result.next.initiative_turns_left).toBe(9);
    }
    if (run.relic === "ice_cream") {
      expect(result.start.mana).toBe(3);
      expect(result.next.mana).toBe(6);
    }

    if (["kings_indian", "queens_indian", "hedgehog"].includes(run.relic)) {
      expect(result.start.pieces.find((p) => p.id === 5)?.square).toBe("g1");
      expect(result.start.pieces.find((p) => p.id === (run.relic === "kings_indian" ? 6 : 3))?.square)
        .toBe(run.relic === "kings_indian" ? "g2" : "b2");
      expect(result.start.rest_options.find((p) => p.id === 5)?.origin).toBe(60);
    }
    await page.setViewportSize(run.relic === "initiative" ? { width: 390, height: 844 } : { width: 1440, height: 1000 });
    await page.evaluate((saved) => localStorage.setItem("chesslayer.run.v1", saved), result.nextSave);
    await page.reload();
    await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
    await expect(page.locator("main")).toHaveAttribute("data-phase", result.next.phase);
    await page.locator(".relic-panel summary").click();
    const relic = result.next.relics.find((r) => r.id === run.relic)!;
    await expect(page.locator(".relic").filter({ hasText: relic.name })).toBeVisible();
    const relicImage = page.locator(".relic").filter({ hasText: relic.name }).locator("img");
    expect(await relicImage.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0)).toBe(true);
    await page.getByRole("button", { name: "원정 현황" }).click();
    await expect(page.locator(".status-relic").filter({ hasText: relic.name })).toHaveCount(1);
    await expect(page.locator(".status-board .piece-image")).toHaveCount(result.next.next_battle_pieces.length);

    await page.locator(".status-relic").filter({ hasText: relic.name }).click();
    await expect(page.locator(".status-relic-detail")).toContainText(relic.description);
    await page.getByRole("button", { name: "원정 현황 닫기" }).click();
    if (run.relic === "initiative") {
      await expect(page.locator(".relic-status")).toHaveText("보호 중 · 남은 적 턴 9회");
      await page.screenshot({ path: "test-results/relic-initiative-mobile.png", fullPage: true });
    }
    if (run.relic === "ice_cream") {
      await expect(page.locator(".mana-orb")).toHaveText("6");
      await expect(page.locator(".relic").filter({ hasText: "아이스크림" })).toContainText(result.next.relics.find(r => r.id === "ice_cream")!.description);
    }
    if (run.relic === "kings_indian") {
      expect(result.start.pieces.find((p) => p.id === 4)?.square).toBe("d1");
      expect(result.start.pieces.find((p) => p.id === 13)?.square).toBe("e2");
      await page.screenshot({ path: "test-results/relic-kia-desktop.png", fullPage: true });
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
  expect(errors).toEqual([]);
});

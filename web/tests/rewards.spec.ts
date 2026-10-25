import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Command, Snapshot } from "../src/types";

test("혼합 보상: 표시 슬롯 획득·다음 전투 드로우·저장 재현", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const commands: Command[] = JSON.parse(readFileSync("../crates/chesslayer/tests/fixtures/winning-run.json", "utf8"));
  const beforeChoice = commands.slice(0, commands.findIndex((c) => c.type === "choose_reward"));
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  const prepared = await page.evaluate(async (commands) => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = new WebGame("vanguard", 2026);
    const verify = () => {
      const before = game.snapshot();
      if (game.snapshot() !== before) throw new Error("snapshot rerolled rewards");
      const restored = WebGame.restore(game.save());
      try {
        if (restored.snapshot() !== before) throw new Error("save replay diverged");
      } finally { restored.free(); }
    };
    try {
      for (const c of commands) {
        game.dispatch(JSON.stringify(c));
        verify();
        let s = JSON.parse(game.snapshot());
        while (s.phase === "combat" && s.turn === 1) {
          s = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" })));
          verify();
        }
      }
      return { saved: game.save(), snapshot: JSON.parse(game.snapshot()) as Snapshot };
    } finally { game.free(); }
  }, beforeChoice);
  expect(JSON.parse(prepared.saved).version).toBe(9);
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
    await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
    const after: Snapshot = await page.evaluate(async ({ saved, index }) => {
      const url = "/src/wasm/chesslayer_web.js";
      const { WebGame } = await import(url);
      const expected = WebGame.restore(saved);
      const resumed = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
      try {
        expected.dispatch(JSON.stringify({ type: "choose_reward", index }));
        if (expected.snapshot() !== resumed.snapshot()) throw new Error("UI selected a different reward slot");
        const result = JSON.parse(resumed.snapshot());
        // Continue both streams across treasure, a new battle, draw and enemy RNG.
        for (const c of [{ type: "select_node", id: 6 }, { type: "claim_treasure" }, { type: "select_node", id: 8 }, { type: "end_turn" }]) {
          expected.dispatch(JSON.stringify(c)); resumed.dispatch(JSON.stringify(c));
          let s = JSON.parse(expected.snapshot());
          while (s.phase === "combat" && s.turn === 1) {
            s = JSON.parse(expected.dispatch(JSON.stringify({ type: "enemy_step" })));
            resumed.dispatch(JSON.stringify({ type: "enemy_step" }));
          }
          const restored = WebGame.restore(resumed.save());
          try {
            if (expected.snapshot() !== restored.snapshot()) throw new Error("future RNG diverged after reward resume");
          } finally { restored.free(); }
        }
        return result;
      } finally { expected.free(); resumed.free(); }
    }, { saved: prepared.saved, index });
    expect(after.rewards).toEqual([]);
    if (type === "card") {
      expect(after.deck_count).toBe(prepared.snapshot.deck_count + 1);
      expect(after.relics).toEqual(prepared.snapshot.relics);
      await page.locator('[data-node-id="6"]').click();
      await page.getByRole("button", { name: /보물 받고 맵으로/ }).click();
      await page.locator('[data-node-id="8"]').click();
      await expect(page.getByRole("heading", { name: "손패 1/10" })).toBeVisible();
      await expect(page.locator(".hand .playing-card")).toContainText(chosen.definition.name);
      await page.locator(".hand .playing-card").click();
      await page.locator('[data-square="g1"]').click();
      await expect(page.locator(".mana-orb")).toHaveText("2");
      await expect(page.locator('[data-square="g1"]')).toHaveAccessibleName(/가호/);
    } else {
      expect(after.deck_count).toBe(prepared.snapshot.deck_count);
      expect(after.relics).toHaveLength(prepared.snapshot.relics.length + 1);
      expect(after.relics.at(-1)?.name).toBe(chosen.definition.name);
    }
  }
  expect(errors).toEqual([]);
});

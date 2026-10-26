import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Command, RelicId, Snapshot } from "../src/types";

const runs: { relic: RelicId; seed: number; commands: Command[] }[] = JSON.parse(
  readFileSync("../crates/chesslayer/tests/fixtures/relic-runs.json", "utf8"),
);

test("신규 유물 6종: 실제 획득·배치·선공권·누적 코스트·명령별 저장 복원", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (error) => errors.push(error.message));
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  for (const run of runs) {
    const result = await page.evaluate(async (run) => {
      const url = "/src/wasm/chesslayer_web.js";
      const { default: init, WebGame } = await import(url);
      await init();
      const game = new WebGame("vanguard", run.seed);
      const check = () => {
        const restored = WebGame.restore(game.save());
        try {
          if (restored.snapshot() !== game.snapshot()) throw new Error("relic replay diverged");
        } finally { restored.free(); }
      };
      const dispatch = (command: Command) => {
        const s: Snapshot = JSON.parse(game.dispatch(JSON.stringify(command)));
        check();
        return s;
      };
      try {
        for (const command of run.commands) {
          let s = dispatch(command);
          while (s.phase === "combat" && s.turn === 1) s = dispatch({ type: "enemy_step" });
        }
        const start: Snapshot = JSON.parse(game.snapshot());
        const saved = game.save();
        if (run.relic === "initiative" || run.relic === "ice_cream") {
          dispatch({ type: "end_turn" });
          let s: Snapshot = JSON.parse(game.snapshot());
          while (s.phase === "combat" && s.turn === 1) s = dispatch({ type: "enemy_step" });
        }
        return { start, saved, next: JSON.parse(game.snapshot()) as Snapshot, nextSave: game.save() };
      } finally { game.free(); }
    }, run);
    expect(JSON.parse(result.saved).version).toBe(12);
    expect(result.start.relics.some((r) => r.id === run.relic)).toBe(true);
    if (run.relic === "initiative") {
      expect(result.start.initiative_turns_left).toBe(10);
      expect(result.next.initiative_turns_left).toBe(9);
    }
    if (run.relic === "ice_cream") {
      expect(result.start.mana).toBe(3);
      expect(result.next.mana).toBe(6);
    }
    if (run.relic === "hard_disk") {
      expect(result.start.phase).toBe("combat");
      expect(result.start.log.some((line) => line.includes("생존 아군 위치를 복원"))).toBe(true);
      expect(result.start.pieces.find((p) => p.id === 4)?.square).toBe("e6");
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
    const relic = result.next.relics.find((r) => r.id === run.relic)!;
    await expect(page.locator(".relic").filter({ hasText: relic.name })).toBeVisible();
    const relicImage = page.locator(".relic").filter({ hasText: relic.name }).locator("img");
    expect(await relicImage.evaluate((el: HTMLImageElement) => el.complete && el.naturalWidth > 0)).toBe(true);
    await page.getByRole("button", { name: "원정 현황" }).click();
    await expect(page.locator(".status-relic").filter({ hasText: relic.name })).toHaveCount(1);
    await expect(page.locator(".status-board .piece-image")).toHaveCount(result.next.next_battle_pieces.length);
    if (run.relic === "hard_disk") {
      for (const piece of result.next.next_battle_pieces) {
        expect(piece.cell).toBe(result.next.rest_options.find((owned) => owned.id === piece.id)?.cell);
        await expect(page.locator(`.status-square[data-cell="${piece.cell}"] img`)).toHaveAttribute("data-piece-key", piece.key);
      }
    }
    await page.locator(".status-relic").filter({ hasText: relic.name }).click();
    await expect(page.locator(".status-relic-detail")).toContainText(relic.description);
    await page.getByRole("button", { name: "원정 현황 닫기" }).click();
    if (run.relic === "initiative") {
      await expect(page.locator(".relic-status")).toHaveText("보호 중 · 남은 적 턴 9회");
      await page.screenshot({ path: "test-results/relic-initiative-mobile.png", fullPage: true });
    }
    if (run.relic === "ice_cream") {
      await expect(page.locator(".mana-orb")).toHaveText("6");
      await expect(page.locator(".mana small")).toContainText("남은 코스트에 3");
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

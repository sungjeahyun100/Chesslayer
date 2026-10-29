import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Command, Snapshot } from "../src/types";

const commands: Command[] = JSON.parse(
  readFileSync("../crates/chesslayer/tests/fixtures/guardian-run.json", "utf8"),
);

test("공용 진형 유물은 수호자에게 적용되고 저장 후 복원된다", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  const prepared = await page.evaluate(async (commands) => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = new WebGame("guardian", 2026);
    try {
      for (const command of commands) {
        let state: Snapshot = JSON.parse(game.snapshot());
        while (state.phase === "combat" && state.turn === 1)
          state = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" })));
        game.dispatch(JSON.stringify(command));
      }
      const before: Snapshot = JSON.parse(game.snapshot());
      const saved = game.save();
      const restored = WebGame.restore(saved);
      try {
        if (restored.snapshot() !== game.snapshot()) throw new Error("relic replay diverged");
      }
      finally { restored.free(); }
      return { saved, before };
    } finally { game.free(); }
  }, [
    ...commands.slice(0, commands.findIndex((c) => c.type === "choose_reward")),
    { type: "choose_reward", index: 1 } as Command,
    { type: "select_node", id: 6 } as Command,
    { type: "claim_treasure" } as Command,
    { type: "select_node", id: 8 } as Command,
  ]);
  expect(JSON.parse(prepared.saved)).toMatchObject({ version: 15, squad: "guardian" });
  expect(prepared.before.relics.some((relic) => relic.id === "kings_indian")).toBe(true);
  expect(prepared.before.pieces.find((piece) => piece.id === 5)?.square).toBe("g1");
  expect(prepared.before.pieces.find((piece) => piece.id === 6)?.square).toBe("g2");
  await page.evaluate((saved) => localStorage.setItem("chesslayer.run.v1", saved), prepared.saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  await expect(page.locator(".relic").filter({ hasText: "인디언 킹의 전략책" })).toBeVisible();
});

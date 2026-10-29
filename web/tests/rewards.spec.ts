import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import type { Command, Snapshot } from "../src/types";

test("혼합 보상: 표시 슬롯 획득·다음 전투 드로우·저장 재현", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const commands: Command[] = JSON.parse(readFileSync("../crates/chesslayer/tests/fixtures/guardian-run.json", "utf8"));
  const beforeChoice = commands.slice(0, commands.findIndex((c) => c.type === "choose_reward"));
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  const prepared = await page.evaluate(async (commands) => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = new WebGame("guardian", 2026);
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
  expect(JSON.parse(prepared.saved).version).toBe(15);
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
      if (chosen.definition.kind === "temporary_upgrade") {
        await page.locator('[data-square="a2"]').click();
        await expect(page.locator(".card-upgrade-picker")).toBeVisible();
        await expect(page.locator(".mana-orb")).toHaveText("3");
      } else {
        await page.locator('[data-square="g1"]').click();
        await expect(page.locator(".mana-orb")).toHaveText("2");
        await expect(page.locator('[data-square="g1"]')).toHaveAccessibleName(/가호/);
      }
    } else {
      expect(after.deck_count).toBe(prepared.snapshot.deck_count);
      expect(after.relics).toHaveLength(prepared.snapshot.relics.length + 1);
      expect(after.relics.at(-1)?.name).toBe(chosen.definition.name);
    }
  }
  expect(errors).toEqual([]);
});

test("엘리트 강화 보상을 건너뛴 뒤 노드에서 한 번만 받는다", async ({ page }) => {
  const commands: Command[] = JSON.parse(readFileSync("../crates/chesslayer/tests/fixtures/guardian-run.json", "utf8"));
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  const saved = await page.evaluate(async (commands) => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = new WebGame("guardian", 2026);
    try {
      for (const command of commands) {
        let state = JSON.parse(game.snapshot()) as Snapshot;
        while (state.phase === "combat" && state.turn === 1)
          state = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" }))) as Snapshot;
        game.dispatch(JSON.stringify(command));
      }
      return game.save();
    } finally { game.free(); }
  }, commands.slice(0, 10));
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
  await expect(page.locator(".skill-node-popup .primary")).toBeVisible();
  await expect(page.locator(".rest-workshop-actions").getByRole("button", { name: "강화 확정", exact: true })).toHaveCount(0);
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

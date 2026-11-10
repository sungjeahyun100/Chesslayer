import { test, expect } from "@playwright/test";
import type { CombatEvent, Snapshot } from "../src/types";

test("최종 보스는 HP·탄약·다음 행동과 위험 범위를 전투 시작부터 공개한다", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await page.locator("button.final-destination").click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  await page.locator('.map-node[data-kind="boss"]').click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  const panel = page.getByRole("status", { name: "부족의 독재자 상태와 행동 예고" });
  await expect(page.getByRole("status", { name: "부족의 독재자 체력", exact: true })).toContainText("HP 5");
  await expect(panel).toContainText("샷건 2/2");
  await expect(panel).toContainText("다음 행동:");
  await expect(page.locator(".boss-intent-panel")).toHaveCount(0);
  await expect(panel).not.toContainText("호위 행동 예고:");
  await expect(page.locator(".piece-layer .piece.owner-1")).toHaveCount(1);
  await expect(page.locator('.piece-layer .piece.owner-1 img[data-piece-key="tribal_dictator"]')).toBeVisible();
  await expect(page.locator(".board-caption")).toHaveCount(0);
  await expect(page.locator(".square.boss-risk")).not.toHaveCount(0);
  await expect(page.getByRole("button", { name: "턴 종료" })).toHaveCount(0);
});

test("boss damage fires the linked chain and carries the attacker to the engine's committed square", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const progress = { highest_floor: 4, unlocked_squads: ["artillery"], final_unlocked: true, final_defeated: false };
    const game = WebGame.start_run("artillery", 1, 0, JSON.stringify(progress));
    try {
      game.dispatch(JSON.stringify({ type: "select_node", id: 0 }));
      localStorage.setItem("chesslayer.run.v1", game.save());
    } finally { game.free(); }
  });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.clock.install();
  await page.clock.pauseAt(new Date());
  await page.locator('.square[data-square="e2"]').click();
  await page.getByRole("button", { name: /특수 능력.*대상 선택/ }).click();
  await page.locator('.square[data-square="e8"]').click();
  await expect(page.locator('.piece[data-piece-id="13"][data-cell="52"]')).toHaveCount(1);
  await page.clock.runFor(620);
  await expect(page.locator(".combat-chain-link").first()).toBeVisible();
  await expect(page.locator('.combat-chain-piece[data-piece-id="13"]')).toHaveAttribute("data-from", "52");
  await expect(page.locator('.combat-chain-piece[data-piece-id="13"]')).toHaveAttribute("data-to", "11");
  await expect(page.locator('.piece-layer .piece[data-piece-id="13"]')).toHaveCount(0);
  await expect(page.locator(".board .square:enabled")).toHaveCount(0);
  await page.clock.runFor(300);
  const top = await page.locator(".combat-chain-piece").evaluate((element) => parseFloat((element as HTMLElement).style.top));
  expect(top).toBeGreaterThan(12.5);
  expect(top).toBeLessThan(75);
  await page.screenshot({ path: "test-results/boss-chain-pull.png" });
  await page.clock.runFor(400);
  await expect(page.locator(".combat-chain-link")).toHaveCount(0);
  await expect(page.locator('.piece[data-piece-id="13"][data-cell="11"]')).toHaveCount(1);
  await expect(page.getByRole("status", { name: "부족의 독재자 체력", exact: true })).toContainText("HP 4");
});

test("boss moves then fires in one turn, showing both plans and the exact shotgun cells", async ({ page }) => {
  await page.goto("/");
  const shot = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const progress = { highest_floor: 4, unlocked_squads: ["artillery"], final_unlocked: true, final_defeated: false };
    const game = WebGame.start_run("artillery", 1, 0, JSON.stringify(progress));
    try {
      let state = JSON.parse(game.dispatch(JSON.stringify({ type: "select_node", id: 0 }))) as Snapshot;
      for (let turn = 0; turn < 12 && state.phase === "combat"; turn++) {
        const plans = state.boss?.plans ?? [];
        const moveThenShot = plans.length === 2 && plans[0].from !== plans[0].to
          && plans[1].name.includes("샷건") && !plans[1].name.includes("재장전");
        state = JSON.parse(game.dispatch(JSON.stringify({ type: "end_turn" }))) as Snapshot;
        const saved = game.save();
        for (let step = 0; step < 2 && state.phase === "combat" && state.turn === 1; step++) {
          const [from, index] = JSON.parse(game.recommend_enemy()) ?? [null, null];
          state = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step_selected", from, index }))) as Snapshot;
          const event = state.combat_events.find((entry) => entry.kind === "shotgun");
          if (moveThenShot && step === 1 && event) {
            localStorage.setItem("chesslayer.run.v1", saved);
            return { event: event as Extract<CombatEvent, { kind: "shotgun" }>, movement: plans[0] };
          }
        }
      }
      throw new Error("Final boss did not use a legal shotgun attack within 12 turns");
    } finally { game.free(); }
  });
  await page.reload();
  await page.clock.install();
  await page.clock.pauseAt(new Date());
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.getByRole("status", { name: "부족의 독재자 상태와 행동 예고" })).toContainText(" → ");
  await expect(page.getByTestId("turn")).toContainText("2회 행동");
  await page.clock.runFor(650);
  await expect(page.locator(".combat-moving-piece")).toHaveCount(1);
  await page.clock.runFor(120);
  await expect(page.locator(`.piece.owner-1[data-cell="${shot.movement.to}"]`)).toHaveCount(1);
  await expect(page.getByTestId("turn")).toContainText("적의 턴");
  await expect(page.getByTestId("turn")).toContainText("1회 행동");
  await page.clock.runFor(650);
  await expect(page.locator(".combat-shotgun-area")).toHaveCount(shot.event.area.length);
  const cells = await page.locator(".combat-shotgun-area").evaluateAll((elements) => elements.map((element) => Number(element.getAttribute("data-cell"))));
  expect(cells.sort((a, b) => a - b)).toEqual([...shot.event.area].sort((a, b) => a - b));
  await page.clock.runFor(110);
  await expect(page.locator(".combat-shotgun-muzzle")).toBeVisible();
  const rays = await page.locator(".combat-shotgun-pellet").evaluateAll((elements) => [...new Set(elements.map((element) => element.getAttribute("data-ray")))]);
  expect(rays.length).toBe(shot.event.rays.filter((ray) => ray.cells.length).length);
  await page.screenshot({ path: "test-results/boss-shotgun-fan.png" });
  await page.clock.runFor(500);
  await expect(page.locator(".combat-shotgun-area")).toHaveCount(0);
  await expect(page.locator(".combat-shotgun-pellet")).toHaveCount(0);
});

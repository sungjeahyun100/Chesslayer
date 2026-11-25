import { expect, test } from "@playwright/test";

test("expedition debugger keeps its save separate and shows real transitions", async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem("chesslayer.run.v1", "untouched-campaign"));
  await page.goto("/expedition-debugger.html");
  await expect(page.getByRole("heading", { name: "원정 진행 디버거" })).toBeVisible();
  await page.getByRole("button", { name: "설정으로 원정 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "강제 승리" }).click();
  await expect(page.locator(".ed-head-status")).toContainText("reward");
  await expect(page.getByText("상태 전이 비교기", { exact: false })).toBeVisible();
  await page.getByRole("button", { name: "보상 건너뛰기 / 계속" }).click();
  await expect(page.locator(".ed-head-status")).toContainText("map");
  await page.getByRole("button", { name: "다음 전투 (강제)" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "현재 전투 재시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  const saves = await page.evaluate(() => ({ campaign: localStorage.getItem("chesslayer.run.v1"),
    debugger: localStorage.getItem("chesslayer.expedition-debugger.run.v1") }));
  expect(saves.campaign).toBe("untouched-campaign");
  expect(saves.debugger).toContain("expedition_debugger");
  await page.locator("summary").filter({ hasText: "스냅샷 · 재현" }).click();
  const [download] = await Promise.all([page.waitForEvent("download"), page.getByRole("button", { name: "JSON 내보내기" }).click()]);
  await page.getByRole("button", { name: "설정", exact: true }).click();
  await page.locator(".ed-upload input").setInputFiles((await download.path())!);
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "반복 실행" }).click();
  await expect(page.getByRole("status").filter({ hasText: "동일 시드 재생 상태 일치" })).toBeVisible();
});

test("engine debug controls use the run reward, boss and defeat paths", async ({ page }) => {
  await page.goto("/expedition-debugger.html");
  const result = await page.evaluate(async () => {
    const { default: init, WebGame, simulator_catalog } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const content = JSON.parse(simulator_catalog());
    const squad = content.squads.find((item: { id: string }) => item.id === "avenger");
    const normal = content.encounters.find((item: { tier: string; floor: number }) => item.tier === "normal" && item.floor === 1);
    const elite = content.encounters.find((item: { tier: string; floor: number }) => item.tier === "elite" && item.floor === 1);
    const boss = content.encounters.find((item: { tier: string; floor: number }) => item.tier === "boss" && item.floor === 1);
    const config = { squad: squad.id, seed: 101, floor: 1, gold: 150, deck: squad.deck, relics: squad.relics,
      upgrades: [], dead: [9], cleared_stages: 0, bosses_defeated: 0, ammo: 10 };
    const game = WebGame.start_expedition_debugger(JSON.stringify(config));
    game.debug_enter_node("combat", normal.index);
    const combat = JSON.parse(game.debug_inspect());
    game.debug_finish_battle(true);
    const reward = JSON.parse(game.snapshot());
    const afterWin = JSON.parse(game.debug_inspect());
    game.dispatch(JSON.stringify({ type: "claim_reward", index: 0 }));
    game.dispatch(JSON.stringify({ type: "continue_reward" }));
    game.debug_enter_node("elite", elite.index);
    game.debug_finish_battle(true);
    const eliteReward = JSON.parse(game.snapshot());
    const upgradePiece = eliteReward.rest_options.find((piece: { can_upgrade: boolean; kind: number }) => piece.can_upgrade
      && eliteReward.upgrade_paths.some((path: { from: number; unlocked: boolean }) => path.from === piece.kind && path.unlocked));
    const upgradePath = eliteReward.upgrade_paths.find((path: { from: number; unlocked: boolean }) => path.from === upgradePiece.kind && path.unlocked);
    game.dispatch(JSON.stringify({ type: "claim_upgrade", id: upgradePiece.id, target: upgradePath.to }));
    const upgraded = JSON.parse(game.debug_inspect()).roster.find((piece: { id: number }) => piece.id === upgradePiece.id);
    game.dispatch(JSON.stringify({ type: "continue_reward" }));
    game.debug_enter_node("boss", boss.index);
    game.debug_finish_battle(true);
    game.dispatch(JSON.stringify({ type: "continue_reward" }));
    const floor = JSON.parse(game.snapshot());
    const saved = game.save();
    const restored = WebGame.restore(saved);
    const same = restored.debug_inspect() === game.debug_inspect();
    const defeat = WebGame.start_expedition_debugger(JSON.stringify(config));
    defeat.debug_enter_node("combat", normal.index);
    defeat.debug_finish_battle(false);
    const lost = JSON.parse(defeat.snapshot());
    const output = { combat, reward, afterWin, eliteReward, upgraded, upgradePath, floor, same, lost };
    game.free(); restored.free(); defeat.free();
    return output;
  });
  expect(result.combat.phase).toBe("combat");
  expect(result.reward.phase).toBe("reward");
  expect(result.reward.rewards).toHaveLength(3);
  expect(result.afterWin.gold).toBe(165);
  const kinds = result.afterWin.trace_events.map((event: { kind: string }) => event.kind);
  expect(kinds.indexOf("battle_start_before")).toBeLessThan(kinds.indexOf("combat_cards"));
  expect(kinds.indexOf("combat_cards")).toBeLessThan(kinds.indexOf("relic_setup"));
  expect(kinds.indexOf("battle_end_before")).toBeLessThan(kinds.indexOf("combat_cards_reset"));
  expect(kinds.indexOf("combat_cards_reset")).toBeLessThan(kinds.indexOf("reward_generated"));
  expect(result.afterWin.roster.find((piece: { id: number }) => piece.id === 9).alive).toBe(false);
  expect(result.eliteReward.phase).toBe("reward");
  expect(result.eliteReward.rest_options.some((piece: { can_upgrade: boolean }) => piece.can_upgrade)).toBe(true);
  expect(result.upgraded.kind).toBe(result.upgradePath.to);
  expect(result.floor.floor).toBe(2);
  expect(result.same).toBe(true);
  expect(result.lost.phase).toBe("defeat");
});

test("temporary combat stats reset while deck, relics and gold survive the next battle", async ({ page }) => {
  await page.goto("/expedition-debugger.html");
  const result = await page.evaluate(async () => {
    const { default: init, WebGame, simulator_catalog } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const content = JSON.parse(simulator_catalog());
    const squad = content.squads[0];
    const normal = content.encounters.find((item: { tier: string; floor: number }) => item.tier === "normal" && item.floor === 1);
    const game = WebGame.start_expedition_debugger(JSON.stringify({ squad: squad.id, seed: 7, floor: 1, gold: 42,
      deck: ["aerobics"], relics: squad.relics, upgrades: [], dead: [], cleared_stages: 0, bosses_defeated: 0, ammo: 10 }));
    game.debug_enter_node("combat", normal.index);
    const initial = JSON.parse(game.snapshot());
    const card = initial.hand.find((item: { definition: { kind: string } }) => item.definition.kind === "aerobics");
    const id = card.targets[0];
    game.dispatch(JSON.stringify({ type: "play_card", id: card.id, target: id }));
    const afterCard = JSON.parse(game.snapshot());
    const boosted = afterCard.pieces.find((piece: { id: number }) => piece.id === id).stats.max_health;
    const cardEvent = (JSON.parse(game.debug_inspect()).trace_events as { kind: string; actor: string | null; target: string | null }[])
      .find((event) => event.kind === "card_resolved");
    game.debug_finish_battle(true);
    game.dispatch(JSON.stringify({ type: "continue_reward" }));
    game.debug_enter_node("combat", normal.index);
    const next = JSON.parse(game.snapshot());
    const later = next.pieces.find((piece: { id: number }) => piece.id === id)?.stats.max_health;
    const output = { initial: initial.pieces.find((piece: { id: number }) => piece.id === id).stats.max_health,
      boosted, later, cardEvent, gold: next.gold, deck: next.deck.map((item: { definition: { kind: string } }) => item.definition.kind),
      relics: next.relics.map((item: { id: string }) => item.id) };
    game.free();
    return output;
  });
  expect(result.boosted).toBeGreaterThan(result.initial);
  expect(result.cardEvent.actor).toBe("유산소 운동");
  expect(result.later).toBe(result.initial);
  expect(result.gold).toBe(57);
  expect(result.deck).toEqual(["aerobics"]);
  expect(result.relics).toHaveLength(1);
});

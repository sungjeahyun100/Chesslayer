import { expect, test } from "@playwright/test";

const settingsKey = "chesslayer.simulator.settings.v1";

test("local simulator configures independent pieces and replays a real battle without touching saves", async ({ page }) => {
  await page.addInitScript(() => localStorage.setItem("chesslayer.run.v1", "existing-expedition"));
  await page.goto("/battle-simulator.html");
  await expect(page.getByRole("heading", { name: "전투 시뮬레이터" })).toBeVisible();
  await expect(page.getByLabel("플레이어 부대").locator("option")).toHaveCount(6);
  await expect(page.locator(".sim-encounter-list button")).toHaveCount(37);
  await page.getByLabel("플레이어 부대").selectOption("guardian");

  const pawnTarget = await page.evaluate(async () => {
    const { default: init, compendium } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const entries = JSON.parse(compendium()) as { id: string; cards: { definition: { kind: string }; available: boolean }[]; relics: { id: string }[]; pieces: { key: string; kind: number; upgrades: { to: number }[] }[] }[];
    const entry = entries.find((squad) => squad.id === "guardian")!;
    const target = entry.pieces.find((piece) => piece.key === "pawn")!.upgrades[0].to;
    return { kind: target, key: entry.pieces.find((piece) => piece.kind === target)!.key,
      cards: entry.cards.filter((card) => card.available).length,
      relics: entry.relics.length };
  });
  await expect(page.locator(".sim-catalog:not(.sim-relics) article")).toHaveCount(pawnTarget.cards);
  await expect(page.locator(".sim-relics article")).toHaveCount(pawnTarget.relics);
  await page.locator(".sim-piece-list button").filter({ hasText: "#9" }).click();
  await page.locator(`.skill-node[data-kind="${pawnTarget.kind}"]`).click();
  await expect(page.locator(".sim-piece-list button").filter({ hasText: "#9" })).toContainText("→");
  await expect(page.locator(".sim-piece-list button").filter({ hasText: "#10" })).toContainText("기본 단계");

  const initialDeck = Number((await page.locator(".sim-launch").textContent())!.match(/덱 (\d+)장/)![1]);
  const card = page.locator(".sim-catalog article").filter({ has: page.locator("button:not([disabled])") }).first();
  await card.getByRole("button", { name: "덱에 추가" }).click();
  await card.getByRole("button", { name: "덱에 추가" }).click();
  await expect(page.locator(".sim-launch")).toContainText(`덱 ${initialDeck + 2}장`);
  const configured = await page.evaluate((key) => localStorage.getItem(key), settingsKey);
  await page.getByRole("button", { name: "전투 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await expect(page.locator(`.piece[data-piece-id="9"] img`)).toHaveAttribute("data-piece-key", pawnTarget.key);
  await expect(page.locator(`.piece[data-piece-id="10"] img`)).toHaveAttribute("data-piece-key", "pawn");
  const firstBattle = await page.locator(".node-navigation").textContent();
  await page.getByRole("button", { name: "전투 재시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await expect(page.locator(".node-navigation")).toHaveText(firstBattle!);
  await page.getByRole("button", { name: "설정으로 돌아가기" }).click();
  await expect(page.locator(".sim-launch")).toContainText(`덱 ${initialDeck + 2}장`);
  expect(await page.evaluate((key) => localStorage.getItem(key), settingsKey)).toBe(configured);
  await page.getByLabel("층 필터").selectOption("final");
  await expect(page.locator(".sim-encounter-list button")).toHaveCount(1);
  await page.locator(".sim-encounter-list button").click();
  await page.getByRole("button", { name: "전투 시작" }).click();
  await expect(page.getByRole("status", { name: "부족의 독재자 체력" })).toBeVisible();
  await page.getByRole("button", { name: "전투 상대 변경" }).click();
  await expect(page.locator(".sim-launch")).toContainText(`덱 ${initialDeck + 2}장`);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe("existing-expedition");
});

test("squad selection exposes only its content pool and resets the edited setup", async ({ page }) => {
  await page.goto("/battle-simulator.html");
  const pools = await page.evaluate(async () => {
    const { default: init, compendium, simulator_catalog } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const entries = JSON.parse(compendium()) as { id: string; cards: { definition: { kind: string; name: string; description: string; rarity?: string }; available: boolean }[];
      relics: { id: string; name: string; description: string }[] }[];
    const squads = JSON.parse(simulator_catalog()).squads as { id: string; deck: string[]; relics: string[] }[];
    return { entries, squads };
  });
  for (const id of ["guardian", "vanguard", "artillery"]) {
    await page.getByLabel("플레이어 부대").selectOption(id);
    const entry = pools.entries.find((item) => item.id === id)!;
    expect(await page.locator(".sim-catalog:not(.sim-relics) article strong").allTextContents())
      .toEqual(entry.cards.filter((card) => card.available).map((card) => card.definition.name));
    expect(await page.locator(".sim-relics article strong").allTextContents())
      .toEqual(entry.relics.map((relic) => relic.name));
    await page.getByLabel("카드 희귀도").selectOption("rare");
    expect(await page.locator(".sim-catalog:not(.sim-relics) article strong").allTextContents())
      .toEqual(entry.cards.filter((card) => card.available && card.definition.rarity === "rare").map((card) => card.definition.name));
    await page.getByLabel("카드 희귀도").selectOption("all");
    const cardQuery = entry.cards.find((card) => card.available)!.definition.kind;
    await page.getByLabel("카드 검색").fill(cardQuery);
    expect(await page.locator(".sim-catalog:not(.sim-relics) article strong").allTextContents())
      .toEqual(entry.cards.filter((card) => card.available &&
        `${card.definition.name} ${card.definition.description} ${card.definition.kind}`.toLowerCase().includes(cardQuery.toLowerCase()))
        .map((card) => card.definition.name));
    await page.getByLabel("카드 검색").fill("");
    const relicQuery = entry.relics[0].id;
    await page.getByLabel("유물 검색").fill(relicQuery);
    expect(await page.locator(".sim-relics article strong").allTextContents())
      .toEqual(entry.relics.filter((relic) =>
        `${relic.name} ${relic.description} ${relic.id}`.toLowerCase().includes(relicQuery.toLowerCase()))
        .map((relic) => relic.name));
    await page.getByLabel("유물 검색").fill("");
    const defaults = pools.squads.find((item) => item.id === id)!;
    const stored = await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), settingsKey);
    expect(stored.deck).toEqual(defaults.deck);
    expect(stored.relics).toEqual(defaults.relics);
    expect(stored.upgrades).toEqual({});
    if (id === "guardian") {
      await page.locator(".sim-catalog:not(.sim-relics) article button").first().click();
      await page.locator(".sim-relics article button:not([disabled])").first().click();
      await page.locator(".sim-piece-list button").filter({ hasText: "#9" }).click();
      await page.locator(".skill-node:not(.root-node)").first().click();
    }
  }
  const vanguardOnly = pools.entries.find((item) => item.id === "vanguard")!.cards
    .find((card) => !pools.entries.find((item) => item.id === "artillery")!.cards.some((other) => other.definition.kind === card.definition.kind));
  expect(vanguardOnly).toBeDefined();
  await expect(page.locator(".sim-catalog:not(.sim-relics)")).not.toContainText(vanguardOnly!.definition.name);
  const nonArtilleryRelic = pools.entries.find((item) => item.id === "vanguard")!.relics
    .find((relic) => !pools.entries.find((item) => item.id === "artillery")!.relics.some((other) => other.id === relic.id));
  expect(nonArtilleryRelic).toBeDefined();
  await expect(page.locator(".sim-relics")).not.toContainText(nonArtilleryRelic!.name);
});

test("battle setup survives return, replay, reload, and sanitizes stale content", async ({ page }) => {
  await page.goto("/battle-simulator.html");
  await page.evaluate(() => localStorage.setItem("chesslayer.run.v1", "existing-expedition"));
  await page.getByLabel("플레이어 부대").selectOption("guardian");
  const target = await page.evaluate(async () => {
    const { default: init, compendium } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const entries = JSON.parse(compendium()) as { id: string; pieces: { key: string; upgrades: { to: number }[] }[] }[];
    return entries.find((entry) => entry.id === "guardian")!.pieces.find((piece) => piece.key === "pawn")!.upgrades[0].to;
  });
  await page.locator(".sim-piece-list button").filter({ hasText: "#9" }).click();
  await page.locator(`.skill-node[data-kind="${target}"]`).click();
  await page.locator(".sim-catalog:not(.sim-relics) article button").first().click();
  await page.locator(".sim-catalog:not(.sim-relics) article button").first().click();
  await page.locator(".sim-relics article button:not([disabled])").first().click();
  await page.getByLabel("층 필터").selectOption("final");
  await page.locator(".sim-encounter-list button").click();
  const before = await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), settingsKey);
  await page.getByRole("button", { name: "전투 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "전투 재시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "설정으로 돌아가기" }).click();
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), settingsKey)).toEqual(before);
  await expect(page.locator(".sim-piece-list button").filter({ hasText: "#9" })).toContainText("→");
  await page.getByRole("button", { name: "전투 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "설정으로 돌아가기" }).click();
  await page.reload();
  await expect(page.locator(".sim-launch")).toBeVisible();
  await expect(page.locator(".sim-enemy-detail")).toContainText("최종 결전");
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), settingsKey)).toEqual(before);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe("existing-expedition");

  await page.evaluate(({ key, before, target }) => localStorage.setItem(key, JSON.stringify({ ...before,
    deck: [...before.deck, "removed-card", "removed-card"], relics: [...before.relics, "removed-relic", before.relics[0]],
    upgrades: { ...before.upgrades, 9: 999999, 999: target },
  })), { key: settingsKey, before, target });
  await page.reload();
  await expect(page.locator(".sim-launch")).toBeVisible();
  const cleaned = await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), settingsKey);
  expect(cleaned.deck).toEqual(before.deck);
  expect(cleaned.relics).toEqual(before.relics);
  expect(cleaned.upgrades).toEqual({});
  expect(cleaned.squad).toBe(before.squad);
  expect(cleaned.encounter).toBe(before.encounter);
});

test("simulator enemy AI uses the replay worker", async ({ page }) => {
  await page.goto("/battle-simulator.html");
  await page.getByLabel("플레이어 부대").selectOption("avenger");
  await page.getByRole("button", { name: "전투 시작" }).click();
  await expect(page.locator('main[data-phase="combat"]')).toBeVisible();
  await page.getByRole("button", { name: "턴 종료" }).click();
  await expect(page.getByTestId("turn")).toContainText("적의 턴");
  await expect(page.getByTestId("turn")).toContainText("나의 턴", { timeout: 30_000 });
  await expect(page.locator(".sim-error")).toHaveCount(0);
});

test("summoned pawn rank remains on the simulator board after card motion and a turn", async ({ page }) => {
  await page.goto("/battle-simulator.html");
  await page.getByLabel("플레이어 부대").selectOption("avenger");
  for (let i = 0; i < 10; i++) await page.locator(".sim-chips button[aria-label$='한 장 제거']").first().click();
  await page.getByLabel("카드 검색").fill("얼마나 박아댄 거야");
  await page.getByRole("button", { name: "덱에 추가" }).click();
  await page.getByLabel("유물 검색").fill("궁궐농밀 아기만들기");
  await page.getByRole("button", { name: "장착", exact: true }).click();
  await page.getByRole("button", { name: "전투 시작" }).click();
  await page.locator(".hand .playing-card").filter({ hasText: "얼마나 박아댄 거야" }).click();
  await page.locator('[data-square="a6"]').click();
  await expect(page.locator('.piece[data-piece-id^="100000"]')).toHaveCount(8);
  await page.waitForTimeout(600);
  await expect(page.locator('.piece[data-piece-id^="100000"]')).toHaveCount(8);
  await page.getByRole("button", { name: "턴 종료" }).click();
  await expect(page.getByTestId("turn")).toContainText("나의 턴", { timeout: 30_000 });
  expect(await page.locator('.piece[data-piece-id^="100000"]').count()).toBeGreaterThan(0);
});

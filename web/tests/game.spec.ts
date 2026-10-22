import { test, expect } from "@playwright/test";

async function startMap(
  page: import("@playwright/test").Page,
  character = "선봉대",
  fixedSeed = true,
) {
  if (fixedSeed)
    await page.addInitScript(() => {
      const original = crypto.getRandomValues.bind(crypto);
      crypto.getRandomValues = ((array: Uint32Array) => {
        if (array instanceof Uint32Array && array.length === 1) {
          array[0] = 2026;
          return array;
        }
        return original(array);
      }) as typeof crypto.getRandomValues;
    });
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: new RegExp(character) }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  await expect(page.locator(".map-node:enabled")).toHaveCount(3);
}
async function start(
  page: import("@playwright/test").Page,
  character = "선봉대",
) {
  await startMap(page, character);
  await page
    .locator(".map-node:enabled")
    .filter({ hasText: "⚔" })
    .and(page.getByRole("button", { name: /홀로 선 왕/ }))
    .first()
    .click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
}
async function layout(page: import("@playwright/test").Page) {
  return page.locator(".map-node").evaluateAll((nodes) =>
    nodes.map((n) => ({
      id: n.getAttribute("data-node-id"),
      kind: n.getAttribute("data-kind"),
      next: n.getAttribute("data-next"),
      position: n.parentElement!.getAttribute("style"),
    })),
  );
}

test("실제 WASM: 초기 진형, 합법 수, 카드 사용, 적 행동", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await start(page);
  await expect(page.locator(".piece.owner-0")).toHaveCount(16);
  await expect(page.locator('[data-square="e8"]')).toHaveAccessibleName(
    "e8 흑 킹 로얄",
  );
  await expect(page.getByRole("heading", { name: "손패 5/10" })).toBeVisible();
  await page
    .getByRole("button", { name: /^1 전술 카드 가호/ })
    .first()
    .click();
  await page.locator('[data-square="e1"]').click();
  await expect(page.locator(".mana-orb")).toHaveText("2");
  await expect(page.getByTestId("turn")).toContainText("2회 행동");
  await expect(page.locator('[data-square="e1"]')).toHaveAccessibleName(
    "e1 백 킹 로얄 가호",
  );
  await page.locator('[data-square="e2"]').click();
  await expect(page.locator('[data-square="e4"]')).toHaveAttribute(
    "data-legal",
    "true",
  );
  await page.locator('[data-square="e4"]').click();
  await expect(page.getByTestId("turn")).toContainText("1회 행동");
  await page.locator('[data-square="d2"]').click();
  await page.locator('[data-square="d4"]').click();
  await expect(page.getByTestId("turn")).toContainText("적의 턴");
  await expect(page.getByTestId("turn")).toContainText("나의 턴");
  await expect(page.getByRole("log")).toContainText("흑 킹");
  await expect(page.locator(".mana-orb")).toHaveText("3");
  await page.screenshot({
    path: "test-results/battle-desktop.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});

test("모바일 보드와 카드가 화면 안에서 조작된다", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await start(page, "수호자");
  await page.locator('[data-square="e2"]').click();
  await expect(page.locator('[data-square="e3"]')).toHaveAttribute(
    "data-legal",
    "true",
  );
  const width = await page.evaluate(() => ({
    body: document.documentElement.scrollWidth,
    viewport: innerWidth,
  }));
  expect(width.body).toBeLessThanOrEqual(width.viewport);
  await page.screenshot({
    path: "test-results/battle-mobile.png",
    fullPage: true,
  });
});

test("생성된 맵을 유지하며 분기·합류·보물·엘리트·휴식·보스까지 완주", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const { readFileSync } = await import("node:fs");
  const commands = JSON.parse(
    readFileSync(
      "../crates/chesslayer/tests/fixtures/winning-run.json",
      "utf8",
    ),
  );
  const square = (cell: number) =>
    `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;
  await startMap(page);
  const generated = await layout(page);
  await page.screenshot({
    path: "test-results/map-desktop.png",
    fullPage: true,
  });
  let current: number | null = null;
  let expectedNext: string[] = [];
  let branches = 0;
  let merges = 0;
  let selections = 0;
  const visitedKinds = new Set<string>();
  for (const cmd of commands) {
    if (cmd.type === "select_node") {
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
      expect(await layout(page)).toEqual(generated);
      if (current !== null) {
        await expect(
          page.locator(`[data-node-id="${current}"]`),
        ).toHaveAttribute("aria-current", "location");
        const enabled = await page
          .locator(".map-node:enabled")
          .evaluateAll((nodes) =>
            nodes.map((n) => n.getAttribute("data-node-id")),
          );
        expect(enabled.sort()).toEqual(expectedNext.sort());
        if (enabled.length > 1) branches++;
        await expect(
          page.locator(`[data-node-id="${current}"]`),
        ).toBeDisabled();
      }
      const node = page.locator(`[data-node-id="${cmd.id}"]`);
      const kind = (await node.getAttribute("data-kind"))!;
      visitedKinds.add(kind);
      expectedNext = (await node.getAttribute("data-next"))!
        .split(",")
        .filter(Boolean);
      if (
        generated.filter((n) => n.next!.split(",").includes(String(cmd.id)))
          .length > 1
      )
        merges++;
      await expect(node).toBeEnabled();
      await node.click();
      current = cmd.id;
      selections++;
      if (selections === 1) {
        await page.getByRole("button", { name: "전체 지도 보기" }).click();
        expect(await layout(page)).toEqual(generated);
        await expect(page.locator(".map-node:enabled")).toHaveCount(0);
        await page
          .getByRole("button", { name: "진행 중인 노드로 돌아가기" })
          .click();
      }
      if (kind === "boss")
        await expect(
          page.getByRole("heading", { name: "마하라자", exact: true }),
        ).toBeVisible();
    } else if (cmd.type === "move") {
      const from = page.locator(`[data-square="${square(cmd.from)}"]`);
      await expect(from).toBeEnabled();
      await from.click();
      await expect(
        page.locator(`[data-square="${square(cmd.to)}"]`),
      ).toHaveAttribute("data-legal", "true");
      await page.locator(`[data-square="${square(cmd.to)}"]`).click();
    } else if (cmd.type === "play_card") {
      await page.locator(`[data-card-id="${cmd.id}"]`).click();
      const target = Number(
        await page
          .locator(`[data-piece-id="${cmd.target}"]`)
          .getAttribute("data-cell"),
      );
      await page.locator(`[data-square="${square(target)}"]`).click();
    } else if (cmd.type === "choose_reward") {
      await expect(
        page.getByRole("heading", { name: "전투 승리", exact: true }),
      ).toBeVisible();
      await page
        .getByRole("button", { name: "보상 가호", exact: true })
        .click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
      await expect(page.locator(".map-supplies")).toContainText("덱");
    } else if (cmd.type === "rest_upgrade") {
      await page.getByRole("button", { name: "♙ a2의 폰 → 템페스트" }).click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
    } else if (cmd.type === "claim_treasure") {
      await expect(
        page.getByRole("heading", { name: "남겨진 보물", exact: true }),
      ).toBeVisible();
      await page.getByRole("button", { name: "보물 받고 맵으로" }).click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
    }
  }
  expect(selections).toBe(7);
  expect(branches).toBeGreaterThanOrEqual(2);
  expect(merges).toBeGreaterThanOrEqual(2);
  expect([...visitedKinds].sort()).toEqual([
    "boss",
    "combat",
    "elite",
    "rest",
    "treasure",
  ]);
  await expect(page.locator("main")).toHaveAttribute("data-phase", "victory");
  await expect(
    page.getByRole("heading", { name: "마지막 왕관까지." }),
  ).toBeVisible();
  await expect(page.getByRole("log")).toContainText("역습");
  await page.screenshot({
    path: "test-results/map-run-victory.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});

test("새 원정마다 다른 전체 맵과 모바일 경로 선택", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await startMap(page, "수호자", false);
  const first = await layout(page);
  await expect(page.locator('.map-node[data-kind="boss"]')).toHaveCount(1);
  const width = await page.evaluate(() => ({
    body: document.documentElement.scrollWidth,
    viewport: innerWidth,
  }));
  expect(width.body).toBeLessThanOrEqual(width.viewport);
  await page.screenshot({
    path: "test-results/map-mobile.png",
    fullPage: true,
  });
  await startMap(page, "수호자", false);
  expect(await layout(page)).not.toEqual(first);
  await page.locator(".map-node:enabled").first().click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
});

test("킹 포획 시 패배하고 새 원정을 시작할 수 있다", async ({ page }) => {
  const { readFileSync } = await import("node:fs");
  const commands = JSON.parse(
    readFileSync("../crates/chesslayer/tests/fixtures/losing-run.json", "utf8"),
  );
  const square = (cell: number) =>
    `${"abcdefgh"[cell % 8]}${8 - Math.floor(cell / 8)}`;
  await start(page);
  for (const cmd of commands) {
    const from = page.locator(`[data-square="${square(cmd.from)}"]`);
    await expect(from).toBeEnabled();
    await from.click();
    await page.locator(`[data-square="${square(cmd.to)}"]`).click();
  }
  await expect(page.locator("main")).toHaveAttribute("data-phase", "defeat");
  await expect(
    page.getByRole("heading", { name: "왕이 쓰러졌습니다." }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "턴 종료" })).toBeDisabled();
  await page.getByRole("button", { name: "새 원정 시작" }).click();
  await expect(
    page.getByRole("heading", { name: "어떤 수를 두시겠어요?" }),
  ).toBeVisible();
});

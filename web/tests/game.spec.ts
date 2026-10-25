import { test, expect } from "@playwright/test";
import type { Command } from "../src/types";

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
  const firstEncounter = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return (JSON.parse(game.snapshot()).map.nodes as { id: number; name: string }[]).find((node) => node.name === "홀로 선 왕")!.id; }
    finally { game.free(); }
  });
  await page.locator(`[data-node-id="${firstEncounter}"]`).click();
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

test("적 로얄 조회는 행마와 정보를 표시하지만 턴·저장을 바꾸지 않는다", async ({ page }) => {
  await start(page);
  const enemyCell = Number(await page.locator(".piece.owner-1").first().getAttribute("data-cell"));
  const allyCell = Number(await page.locator(".piece.owner-0").first().getAttribute("data-cell"));
  const saved = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  const turn = await page.locator(".section-label").first().textContent();
  await page.locator(".square").nth(enemyCell).click();
  await expect(page.locator(".square.enemy-selected")).toHaveCount(1);
  await expect(page.locator(".enemy-move-dot")).not.toHaveCount(0);
  await expect(page.locator(".enemy-piece-info")).toContainText("승리 목표");
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(saved);
  expect(await page.locator(".section-label").first().textContent()).toBe(turn);
  await page.locator(".square").nth(enemyCell).click();
  await expect(page.locator(".enemy-piece-info")).toHaveCount(0);
  await page.locator(".square").nth(enemyCell).click();
  await page.locator(".square").nth(allyCell).click();
  await expect(page.locator(".selected-piece-info:not(.enemy-piece-info)")).toHaveCount(1);
  await page.locator(".square").nth(enemyCell).click();
  await page.keyboard.press("Escape");
  await expect(page.locator(".enemy-piece-info")).toHaveCount(0);
});

test("새 원정은 ACT I 연출 뒤 지도로 들어가고 저장 재개에는 연출을 반복하지 않는다", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".home-version")).toHaveText("v0.2.3");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.getByRole("status", { name: "제1층 진입" })).toContainText("ACT I");
  await expect(page.locator(".floor-transition")).toHaveCount(0);
  await expect(page.locator(".act-map-screen")).toHaveAttribute("data-floor", "1");
  const saved = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  expect(JSON.parse(saved!).commands).toHaveLength(0);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator(".floor-transition")).toHaveCount(0);
  await expect(page.locator(".map-node:enabled")).toHaveCount(3);

  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.getByRole("status", { name: "제1층 진입" })).toBeVisible();
  await page.getByRole("button", { name: "층 전환 연출 건너뛰기" }).click();
  await expect(page.locator(".act-map-screen")).toHaveAttribute("data-floor", "1");

  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.reload();
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator(".floor-transition")).toHaveCount(0);
  await expect(page.locator(".act-map-screen")).toHaveAttribute("data-floor", "1");
});

test("원정 현황 버튼은 우측 상단에 고정되고 실제 덱·진입 배치·유물을 연다", async ({ page }) => {
  await startMap(page);
  const before = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  const snapshot = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()) as { rest_options: { kind: number }[]; next_battle_pieces: unknown[]; relics: unknown[] }; }
    finally { game.free(); }
  });
  await expect(page.locator(".map-aside, .map-legend")).toHaveCount(0);
  const trigger = page.getByRole("button", { name: "원정 현황" });
  const top = await trigger.boundingBox();
  await page.locator('.map-node[data-row="0"]').first().scrollIntoViewIfNeeded();
  const afterScroll = await trigger.boundingBox();
  expect(afterScroll?.x).toBeCloseTo(top!.x, 0);
  expect(afterScroll?.y).toBeCloseTo(top!.y, 0);
  await page.getByRole("button", { name: "원정 현황" }).click();
  await expect(page.getByRole("dialog", { name: "원정 현황" })).toBeVisible();
  await expect(page.locator(".map-overlay")).toHaveCount(0);
  await expect(page.locator(".status-piece-group")).toHaveCount(new Set(snapshot.rest_options.map((piece) => piece.kind)).size);
  await expect(page.locator(".status-board .piece-image")).toHaveCount(snapshot.next_battle_pieces.length);
  await expect(page.locator(".status-relic")).toHaveCount(snapshot.relics.length);
  await page.locator(".status-piece-group").first().click();
  await expect(page.locator(".status-piece-detail")).toBeVisible();
  await page.locator(".status-relic").first().click();
  await expect(page.locator(".status-relic-detail")).toBeVisible();
  await expect(page.locator(".map-aside, .map-legend")).toHaveCount(0);
  await page.getByRole("button", { name: "원정 현황 닫기" }).click();
  await page.setViewportSize({ width: 390, height: 844 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(before);
  await expect(page.locator(".map-node:enabled")).toHaveCount(3);
});

test("14단계 지도의 두 휴식 구간과 보스를 스크롤해서 확인한다", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await startMap(page);
  const map = page.locator(".map-chart");
  await expect(page.locator('.map-node[data-row="13"][data-kind="boss"]')).toHaveCount(1);
  for (const row of [6, 12]) {
    const nodes = page.locator(`.map-node[data-row="${row}"]`);
    expect(await nodes.count()).toBeGreaterThan(0);
    expect(await nodes.evaluateAll((items) => items.every((node) => node.getAttribute("data-kind") === "rest"))).toBe(true);
  }
  expect(await map.evaluate((element) => element.getBoundingClientRect().height)).toBeGreaterThan(1300);
  await page.locator('.map-node[data-row="0"]').first().scrollIntoViewIfNeeded();
  await expect(page.locator('.map-node[data-row="0"]').first()).toBeInViewport();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.locator('.map-node[data-row="0"]:enabled').first().click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
});

test("지도 노드는 연결된 경로에 가까이 배치되고 서로 겹치지 않는다", async ({ page }) => {
  await startMap(page);
  await expect(page.locator(".node-caption")).toHaveCount(0);
  const rowDistance = await page.locator(".map-chart").evaluate((chart) => {
    const row0 = chart.querySelector<HTMLElement>('.map-node[data-row="0"]')!.parentElement!;
    const row1 = chart.querySelector<HTMLElement>('.map-node[data-row="1"]')!.parentElement!;
    return (parseFloat(row0.style.top) - parseFloat(row1.style.top)) * chart.getBoundingClientRect().height / 100;
  });
  expect(rowDistance).toBeCloseTo(104.5, 1);
  const nodes = await page.locator(".map-node").evaluateAll((buttons) =>
    buttons.map((button) => ({
      id: Number(button.getAttribute("data-node-id")),
      row: Number(button.getAttribute("data-row")),
      next: (button.getAttribute("data-next") ?? "").split(",").filter(Boolean).map(Number),
      x: parseFloat(button.parentElement!.style.left) * 5,
    })),
  );
  for (const row of new Set(nodes.map((node) => node.row))) {
    const positions = nodes.filter((node) => node.row === row).map((node) => node.x);
    for (let index = 1; index < positions.length; index++) {
      expect(positions[index] - positions[index - 1]).toBeGreaterThanOrEqual(55.9);
    }
  }
  const byId = new Map(nodes.map((node) => [node.id, node]));
  const horizontalCost = (x: (node: typeof nodes[number]) => number) =>
    nodes.reduce((cost, node) => cost + node.next.reduce((sum, id) =>
      sum + (x(node) - x(byId.get(id)!)) ** 2, 0), 0);
  const fixedX = (node: typeof nodes[number]) => {
    const peers = nodes.filter((peer) => peer.row === node.row);
    return 250 + (peers.findIndex((peer) => peer.id === node.id) - (peers.length - 1) / 2) * 110;
  };
  expect(horizontalCost((node) => node.x)).toBeLessThan(horizontalCost(fixedX));
  for (const width of [390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    const overlaps = await page.locator(".map-node").evaluateAll((buttons) => {
      const rows = new Map<string, DOMRect[]>();
      for (const button of buttons) {
        const row = button.getAttribute("data-row")!;
        rows.set(row, [...(rows.get(row) ?? []), button.getBoundingClientRect()]);
      }
      return [...rows.values()].some((boxes) =>
        boxes.slice(1).some((box, index) => boxes[index].right > box.left),
      );
    });
    expect(overlaps).toBe(false);
  }
});

test("전투 전 적 정체를 숨기고 전투 중 현황 오버레이가 상태를 보존한다", async ({ page }) => {
  await startMap(page);
  const nodes = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(game.snapshot()).map.nodes as { id: number; kind: string; name: string }[]; }
    finally { game.free(); }
  });
  for (const node of nodes.filter((item) => item.kind === "combat" || item.kind === "elite")) {
    const button = page.locator(`[data-node-id="${node.id}"]`);
    await expect(button).not.toHaveAttribute("title", new RegExp(node.name));
    await expect(button).not.toHaveAttribute("aria-label", new RegExp(node.name));
    await expect(button.locator("xpath=..")).not.toContainText(node.name);
  }
  const boss = nodes.find((node) => node.kind === "boss")!;
  await expect(page.locator(`[data-node-id="${boss.id}"]`)).toHaveAttribute("title", new RegExp(boss.name));
  await page.locator('.map-node[data-kind="combat"]:enabled').first().click();
  const saved = await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"));
  const before = await page.locator(".board").getAttribute("class");
  await page.getByRole("button", { name: "원정 현황" }).click();
  await expect(page.getByRole("dialog", { name: "원정 현황" })).toBeVisible();
  await expect(page.locator(".status-board .piece-image")).toHaveCount(await page.locator(".piece.owner-0").count());
  await expect(page.locator(".map-node:enabled")).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(saved);
  await page.getByRole("button", { name: "원정 현황 닫기" }).click();
  await expect(page.getByRole("dialog", { name: "원정 현황" })).toHaveCount(0);
  expect(await page.locator(".board").getAttribute("class")).toBe(before);
  expect(await page.evaluate(() => localStorage.getItem("chesslayer.run.v1"))).toBe(saved);
});

test("실제 WASM: 빈 초기 덱, 합법 수, 적 행동", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await start(page);
  await expect(page.locator(".piece.owner-0")).toHaveCount(16);
  await expect(page.locator('[data-square="e8"]')).toHaveAccessibleName(
    "e8 흑 킹 로얄",
  );
  await expect(page.getByRole("heading", { name: "손패 0/10" })).toBeVisible();
  await expect(page.getByTestId("deck-count")).toHaveText("0");
  await expect(page.getByTestId("draw-count")).toHaveText("0");
  await expect(page.locator(".mana-orb")).toHaveText("3");
  await expect(page.getByTestId("turn")).toContainText("2회 행동");
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
  await expect(page.getByRole("heading", { name: "손패 0/10" })).toBeVisible();
  await expect(page.getByTestId("deck-count")).toHaveText("0");
  await expect(page.locator(".mana-orb")).toHaveText("3");
  await page.screenshot({
    path: "test-results/battle-desktop.png",
    fullPage: true,
  });
  expect(errors).toEqual([]);
});

test("모바일 빈 덱 전투 보드가 화면 안에서 조작된다", async ({ page }) => {
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

test("생성된 맵을 유지하며 분기·보물·전투·휴식까지 진행", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const { readFileSync } = await import("node:fs");
  const commands = (JSON.parse(
    readFileSync(
      "../crates/chesslayer/tests/fixtures/winning-run.json",
      "utf8",
    ),
  ) as Command[]).slice(0, 14);
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
        expect(enabled.filter((id) => id !== String(current)).sort()).toEqual(expectedNext.sort());
        if (expectedNext.length > 1) branches++;
        const previous = page.locator(`[data-node-id="${current}"]`);
        if ((await previous.getAttribute("data-status"))?.includes("보상"))
          await expect(previous).toBeEnabled();
        else
          await expect(previous).toBeDisabled();
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
        await page.getByRole("button", { name: "원정 현황" }).click();
        await expect(page.getByRole("dialog", { name: "원정 현황" })).toBeVisible();
        await expect(page.locator(".map-overlay")).toHaveCount(0);
        await expect(page.locator(".status-board .piece-image")).not.toHaveCount(0);
        await page.getByRole("button", { name: "원정 현황 닫기" }).click();
      }
      if (kind === "boss") {
        await expect(
          page.getByRole("heading", { name: "구행", exact: true }),
        ).toBeVisible();
        if (await page.locator("main").getAttribute("data-phase") === "combat") {
          const hook = page.locator('[data-piece-id="100"] img');
          await expect(hook).toHaveAttribute("data-piece-key", "hook");
          await expect(hook).toBeVisible();
          await expect.poll(() => hook.evaluate((img) => (img as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
          await page.screenshot({ path: "test-results/hook-boss.png", fullPage: true });
        } else {
          await expect(page.locator("main")).toHaveAttribute("data-phase", "reward");
          await expect(page.getByRole("log")).toContainText("하드디스크");
        }
      }
    } else if (cmd.type === "move") {
      const from = page.locator(`[data-square="${square(cmd.from)}"]`);
      await expect(from).toBeEnabled();
      await from.click();
      await expect(
        page.locator(`[data-square="${square(cmd.to)}"]`),
      ).toHaveAttribute("data-legal", "true");
      await page.locator(`[data-square="${square(cmd.to)}"]`).click();
      if (cmd.promotion) {
        const name = await page.evaluate(async (kind) => {
          const url = "/src/wasm/chesslayer_web.js";
          const { default: init, WebGame } = await import(url);
          await init();
          const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
          try {
            return JSON.parse(game.snapshot()).moves.find((m: any) => m.promotion?.kind === kind).promotion.name;
          } finally { game.free(); }
        }, cmd.promotion);
        await page.getByRole("button", { name: `${name}으로 승격`, exact: true }).click();
      }
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
      await expect(page.locator(".reward-cards button")).toHaveCount(3);
      await page.locator(".reward-cards button").nth(cmd.index).click();
      await page.getByRole("button", { name: /다음으로 진행/ }).click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
    } else if (cmd.type === "rest_upgrade") {
      await page.locator('[data-square="a2"]').click();
      await page
        .getByRole("button", { name: "템페스트 폰 · 선택 가능 · 상세 보기" })
        .click();
      await page
        .getByRole("button", { name: "강화 확정", exact: true })
        .click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
      await page.getByRole("button", { name: "맵으로 돌아가기 →", exact: true }).click();
    } else if (cmd.type === "claim_treasure") {
      await expect(
        page.getByRole("heading", { name: "남겨진 보물", exact: true }),
      ).toBeVisible();
      await page.getByRole("button", { name: "보물 받고 맵으로" }).click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
    }
  }
  expect(selections).toBe(4);
  expect(branches).toBeGreaterThanOrEqual(1);
  expect(merges).toBeGreaterThanOrEqual(1);
  expect(visitedKinds).toContain("rest");
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  await expect(page.getByRole("heading", { name: "갈림길의 숲" })).toBeVisible();
  expect((await page.evaluate(() => localStorage.getItem("chesslayer.run.v1")))?.includes('"rest_upgrade"')).toBe(true);
  await page.screenshot({
    path: "test-results/map-rest-route.png",
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

test("호신용품: 보호 대상 표시·설명과 저장 복원", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  const target = await page.evaluate(async () => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = new WebGame("guardian", 2026);
    const s = JSON.parse(game.dispatch(JSON.stringify({ type: "select_node", id: 2 })));
    const targets = s.pieces.filter((p: any) => p.self_defense);
    if (targets.length !== 1 || targets[0].owner !== 0) throw new Error("expected one living ally");
    localStorage.setItem("chesslayer.run.v1", game.save());
    game.free();
    return targets[0];
  });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator(".self-defense-mark")).toHaveCount(1);
  await expect(page.locator(`[data-piece-id="${target.id}"] .self-defense-mark`)).toBeVisible();
  const cell = page.locator(`[data-square="${target.square}"]`);
  await expect(cell).toHaveAccessibleName(/호신용품 보호 대상/);
  await expect(cell).toHaveAttribute("title", /포획한 적도 사망/);
  await cell.click();
  await expect(page.locator(".self-defense-info")).toContainText("포획한 적도 사망");
  await page.screenshot({ path: "test-results/self-defense.png", fullPage: true });
});

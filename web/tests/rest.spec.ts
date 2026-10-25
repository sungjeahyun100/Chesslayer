import { test, expect, type Page } from "@playwright/test";
import { readFileSync } from "node:fs";
const commands = JSON.parse(
  readFileSync("../crates/chesslayer/tests/fixtures/winning-run.json", "utf8"),
);
const saveKey = "chesslayer.run.v1";

// Build a legitimate save through the production WASM commands, including AI turns.
// No injected snapshots or test-only game APIs.
async function loadRest(page: Page, casualties = false) {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).waitFor();
  await page.evaluate(
    async ({ commands, saveKey }) => {
      const url = "/src/wasm/chesslayer_web.js";
      const { default: init, WebGame } = await import(url);
      await init();
      const game = new WebGame("vanguard", 2026);
      for (const [index, command] of commands.entries()) {
        let s = JSON.parse(game.snapshot());
        while (s.phase === "combat" && s.turn === 1)
          s = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" })));
        if (s.phase === "rest") break;
        try { game.dispatch(JSON.stringify(command)); }
        catch (e) { throw new Error(`fixture command ${index}: ${String(e)}`); }
      }
      localStorage.setItem(saveKey, game.save());
      game.free();
    },
    { commands: casualties ? JSON.parse(readFileSync("../crates/chesslayer/tests/fixtures/rest-revive.json", "utf8")) : commands, saveKey },
  );
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "rest");
}
const saved = (page: Page) =>
  page.evaluate((key) => localStorage.getItem(key), saveKey);
async function restoreSnapshot(page: Page) {
  return page.evaluate(async (key) => {
    const url = "/src/wasm/chesslayer_web.js";
    const { default: init, WebGame } = await import(url);
    await init();
    const game = WebGame.restore(localStorage.getItem(key)!);
    const snapshot = JSON.parse(game.snapshot());
    game.free();
    return snapshot;
  }, saveKey);
}
async function allImagesLoaded(page: Page) {
  await expect(page.locator(".missing-piece")).toHaveCount(0);
  expect(
    await page
      .locator("img.piece-image")
      .evaluateAll((images) =>
        images.every(
          (i) =>
            (i as HTMLImageElement).complete &&
            (i as HTMLImageElement).naturalWidth > 0,
        ),
      ),
  ).toBe(true);
}

test("휴식 트리: 신규 후속 행마가 잠긴 가지와 SVG로 표시된다", async ({ page }) => {
  await loadRest(page);
  const paths = (await restoreSnapshot(page)).upgrade_paths;
  for (const [from, to] of [[2, 18], [18, 19], [18, 20], [14, 17], [15, 21], [15, 22]])
    expect(paths.some((p: any) => p.from === from && p.to === to)).toBe(true);
  for (const [square, expected] of [
    ["b1", [[18, "available"], [19, "locked"], [20, "locked"]]],
    ["a1", [[17, "locked"]]],
    ["c1", [[21, "locked"], [22, "locked"]]],
  ] as const) {
    await page.locator(`[data-square="${square}"]`).click();
    for (const [kind, status] of expected) {
      const node = page.locator(`.skill-node[data-kind="${kind}"]`);
      await expect(node).toHaveClass(new RegExp(`\\b${status}\\b`));
      await node.click();
      await expect(page.locator(".rest-portrait img")).toHaveAttribute("data-piece-key", paths.find((p: any) => p.to === kind).to_key);
    }
    await allImagesLoaded(page);
  }
});

test("휴식 트리: 상위 복합 기물의 잠금 조건과 미리보기를 표시한다", async ({ page }) => {
  await loadRest(page);
  const paths = (await restoreSnapshot(page)).upgrade_paths;
  for (const [from, to] of [[16, 23], [12, 24], [9, 25], [10, 26], [12, 27], [9, 28], [10, 29], [11, 30]]) {
    expect(paths.some((p: any) => p.from === from && p.to === to)).toBe(true);
  }
  await page.locator('[data-square="d1"]').click();
  const god = page.locator('.skill-node[data-kind="23"]');
  await expect(god).toHaveClass(/\blocked\b/);
  await expect(god).toHaveAttribute("title", /구행: 미보유[\s\S]*텐쿠\(天狗\): 미보유/);
  await god.click();
  await expect(page.locator(".rest-portrait img")).toHaveAttribute("data-piece-key", "movement_god");
  await expect(page.locator(".skill-owned.missing")).toHaveCount(2);
  await expect(page.locator(".skill-node-popup")).toHaveCount(0);
  const bouncingTempest = page.locator('.skill-node[data-kind="24"]');
  await bouncingTempest.click();
  await expect(page.locator(".rest-portrait img")).toHaveAttribute("data-piece-key", "bouncing_tempest_queen");
  await expect(page.locator(".skill-owned.missing")).toHaveCount(2);
  await allImagesLoaded(page);
});

test("강화 노드 팝업: 선택 교체, 재클릭과 빈 공간 닫기, 확대와 스크롤 추적", async ({ page }) => {
  await loadRest(page);
  await page.locator('[data-square="a2"]').click();
  const first = page.locator('.skill-node[data-kind="8"]');
  const second = page.locator('.skill-node[data-kind="13"]');
  await first.click();
  const popup = page.locator(".skill-node-popup");
  await expect(popup).toBeVisible();
  await expect(popup).toHaveText("강화 확정");
  await expect(popup.locator("button")).toHaveCount(1);
  await expect(popup.getByRole("button", { name: "강화 확정" })).toBeVisible();
  await expect(page.locator(".skill-popup-connector")).toHaveCount(0);
  await first.click();
  await expect(popup).toHaveCount(0);
  await first.click();
  await second.click();
  await expect(popup).toHaveCount(1);
  await expect(second).toHaveAttribute("aria-expanded", "true");
  await expect(first).toHaveAttribute("aria-expanded", "false");
  await expect(page.locator(".rest-piece-study h2")).toHaveText("바운싱 폰");
  await page.locator(".skill-viewport").click({ position: { x: 8, y: 8 } });
  await expect(popup).toHaveCount(0);
  await first.click();
  await page.getByRole("button", { name: "트리 확대" }).click();
  await page.getByRole("button", { name: "트리 확대" }).click();
  await page.locator(".skill-viewport").evaluate((el) => { el.scrollLeft += 80; el.scrollTop += 25; });
  await expect.poll(async () => page.evaluate(() => {
    const node = document.querySelector('.skill-node[data-kind="8"] .skill-node-orb')!.getBoundingClientRect();
    const panel = document.querySelector('.skill-node-popup')!.getBoundingClientRect();
    return panel.top >= 0 && panel.bottom <= innerHeight && panel.left >= 0 && panel.right <= innerWidth
      && (Math.abs(panel.left - node.right) < 12 || Math.abs(node.left - panel.right) < 12);
  })).toBe(true);
});

test("강화 노드 팝업: 모바일 화면 안에 배치되고 다른 노드를 선택할 수 있다", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await loadRest(page);
  await page.locator('[data-square="a2"]').click();
  await page.locator('.skill-node[data-kind="8"]').click();
  const popup = page.locator(".skill-node-popup");
  await expect(popup).toBeVisible();
  await expect(popup).toHaveAttribute("data-side", "left");
  await expect.poll(async () => popup.evaluate((el) => {
    const rect = el.getBoundingClientRect();
    return rect.left >= 0 && rect.right <= innerWidth && rect.top >= 0 && rect.bottom <= innerHeight;
  })).toBe(true);
  await page.locator('.skill-node[data-kind="13"]').click();
  await expect(popup).toHaveCount(1);
  await expect(page.locator(".rest-piece-study h2")).toHaveText("바운싱 폰");
});

test("휴식: 보드 선택·강화 미리보기·취소·확정·저장·다음 전투", async ({
  page,
}) => {
  await loadRest(page);
  const before = await saved(page);
  const formation = await restoreSnapshot(page);
  expect(formation.rest_options.every((p: any) => p.dead ? p.cell === null : p.cell === p.origin)).toBe(true);
  expect(formation.pieces.every((p: any) => !p.ward && !p.self_defense)).toBe(true);
  await expect(page.locator(".rest-grid, .outcome, .turn-panel")).toHaveCount(
    0,
  );
  await page.locator('[data-square="a8"]').click();
  await expect(page.locator(".rest-detail")).toHaveCount(0);
  await page.locator('[data-square="a3"]').click();
  await expect(page.locator(".rest-detail")).toHaveCount(0);
  await page.locator('[data-square="e1"]').click();
  await page.locator(".skill-node").click();
  await expect(
    page.getByText("이 기물에 정의된 강화 경로가 없습니다."),
  ).toBeVisible();
  await page.locator('[data-square="a2"]').click();
  await expect(page.locator('[data-square="a2"]')).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.locator(".skill-node.available small")).toHaveText(["선택 가능", "선택 가능"]);
  await expect(page.locator(".skill-node")).toHaveCount(3);
  await expect(page.locator(".skill-connection.available")).toHaveCount(2);
  // Each real branch is a separate connected node, not a row of upgrade cards.
  const positions = await page.locator(".skill-node").evaluateAll((nodes) => nodes.map((n) => ({
    x: (n as HTMLElement).style.left, y: (n as HTMLElement).style.top,
  })));
  expect(new Set(positions.map((p) => p.y)).size).toBe(3);
  expect(new Set(positions.map((p) => p.x)).size).toBe(2);
  await page
    .getByRole("button", { name: "템페스트 폰 · 선택 가능 · 상세 보기" })
    .click();
  await expect(page.locator(".rest-portrait img")).toHaveAttribute(
    "data-piece-key",
    "tempest_pawn",
  );
  expect(await saved(page)).toBe(before);
  await page.locator('[data-square="a8"]').click();
  await expect(page.locator(".rest-selected")).toHaveAttribute(
    "data-square",
    "a2",
  );
  await expect(page.getByRole("alert")).toHaveCount(0);
  await page.getByRole("button", { name: "기물 선택 닫기" }).click();
  await expect(page.locator(".rest-detail")).toHaveCount(0);
  expect(await saved(page)).toBe(before);
  await page.locator('[data-square="a2"]').click();
  await page
    .getByRole("button", { name: "템페스트 폰 · 선택 가능 · 상세 보기" })
    .click();
  await allImagesLoaded(page);
  await page.screenshot({
    path: "test-results/rest-upgrade-desktop.png",
    fullPage: true,
    animations: "disabled",
  });
  await page.getByRole("button", { name: "강화 확정", exact: true }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  await expect(page.getByRole("status").filter({ hasText: "강화 완료" })).toBeVisible();
  await expect(page.locator(".skill-connection.acquired")).toHaveCount(1);
  await expect(page.locator(".skill-connection.locked")).toHaveCount(1);
  await page.getByRole("button", { name: "바운싱 폰 · 잠김 · 상세 보기" }).click();
  await expect(page.locator(".skill-lock-reason")).toContainText("이 가지로 변경하는 강화 경로는 없습니다");
  await expect(page.locator(".skill-requirements")).toContainText("살아 있는 일반 폰");
  await expect(page.getByRole("button", { name: "강화 확정", exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "템페스트 폰 · 현재 기물 · 상세 보기" }).click();
  await expect(page.locator(".skill-inspector")).toContainText("현재 기물에 적용된 능력");
  await page.screenshot({ path: "test-results/rest-tree-acquired.png", fullPage: true, animations: "disabled" });
  await page.getByRole("button", { name: "맵으로 돌아가기 →", exact: true }).click();
  await expect(page.locator(".act-map-screen")).toBeVisible();
  const snapshot = await restoreSnapshot(page);
  expect(snapshot.pieces.find((p: any) => p.id === 9).key).toBe("tempest_pawn");
  expect(snapshot.rest_options.find((p: any) => p.id === 9).upgrades).toEqual([
    "tempest",
  ]);
  await page.getByRole("button", { name: "원정 현황" }).click();
  await page.locator(".status-piece-group").filter({ hasText: "템페스트 폰" }).click();
  await expect(page.locator(".status-upgrade-history")).toContainText("폰 → 템페스트 폰");
  const entry = snapshot.next_battle_pieces.find((piece: any) => piece.id === 9);
  await expect(page.locator(`.status-square[aria-label="${entry.square} 아군 템페스트 폰"]`)).toBeVisible();
  await page.getByRole("button", { name: "원정 현황 닫기" }).click();
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator('[data-node-id="13"]').click();
  await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute(
    "data-piece-key",
    "tempest_pawn",
  );
  await page.locator('[data-square="a3"]').click();
  await expect(page.locator('[data-square="a4"]')).toHaveAttribute(
    "data-legal",
    "true",
  );
});

test("모바일 휴식: 사망 기물만 원점 부활·취소·저장 후 재개", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await loadRest(page, true);
  const before = await saved(page);
  await page.locator('[data-square="f1"]').click();
  await expect(page.getByText("이 기물을 되살리시겠습니까?")).toBeVisible();
  await expect(page.locator(".rest-portrait img")).toHaveAttribute(
    "data-piece-key",
    "bishop",
  );
  await expect(page.locator(".rest-ghost")).toHaveCount(1);
  await expect(page.locator(".rest-ghost img")).toBeVisible();
  expect(await saved(page)).toBe(before);
  await page.keyboard.press("Escape");
  await expect(page.locator(".rest-ghost")).toHaveCount(0);
  await page.locator('[data-square="f1"]').click();
  await allImagesLoaded(page);
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: "test-results/rest-revive-mobile.png",
    fullPage: true,
    animations: "disabled",
  });
  await page
    .getByRole("button", { name: "부활 확정" })
    .scrollIntoViewIfNeeded();
  await expect(page.locator(".board")).toBeInViewport({ ratio: 1 });
  await page.getByRole("button", { name: "부활 확정" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  let snapshot = await restoreSnapshot(page);
  expect(snapshot.pieces.find((p: any) => p.id === 6)).toMatchObject({
    key: "bishop",
    cell: 61,
  });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator('[data-node-id="14"]').click();
  snapshot = await restoreSnapshot(page);
  expect(snapshot.phase).toBe("combat");
  expect(snapshot.pieces.find((p: any) => p.id === 6)?.key).toBe("bishop");
});

test("휴식: 룩·비숍·나이트·퀸의 분기 강화와 저장·진형 계승", async ({ page }) => {
  await loadRest(page);
  const original = (await saved(page))!;
  const before = await restoreSnapshot(page);
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const pieces = [
    { id: 1, square: "a1", name: "룩", key: "rook", families: ["tempest", "bouncing"] },
    { id: 3, square: "c1", name: "비숍", key: "bishop", families: ["tempest", "bouncing"] },
    { id: 2, square: "b1", name: "나이트", key: "knight", families: ["tempest", "knightrider"] },
    { id: 4, square: "d1", name: "퀸", key: "queen", families: ["tempest", "bouncing"] },
  ];
  for (const piece of pieces) {
    for (const family of piece.families) {
      // Replay a real rest save for each independent choice; never inject a board.
      await page.evaluate(({ key, value }) => localStorage.setItem(key, value), {
        key: saveKey, value: original,
      });
      await page.reload();
      await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
      await page.locator(`[data-square="${piece.square}"]`).click();
      await expect(page.locator(".skill-node.available")).toHaveCount(piece.families.length);
      const name = family === "knightrider" ? "나이트라이더" : `${family === "tempest" ? "템페스트" : "바운싱"} ${piece.name}`;
      const key = family === "knightrider" ? "knightrider" : `${family}_${piece.key}`;
      await page.getByRole("button", { name: `${name} · 선택 가능 · 상세 보기` }).click();
      await expect(page.locator(".rest-portrait img")).toHaveAttribute("data-piece-key", key);
      await expect(page.locator(`[data-piece-id="${piece.id}"] img`)).toHaveAttribute("data-piece-key", key);
      expect(await saved(page)).toBe(original);
      await allImagesLoaded(page);
      if (key === "bouncing_queen") {
        await page.screenshot({ path: "test-results/rest-queen-branches.png", fullPage: true, animations: "disabled" });
      }
      await page.getByRole("button", { name: "강화 확정", exact: true }).click();
      await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
      const after = await restoreSnapshot(page);
      const prior = before.rest_options.find((p: any) => p.id === piece.id);
      const upgraded = after.rest_options.find((p: any) => p.id === piece.id);
      expect(upgraded).toMatchObject({
        id: piece.id, key, cell: prior.cell, origin: prior.origin, ward: prior.ward,
        upgrades: [...prior.upgrades, family],
      });
      expect(JSON.parse((await saved(page))!).commands.at(-1)).toMatchObject({
        type: "rest_upgrade", id: piece.id, target: upgraded.kind,
      });
      expect(after.log.at(-1)).toContain(`${piece.name} → ${name}`);
      await page.reload();
      await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
      await page.locator('[data-node-id="13"]').click();
      await expect(page.locator(`[data-piece-id="${piece.id}"] img`)).toHaveAttribute("data-piece-key", key);
      const next = await restoreSnapshot(page);
      expect(next.rest_options.find((p: any) => p.id === piece.id)).toMatchObject({
        key, cell: ({ 1: 58, 2: 51, 3: 49, 4: 50 } as Record<number, number>)[piece.id], origin: prior.origin, ward: prior.ward, upgrades: [...prior.upgrades, family],
      });
      const deployedCell = next.pieces.find((p: any) => p.id === piece.id).cell;
      const deployedSquare = `${"abcdefgh"[deployedCell % 8]}${8 - Math.floor(deployedCell / 8)}`;
      await page.locator(`[data-square="${deployedSquare}"]`).click();
      const destinations = new Set(next.moves.filter((m: any) => m.from === deployedCell).map((m: any) => m.to));
      await expect(page.locator('[data-legal="true"]')).toHaveCount(destinations.size);
    }
  }
  expect(errors).toEqual([]);
});

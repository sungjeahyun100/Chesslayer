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
    async ({ commands, casualties, saveKey }) => {
      const url = "/src/wasm/chesslayer_web.js";
      const { default: init, WebGame } = await import(url);
      await init();
      const game = new WebGame("vanguard", 2026);
      for (const command of commands) {
        let s = JSON.parse(game.snapshot());
        while (s.phase === "combat" && s.turn === 1)
          s = JSON.parse(game.dispatch(JSON.stringify({ type: "enemy_step" })));
        if (s.phase === "rest") break;
        if (
          casualties &&
          command.type === "move" &&
          command.from === 12 &&
          command.to === 5
        ) {
          const jeans = s.hand.find((c: any) => c.definition.kind === "jeans");
          game.dispatch(JSON.stringify({ type: "play_card", id: jeans.id }));
        }
        game.dispatch(
          JSON.stringify(
            casualties && command.type === "move" && command.from === 59
              ? { type: "move", from: 55, to: 47 }
              : command,
          ),
        );
      }
      localStorage.setItem(saveKey, game.save());
      game.free();
    },
    { commands, casualties, saveKey },
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

test("휴식: 보드 선택·강화 미리보기·취소·확정·저장·다음 전투", async ({
  page,
}) => {
  await loadRest(page);
  const before = await saved(page);
  await expect(page.locator(".rest-grid, .outcome, .turn-panel")).toHaveCount(
    0,
  );
  await page.locator('[data-square="a8"]').click();
  await expect(page.locator(".rest-detail")).toHaveCount(0);
  await page.locator('[data-square="a3"]').click();
  await expect(page.locator(".rest-detail")).toHaveCount(0);
  await page.locator('[data-square="e1"]').click();
  await expect(
    page.getByText("이 기물에 정의된 강화 경로가 없습니다."),
  ).toBeVisible();
  await page.locator('[data-square="a2"]').click();
  await expect(page.locator('[data-square="a2"]')).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.locator(".tree-status")).toHaveText(["선택 가능", "선택 가능"]);
  await page
    .getByRole("button", { name: "폰 → 템페스트 폰 강화 미리보기" })
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
  await page.getByRole("button", { name: "취소", exact: true }).click();
  await expect(page.locator(".rest-detail")).toHaveCount(0);
  expect(await saved(page)).toBe(before);
  await page.locator('[data-square="a2"]').click();
  await page
    .getByRole("button", { name: "폰 → 템페스트 폰 강화 미리보기" })
    .click();
  await allImagesLoaded(page);
  await page.screenshot({
    path: "test-results/rest-upgrade-desktop.png",
    fullPage: true,
    animations: "disabled",
  });
  await page.getByRole("button", { name: "강화 확정", exact: true }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "map");
  const snapshot = await restoreSnapshot(page);
  expect(snapshot.pieces.find((p: any) => p.id === 9).key).toBe("tempest_pawn");
  expect(snapshot.rest_options.find((p: any) => p.id === 9).upgrades).toEqual([
    "tempest",
  ]);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator('[data-node-id="14"]').click();
  await expect(page.locator('[data-piece-id="9"] img')).toHaveAttribute(
    "data-piece-key",
    "tempest_pawn",
  );
  await page.locator('[data-square="a2"]').click();
  await expect(page.locator('[data-square="a3"]')).toHaveAttribute(
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
  await page.locator('[data-square="a1"]').click();
  await expect(page.getByText("이 기물을 되살리시겠습니까?")).toBeVisible();
  await expect(page.locator(".rest-portrait img")).toHaveAttribute(
    "data-piece-key",
    "rook",
  );
  await expect(page.locator(".rest-ghost")).toHaveCount(1);
  await expect(page.locator(".rest-ghost img")).toBeVisible();
  expect(await saved(page)).toBe(before);
  await page.keyboard.press("Escape");
  await expect(page.locator(".rest-ghost")).toHaveCount(0);
  await page.locator('[data-square="a1"]').click();
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
  expect(snapshot.pieces.find((p: any) => p.id === 1)).toMatchObject({
    key: "rook",
    cell: 56,
  });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator('[data-node-id="14"]').click();
  await expect(page.locator('[data-square="a1"]')).toHaveAccessibleName(
    "a1 백 룩",
  );
  snapshot = await restoreSnapshot(page);
  expect(snapshot.phase).toBe("combat");
  expect(snapshot.pieces.find((p: any) => p.id === 1).cell).toBe(56);
});

test("휴식: 룩·비숍·나이트·퀸의 7개 분기 강화와 저장·진형 계승", async ({ page }) => {
  await loadRest(page);
  const original = (await saved(page))!;
  const before = await restoreSnapshot(page);
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  const pieces = [
    { id: 1, square: "a1", name: "룩", key: "rook", families: ["tempest", "bouncing"] },
    { id: 3, square: "g7", name: "비숍", key: "bishop", families: ["tempest", "bouncing"] },
    { id: 2, square: "b1", name: "나이트", key: "knight", families: ["tempest"] },
    { id: 4, square: "d3", name: "퀸", key: "queen", families: ["tempest", "bouncing"] },
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
      await expect(page.locator(".upgrade-path")).toHaveCount(piece.families.length);
      const name = `${family === "tempest" ? "템페스트" : "바운싱"} ${piece.name}`;
      const key = `${family}_${piece.key}`;
      await page.getByRole("button", { name: `${piece.name} → ${name} 강화 미리보기` }).click();
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
      await page.locator('[data-node-id="14"]').click();
      await expect(page.locator(`[data-piece-id="${piece.id}"] img`)).toHaveAttribute("data-piece-key", key);
      const next = await restoreSnapshot(page);
      expect(next.rest_options.find((p: any) => p.id === piece.id)).toMatchObject({
        key, cell: prior.cell, ward: prior.ward, upgrades: [...prior.upgrades, family],
      });
      await page.locator(`[data-square="${piece.square}"]`).click();
      const destinations = new Set(next.moves.filter((m: any) => m.from === prior.cell).map((m: any) => m.to));
      await expect(page.locator('[data-legal="true"]')).toHaveCount(destinations.size);
    }
  }
  expect(errors).toEqual([]);
});

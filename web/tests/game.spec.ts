import { test, expect } from "@playwright/test";

async function start(
  page: import("@playwright/test").Page,
  character = "선봉대",
) {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: new RegExp(character) }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
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

test("카드부터 보상·계승·역습·휴식·보스 승리까지 실제 클릭으로 완주", async ({
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
  await start(page);
  let continues = 0;
  for (const cmd of commands) {
    if (cmd.type === "move") {
      const from = page.locator(`[data-square="${square(cmd.from)}"]`);
      await expect(from).toBeEnabled();
      await from.click();
      await expect(
        page.locator(`[data-square="${square(cmd.to)}"]`),
      ).toHaveAttribute("data-legal", "true");
      await page.locator(`[data-square="${square(cmd.to)}"]`).click();
    } else if (cmd.type === "play_card") {
      await page.locator(`[data-card-id="${cmd.id}"]`).click();
      const targetCell = Number(
        await page
          .locator(`[data-piece-id="${cmd.target}"]`)
          .getAttribute("data-cell"),
      );
      await page.locator(`[data-square="${square(targetCell)}"]`).click();
    } else if (cmd.type === "choose_reward") {
      await expect(
        page.getByRole("heading", { name: "전투 승리", exact: true }),
      ).toBeVisible();
      await page
        .getByRole("button", { name: "보상 가호", exact: true })
        .click();
    } else if (cmd.type === "continue") {
      await page
        .getByRole("button", { name: /다음 전투 진행|쉼터로 이동/ })
        .click();
      continues++;
      if (continues === 1) {
        await expect(page.locator('[data-square="e8"]')).toHaveAccessibleName(
          "e8 흑 나이트 로얄",
        );
        await expect(page.locator('[data-square="d4"]')).toHaveAccessibleName(
          "d4 백 폰",
        );
        await expect(page.locator('[data-square="e7"]')).toHaveAccessibleName(
          "e7 백 비숍",
        );
        await expect(page.getByTestId("deck-count")).toHaveText("11");
      }
      if (continues === 3) {
        await expect(page.getByRole("log")).toContainText("역습");
        await expect(page.locator('[data-square="e8"]')).toHaveAccessibleName(
          "e8 흑 킹 로얄",
        );
        await expect(page.locator('[data-square="a2"]')).toHaveAccessibleName(
          "a2 백 템페스트 폰",
        );
      }
      if (continues === 5)
        await expect(
          page.getByRole("heading", { name: "마하라자", exact: true }),
        ).toBeVisible();
    } else if (cmd.type === "rest_upgrade") {
      await expect(
        page.getByRole("heading", { name: "작은 쉼터", exact: true }),
      ).toHaveCount(2);
      await page.getByRole("button", { name: "♙ a2의 폰 → 템페스트" }).click();
    }
  }
  await expect(page.locator("main")).toHaveAttribute("data-phase", "victory");
  await expect(
    page.getByRole("heading", { name: "마지막 왕관까지." }),
  ).toBeVisible();
  await page.screenshot({ path: "test-results/victory.png", fullPage: true });
  expect(errors).toEqual([]);
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

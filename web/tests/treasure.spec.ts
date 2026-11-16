import { test, expect } from "@playwright/test";
import { dispatch, newRun, snapshot, winHillBattle } from "./helpers/uiRuns";

test("보물방에서 유물을 받지 않고 떠나면 보유품을 유지한 채 다음 노드로 진행한다", async ({ page }) => {
  const game = newRun("guardian", 2026);
  let saved: string;
  let before: ReturnType<typeof snapshot>;
  let treasureId: number;
  try {
    const nodes = snapshot(game).map.nodes;
    const treasure = nodes.find(node => node.kind === "treasure" && node.row === 1)!;
    const start = nodes.find(node => node.row === 0 && node.next.includes(treasure.id))!;
    const king = snapshot(game).pieces.find(piece => piece.key === "king")!;
    dispatch(game, { type: "developer_set_upgrade", id: king.id, target: 94 });
    winHillBattle(game, start.id);
    dispatch(game, { type: "continue_reward" });
    dispatch(game, { type: "select_node", id: treasure.id });
    before = snapshot(game);
    treasureId = treasure.id;
    saved = game.save();
  } finally { game.free(); }

  await page.goto("/");
  await page.evaluate(value => localStorage.setItem("chesslayer.run.v1", value), saved);
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "treasure");
  await expect(page.getByRole("button", { name: "유물을 받지 않고 떠나기 →" })).toBeVisible();
  await page.getByRole("button", { name: "유물을 받지 않고 떠나기 →" }).click();
  await expect(page.locator(".act-map-screen")).toBeVisible();

  const after = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const restored = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try { return JSON.parse(restored.snapshot()); } finally { restored.free(); }
  });
  expect(after.deck).toEqual(before.deck);
  expect(after.relics).toEqual(before.relics);
  expect(after.map.nodes.find((node: { id: number }) => node.id === treasureId).visited).toBe(true);
  expect(after.map.nodes.some((node: { available: boolean }) => node.available)).toBe(true);
});

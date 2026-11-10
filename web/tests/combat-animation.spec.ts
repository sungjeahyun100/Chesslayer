import { expect, test } from "@playwright/test";
import { combatPlan, combatPresentationSteps } from "../src/game/combatPlan";
import type { CombatEvent, Piece, Snapshot } from "../src/types";
import { dispatch, newRun, snapshot } from "./helpers/uiRuns";

const piece = (id: number, key: string, cell: number, ward = false): Piece => ({
  id, key, cell, ward, kind: id, owner: id === 1 ? 0 : 1,
  name: key, description: "", square: "", royal: false, self_defense: false,
  temporary_upgrade: false, limiter: null, absorbed_moves: [], statuses: [], dice_roll: null,
});
const state = (pieces: Piece[], log: string[] = []): Snapshot =>
  ({ phase: "combat", pieces, log } as Snapshot);

test("the presentation plan follows the engine result for a guarded capture and a missed shot", () => {
  const attacker = piece(1, "knight", 52);
  const guarded = piece(2, "king", 36, true);
  const before = state([attacker, guarded]);
  const after = state([attacker, { ...guarded, ward: false }], ["가호: 공격을 막았습니다."]);
  const capture = combatPlan(before, after, { type: "move", from: 52, to: 36 });
  expect(capture).toMatchObject({ kind: "return", result: "ward", impact: 110, duration: 250 });
  const corpse = combatPlan(state([attacker, { ...guarded, ward: false }]), state([attacker, { ...guarded, key: "corpse", ward: false }]),
    { type: "move", from: 52, to: 36 });
  expect(corpse).toMatchObject({ kind: "return", result: "corpse" });

  const shooter = piece(1, "machine_gun_drone", 52);
  const target = piece(2, "king", 36);
  const shot = combatPlan(state([shooter, target]), state([shooter, target], ["사격 빗나감"]),
    { type: "special_action", from: 52, to: 36 });
  expect(shot).toMatchObject({ kind: "tracer", result: "miss", count: 10, interval: 40 });
  const fullLog = Array.from({ length: 60 }, (_, index) => `이전 기록 ${index}`);
  const capped = combatPlan(state([shooter, target], fullLog),
    state([shooter, target], [...fullLog.slice(1), "사격 빗나감"]),
    { type: "special_action", from: 52, to: 36 });
  expect(capped?.result).toBe("miss");
  const pistol = combatPlan(state([piece(1, "pistol_infantry", 52), target]),
    state([piece(1, "pistol_infantry", 52)]), { type: "special_action", from: 52, to: 36 });
  expect(pistol).toMatchObject({ kind: "tracer", count: 1, result: "hit" });
  const mortar = combatPlan(state([piece(1, "mortar", 52), target]),
    state([piece(1, "mortar", 52)]), { type: "special_action", from: 52, to: 36 });
  expect(mortar).toMatchObject({ kind: "mortar", impact: 400 });
  const boss = piece(1, "tribal_dictator", 52);
  expect(combatPlan(state([boss, target]), state([boss, target]),
    { type: "enemy_step_selected", from: 52, index: 0 }, { enemyTarget: 36 })).toBeNull();
  expect(combatPlan(state([boss, target]), state([boss, target], ["부족의 독재자: 3갈래 샷건 발사"]),
    { type: "enemy_step_selected", from: 52, index: 0 }, { enemyTarget: 36 }))
    .toBeNull();
});

test("shotgun presentation uses all engine rays and individual results without inferring from a log or target marker", () => {
  const boss = piece(100, "tribal_dictator", 28);
  const targets = [piece(1, "pawn", 19), piece(2, "king", 12, true), piece(3, "rook", 21)];
  const event: CombatEvent = { kind: "shotgun", actor_id: 100, from: 28, toward: 20,
    area: [19, 20, 12, 21], rays: [
      { direction: [-1, -1], cells: [19] },
      { direction: [0, -1], cells: [20, 12] },
      { direction: [1, -1], cells: [21] },
    ], hits: [
      { cell: 19, piece_id: 1, result: "hit" },
      { cell: 12, piece_id: 2, result: "ward" },
      { cell: 21, piece_id: 3, result: "miss" },
    ] };
  const after = { ...state([boss, { ...targets[1], ward: false }, targets[2]]), combat_events: [event] };
  const plan = combatPlan(state([boss, ...targets]), after,
    { type: "enemy_step_selected", from: 28, index: 0 });
  expect(plan).toMatchObject({ kind: "shotgun", origin: 28, target: 20, blast: event.area, shotgun: event });
  expect(plan?.victims?.map((victim) => victim.id)).toEqual([1]);
  expect(plan?.blast).not.toContain(4); // The second ray stops at its first blocker.
});

test("a real hit and automatic hook play in order without revealing the pulled square before the chain arrives", () => {
  const shooter = piece(13, "rifleman", 52);
  const boss = piece(100, "tribal_dictator", 4);
  const before = { ...state([shooter, boss]), turn: 0 };
  const after: Snapshot = { ...state([{ ...shooter, cell: 11 }, boss], ["소총수 사격 명중 (80%)."]),
    turn: 1, combat_events: [{ kind: "chain_pull", boss_id: 100, boss_cell: 4, piece_id: 13, from: 52, to: 11 }] };
  const steps = combatPresentationSteps(before, after, { type: "special_action", from: 52, to: 4 });
  expect(steps.map((step) => step.plan.kind)).toEqual(["tracer", "chain"]);
  expect(steps[0].after.pieces.find((entry) => entry.id === 13)?.cell).toBe(52);
  expect(steps[1].plan).toMatchObject({ origin: 4, chain: { from: 52, to: 11, piece: { id: 13 } } });
  expect(steps[1].after).toBe(after);
  expect(before.pieces[0].cell).toBe(52);
  const missed = { ...before, turn: 1, log: ["사격 빗나감"], combat_events: [] };
  expect(combatPresentationSteps(before, missed, { type: "special_action", from: 52, to: 4 })
    .map((step) => step.plan.kind)).toEqual(["tracer"]);
});

test("the WASM queries return exact blast cells and the selected enemy action target", () => {
  const artillery = newRun("artillery");
  try {
    dispatch(artillery, { type: "developer_set_upgrade", id: 2, target: 83 });
    dispatch(artillery, { type: "developer_set_upgrade", id: 2, target: 85 });
    expect(JSON.parse(artillery.blast_cells(57))).toHaveLength(9);
  } finally { artillery.free(); }

  const guardian = newRun();
  try {
    const node = snapshot(guardian).map.nodes.find((entry) => entry.available && entry.kind === "combat")!;
    dispatch(guardian, { type: "select_node", id: node.id });
    dispatch(guardian, { type: "end_turn" });
    const [from, index] = JSON.parse(guardian.recommend_enemy()) as [number, number];
    const actor = snapshot(guardian).pieces.find((entry) => entry.cell === from)!;
    const target = guardian.enemy_action_target(from, index);
    const after = dispatch(guardian, { type: "enemy_step_selected", from, index });
    expect(after.pieces.find((entry) => entry.id === actor.id)?.cell).toBe(target);
  } finally { guardian.free(); }
});

test("a legal board move animates while input is gated and ends at the committed square", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  const skip = page.getByRole("button", { name: "층 전환 연출 건너뛰기" });
  if (await skip.isVisible()) await skip.click();
  await page.locator(".map-node:enabled").first().click();
  await expect(page.locator("main")).toHaveAttribute("data-phase", "combat");
  const move = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try {
      const snapshot = JSON.parse(game.snapshot()) as Snapshot;
      return snapshot.moves.find((entry) => !snapshot.pieces.some((p) => p.cell === entry.to))!;
    } finally { game.free(); }
  });
  await page.locator(`.square:nth-child(${move.from + 1})`).click();
  await page.locator(`.square:nth-child(${move.to + 1})`).click();
  await expect(page.locator(".combat-moving-piece")).toHaveCount(1);
  await expect(page.locator(".board .square:enabled")).toHaveCount(0);
  await expect(page.locator(".combat-moving-piece")).toHaveCount(0);
  await expect(page.locator(`.piece[data-cell="${move.to}"]`)).toHaveCount(1);
  const saved = await page.evaluate(() => JSON.parse(localStorage.getItem("chesslayer.run.v1")!));
  expect(saved.commands.at(-1)).toMatchObject({ type: "move", from: move.from, to: move.to });
});

test("a tactical nuke uses the engine blast cells and shows a mushroom cloud", async ({ page }) => {
  await page.goto("/");
  const blastCount = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const progress = { highest_floor: 4, unlocked_squads: ["guardian", "vanguard", "heretic", "artillery", "special"], final_unlocked: true, final_defeated: false };
    const game = WebGame.start_run("artillery", 2026, 1, JSON.stringify(progress));
    game.set_developer_mode(true);
    game.dispatch(JSON.stringify({ type: "developer_set_upgrade", id: 2, target: 83 }));
    game.dispatch(JSON.stringify({ type: "developer_set_upgrade", id: 2, target: 85 }));
    const map = JSON.parse(game.snapshot()) as Snapshot;
    game.dispatch(JSON.stringify({ type: "select_node", id: map.map.nodes.find((node) => node.available && node.kind === "combat")!.id }));
    const count = (JSON.parse(game.blast_cells(57)) as number[]).length;
    localStorage.setItem("chesslayer.run.v1", game.save());
    localStorage.setItem("chesslayer.run.developer.v1", "true");
    game.free();
    return count;
  });
  await page.reload();
  await page.getByRole("button", { name: "저장된 원정 이어하기" }).click();
  await page.locator('.square[data-square="b1"]').click();
  await page.getByRole("button", { name: /전술핵 자폭/ }).click();
  await page.locator('.square[data-square="b1"]').click();
  await expect(page.locator(".combat-mushroom")).toHaveCount(1);
  await expect(page.locator(".combat-blast-cell")).toHaveCount(blastCount);
  await expect(page.locator(".combat-mushroom")).toHaveCount(0);
  await expect(page.locator('.piece[data-piece-id="2"]')).toHaveCount(0);
});

test("rifle fire shows a three-shot tracer burst while the shooter stays put", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "개발자 모드 켜기" }).click();
  await page.locator("button.character").filter({ hasText: "포병대대" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  const skip = page.getByRole("button", { name: "층 전환 연출 건너뛰기" });
  if (await skip.isVisible()) await skip.click();
  await page.locator('[data-node-id="0"]').click();
  await page.locator('.square[data-square="e2"]').click();
  await page.getByRole("button", { name: /특수 능력.*대상 선택/ }).click();
  await page.locator('.square[data-square="e8"]').click();
  await expect(page.locator(".combat-projectiles")).toHaveCount(1);
  await expect(page.locator('.piece[data-piece-id="13"][data-cell="52"]')).toHaveCount(1);
  await expect(page.locator(".combat-projectiles")).toHaveCount(0);
  const saved = await page.evaluate(() => JSON.parse(localStorage.getItem("chesslayer.run.v1")!));
  expect(saved.commands.at(-1)).toMatchObject({ type: "special_action", from: 52, to: 4 });
});

test("reduced motion applies the committed move without a moving overlay", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await page.getByRole("button", { name: "새 게임 시작" }).click();
  await page.getByRole("button", { name: "런 시작" }).click();
  await page.locator(".map-node:enabled").first().click();
  const move = await page.evaluate(async () => {
    const { default: init, WebGame } = await import("/src/wasm/chesslayer_web.js");
    await init();
    const game = WebGame.restore(localStorage.getItem("chesslayer.run.v1")!);
    try {
      const snapshot = JSON.parse(game.snapshot()) as Snapshot;
      return snapshot.moves.find((entry) => !snapshot.pieces.some((p) => p.cell === entry.to))!;
    } finally { game.free(); }
  });
  await page.locator(`.square:nth-child(${move.from + 1})`).click();
  await page.locator(`.square:nth-child(${move.to + 1})`).click();
  await expect(page.locator(".combat-moving-piece")).toHaveCount(0);
  await expect(page.locator(`.piece[data-cell="${move.to}"]`)).toHaveCount(1);
});
